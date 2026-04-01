//! Derive macro crate for Velomorph.
//!
//! Use `#[derive(Morph)]` on a target struct or enum to generate a `TryMorph`
//! implementation from a source input type.
//!
//! - The **source type** defaults to `RawInput<...>`, or can be configured with
//!   a *type-level* attribute: `#[morph(from = "TypePath<'a>")]`.
//! - Individual **fields** can map from differently named source fields via a
//!   *field-level* attribute: `#[morph(from = "source_field_name")]`.
//! - Field-level controls include `with`, `default`, and `skip`.
//! - Type-level validation is supported via `#[morph(validate = "path::to::fn")]`.
//! - Enums support same-name variant mapping, plus variant overrides with
//!   `#[morph(from = "SourceVariant")]`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{
    Data, DeriveInput, Expr, Field, Fields, FieldsNamed, Ident, LitStr, Path, Type,
    parse_macro_input, spanned::Spanned,
};

/// Derives a `velomorph::TryMorph<Target>` implementation.
///
/// By default, the macro uses `RawInput` as the source type.
/// You can override this with:
///
/// ```ignore
/// #[derive(Morph)]
/// #[morph(from = "MySource<'a>")]
/// struct MyTarget<'a> { /* ... */ }
/// ```
///
/// Generated mapping behavior:
/// - `Option<T>` target fields are passed through from source as-is.
/// - `Cow<'a, str>` target fields are created with `Cow::from(source_field)`.
/// - Any other target field expects an `Option<T>` source field and returns
///   `MorphError::MissingField` when `None`.
/// - `#[morph(with = "...")]` currently expects a transform function returning
///   `Result<TargetField, E>`.
#[proc_macro_derive(Morph, attributes(morph))]
pub fn derive_morph(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    expand_morph(input).unwrap_or_else(|err| err.to_compile_error().into())
}

fn expand_morph(input: DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let type_attrs = parse_type_morph_attrs(&input)?;
    let from_type = type_attrs
        .from_type
        .unwrap_or_else(|| quote!(RawInput #ty_generics));

    let body = match &input.data {
        Data::Struct(data) => expand_struct_body(name, data, &type_attrs.validate_path)?,
        Data::Enum(data) => expand_enum_body(name, data, &type_attrs.validate_path)?,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "Velomorph can only be derived for structs or enums.",
            ));
        }
    };

    let expanded = quote! {
        #[cfg(feature = "janitor")]
        impl #impl_generics velomorph::TryMorph<#name #ty_generics> for #from_type #where_clause {
            fn try_morph(self, janitor: &velomorph::Janitor) -> Result<#name #ty_generics, velomorph::MorphError> {
                let _ = janitor;
                let old = self;
                #body
            }
        }

        #[cfg(not(feature = "janitor"))]
        impl #impl_generics velomorph::TryMorph<#name #ty_generics> for #from_type #where_clause {
            fn try_morph(self) -> Result<#name #ty_generics, velomorph::MorphError> {
                let old = self;
                #body
            }
        }
    };

    Ok(TokenStream::from(expanded))
}

#[derive(Default)]
struct TypeMorphAttrs {
    from_type: Option<proc_macro2::TokenStream>,
    validate_path: Option<Path>,
}

enum DefaultKind {
    TraitDefault,
    Expr(Expr),
}

#[derive(Default)]
struct FieldMorphAttrs {
    from: Option<Ident>,
    with_path: Option<Path>,
    default_kind: Option<DefaultKind>,
    skip: bool,
}

fn expand_struct_body(
    target_name: &Ident,
    data: &syn::DataStruct,
    validate_path: &Option<Path>,
) -> syn::Result<proc_macro2::TokenStream> {
    let fields = match &data.fields {
        Fields::Named(named) => &named.named,
        _ => {
            return Err(syn::Error::new(
                data.fields.span(),
                "Velomorph requires named fields for structs.",
            ));
        }
    };

    let field_mappings = build_named_field_mappings(fields)?;
    let validate = generate_validate_call(validate_path);
    Ok(quote! {
        let mapped = #target_name {
            #(#field_mappings),*
        };
        #validate
        Ok(mapped)
    })
}

fn expand_enum_body(
    target_name: &Ident,
    data: &syn::DataEnum,
    validate_path: &Option<Path>,
) -> syn::Result<proc_macro2::TokenStream> {
    let mut arms = Vec::with_capacity(data.variants.len());

    for variant in &data.variants {
        let target_variant = &variant.ident;
        let source_variant = resolve_source_variant_ident(variant)?;
        let validate = generate_validate_call(validate_path);
        let arm = match &variant.fields {
            Fields::Unit => {
                quote! {
                    Self::#source_variant => {
                        let mapped = #target_name::#target_variant;
                        #validate
                        Ok(mapped)
                    }
                }
            }
            Fields::Named(named) => {
                let destructures = build_named_source_destructures(named)?;
                let mappings = build_named_variant_mappings(named)?;
                quote! {
                    Self::#source_variant { #(#destructures),* } => {
                        let mapped = #target_name::#target_variant { #(#mappings),* };
                        #validate
                        Ok(mapped)
                    }
                }
            }
            Fields::Unnamed(unnamed) => {
                let mut source_bindings = Vec::with_capacity(unnamed.unnamed.len());
                let mut target_exprs = Vec::with_capacity(unnamed.unnamed.len());
                for (idx, field) in unnamed.unnamed.iter().enumerate() {
                    let binding = syn::Ident::new(&format!("__v{}", idx), field.span());
                    source_bindings.push(binding.clone());
                    target_exprs.push(build_positional_field_expr(field, &binding)?);
                }
                quote! {
                    Self::#source_variant( #(#source_bindings),* ) => {
                        let mapped = #target_name::#target_variant( #(#target_exprs),* );
                        #validate
                        Ok(mapped)
                    }
                }
            }
        };
        arms.push(arm);
    }

    Ok(quote! {
        match old {
            #(#arms),*
        }
    })
}

fn build_named_field_mappings(
    fields: &syn::punctuated::Punctuated<Field, syn::token::Comma>,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut field_mappings = Vec::with_capacity(fields.len());
    for field in fields {
        let target_name = field
            .ident
            .as_ref()
            .expect("Velomorph requires named fields");
        let attrs = parse_field_morph_attrs(field)?;
        let source_name = attrs.from.clone().unwrap_or_else(|| target_name.clone());
        let expr = build_field_expr(field, &attrs, quote!(old.#source_name))?;
        field_mappings.push(quote!(#target_name: #expr));
    }
    Ok(field_mappings)
}

fn build_named_source_destructures(
    fields: &FieldsNamed,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut destructures = Vec::with_capacity(fields.named.len());
    for field in &fields.named {
        let attrs = parse_field_morph_attrs(field)?;
        if attrs.skip {
            continue;
        }
        let source_ident = attrs
            .from
            .clone()
            .unwrap_or_else(|| field.ident.clone().expect("named field"));
        let binding = source_binding_for_field(field);
        if source_ident == binding {
            destructures.push(quote!(#source_ident));
        } else {
            destructures.push(quote!(#source_ident: #binding));
        }
    }
    Ok(destructures)
}

fn build_named_variant_mappings(
    fields: &FieldsNamed,
) -> syn::Result<Vec<proc_macro2::TokenStream>> {
    let mut mappings = Vec::with_capacity(fields.named.len());
    for field in &fields.named {
        let target_name = field.ident.as_ref().expect("named field");
        let attrs = parse_field_morph_attrs(field)?;
        let expr = if attrs.skip {
            quote!(std::default::Default::default())
        } else {
            let binding = source_binding_for_field(field);
            build_field_expr(field, &attrs, quote!(#binding))?
        };
        mappings.push(quote!(#target_name: #expr));
    }
    Ok(mappings)
}

fn build_positional_field_expr(
    field: &Field,
    binding: &Ident,
) -> syn::Result<proc_macro2::TokenStream> {
    let attrs = parse_field_morph_attrs(field)?;
    if attrs.from.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "`from` is not supported on tuple enum fields.",
        ));
    }
    if attrs.skip {
        return Ok(quote!(std::default::Default::default()));
    }
    build_field_expr(field, &attrs, quote!(#binding))
}

fn build_field_expr(
    field: &Field,
    attrs: &FieldMorphAttrs,
    source_expr: proc_macro2::TokenStream,
) -> syn::Result<proc_macro2::TokenStream> {
    if attrs.skip {
        return Ok(quote!(std::default::Default::default()));
    }
    if attrs.default_kind.is_some() && is_type_name(&field.ty, "Option") {
        return Err(syn::Error::new(
            field.span(),
            "`default` is only valid for strict Option<T> -> T mappings.",
        ));
    }

    if let Some(path) = &attrs.with_path {
        return Ok(quote! {
            #path(#source_expr).map_err(|e| velomorph::MorphError::TransformError(e.to_string()))?
        });
    }

    if is_type_name(&field.ty, "Cow") {
        return Ok(quote!(std::borrow::Cow::from(#source_expr)));
    }
    if is_type_name(&field.ty, "Option") {
        return Ok(source_expr);
    }

    let field_name = field_name_for_error(field);
    if let Some(default_kind) = &attrs.default_kind {
        return Ok(match default_kind {
            DefaultKind::TraitDefault => quote! {
                match #source_expr {
                    Some(value) => value,
                    None => std::default::Default::default(),
                }
            },
            DefaultKind::Expr(expr) => quote! {
                match #source_expr {
                    Some(value) => value,
                    None => #expr,
                }
            },
        });
    }

    Ok(quote! {
        #source_expr.ok_or(velomorph::MorphError::MissingField(#field_name.to_string()))?
    })
}

fn field_name_for_error(field: &Field) -> String {
    field
        .ident
        .as_ref()
        .map(ToString::to_string)
        .unwrap_or_else(|| "unknown_field".to_string())
}

fn source_binding_for_field(field: &Field) -> Ident {
    let target_name = field.ident.as_ref().expect("named field");
    syn::Ident::new(&format!("__src_{}", target_name), target_name.span())
}

fn generate_validate_call(validate_path: &Option<Path>) -> proc_macro2::TokenStream {
    match validate_path {
        Some(path) => quote! {
            #path(&mapped).map_err(|e| velomorph::MorphError::ValidationError(e.to_string()))?;
        },
        None => quote! {},
    }
}

fn is_type_name(ty: &Type, target: &str) -> bool {
    if let Type::Path(tp) = ty {
        return tp.path.segments.last().is_some_and(|s| s.ident == target);
    }
    false
}

fn parse_type_morph_attrs(input: &DeriveInput) -> syn::Result<TypeMorphAttrs> {
    let mut result = TypeMorphAttrs::default();
    let ty_generics = &input.generics.split_for_impl().1;
    for attr in &input.attrs {
        if !attr.path().is_ident("morph") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                let lit: LitStr = meta.value()?.parse()?;
                let path = syn::parse_str::<syn::Path>(&lit.value()).map_err(|_| {
                    syn::Error::new(
                        lit.span(),
                        "Malformed #[morph(from = \"...\")] on struct. `from` must be a valid Rust type path string.",
                    )
                })?;
                let has_generics = path
                    .segments
                    .last()
                    .is_some_and(|segment| !segment.arguments.is_empty());

                result.from_type = Some(if has_generics {
                    quote!(#path)
                } else {
                    quote!(#path #ty_generics)
                });
                return Ok(());
            }
            if meta.path.is_ident("validate") {
                let lit: LitStr = meta.value()?.parse()?;
                let path = syn::parse_str::<syn::Path>(&lit.value()).map_err(|_| {
                    syn::Error::new(
                        lit.span(),
                        "Malformed #[morph(validate = \"...\")] on type. `validate` must be a valid Rust path string.",
                    )
                })?;
                result.validate_path = Some(path);
                return Ok(());
            }
            Err(meta.error("Unsupported type-level morph key. Expected one of: from, validate"))
        })?;
    }
    Ok(result)
}

fn parse_field_morph_attrs(field: &Field) -> syn::Result<FieldMorphAttrs> {
    let mut parsed = FieldMorphAttrs::default();
    for attr in &field.attrs {
        if !attr.path().is_ident("morph") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                let lit: LitStr = meta.value()?.parse()?;
                let name = lit.value();
                parsed.from = Some(syn::Ident::new(&name, meta.path.span()));
                return Ok(());
            }
            if meta.path.is_ident("with") {
                let lit: LitStr = meta.value()?.parse()?;
                let path = syn::parse_str::<syn::Path>(&lit.value()).map_err(|_| {
                    syn::Error::new(
                        lit.span(),
                        "Malformed #[morph(with = \"...\")] attribute. `with` must be a valid Rust path string.",
                    )
                })?;
                parsed.with_path = Some(path);
                return Ok(());
            }
            if meta.path.is_ident("default") {
                if meta.input.is_empty() {
                    parsed.default_kind = Some(DefaultKind::TraitDefault);
                } else {
                    let lit: LitStr = meta.value()?.parse()?;
                    let expr = syn::parse_str::<Expr>(&lit.value()).map_err(|_| {
                        syn::Error::new(
                            lit.span(),
                            "Malformed #[morph(default = \"...\")] expression.",
                        )
                    })?;
                    parsed.default_kind = Some(DefaultKind::Expr(expr));
                }
                return Ok(());
            }
            if meta.path.is_ident("skip") {
                if !meta.input.is_empty() {
                    return Err(meta.error("`skip` does not take a value."));
                }
                parsed.skip = true;
                return Ok(());
            }
            Err(meta.error(
                "Unsupported field morph key. Expected one of: from, with, default, skip",
            ))
        })?;
    }
    if parsed.skip && parsed.with_path.is_some() {
        return Err(syn::Error::new(
            field.span(),
            "`skip` and `with` cannot be used together.",
        ));
    }
    Ok(parsed)
}

fn resolve_source_variant_ident(variant: &syn::Variant) -> syn::Result<Ident> {
    for attr in &variant.attrs {
        if !attr.path().is_ident("morph") {
            continue;
        }
        let mut source: Option<Ident> = None;
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                let lit: LitStr = meta.value()?.parse()?;
                source = Some(Ident::new(&lit.value(), variant.ident.span()));
                return Ok(());
            }
            Err(meta.error("Unsupported enum variant morph key. Expected: from"))
        })?;
        if let Some(source_ident) = source {
            return Ok(source_ident);
        }
        return Err(syn::Error::new(
            attr.span(),
            "Malformed #[morph(...)] on enum variant. Expected: #[morph(from = \"SourceVariant\")].",
        ));
    }
    Ok(variant.ident.clone())
}
