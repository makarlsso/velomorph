//! Derive macro crate for Velomorph.
//!
//! Use `#[derive(Morph)]` on a target struct to generate a `TryMorph`
//! implementation from a source input type.
//!
//! - The **source type** defaults to `RawInput<...>`, or can be configured with
//!   a *type-level* attribute: `#[morph(from = "TypePath<'a>")]`.
//! - Individual **fields** can map from differently named source fields via a
//!   *field-level* attribute: `#[morph(from = "source_field_name")]`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Type, parse_macro_input, spanned::Spanned};

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
#[proc_macro_derive(Morph, attributes(morph))]
pub fn derive_morph(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    // The source type can be provided via #[morph(from = "Source")].
    // If omitted, RawInput with the target generics is used by default.
    let from_type =
        resolve_from_type(&input, &ty_generics).unwrap_or_else(|| quote!(RawInput #ty_generics));

    let fields = if let Data::Struct(data) = &input.data {
        if let Fields::Named(fields) = &data.fields {
            &fields.named
        } else {
            panic!("Velomorph requires named fields.");
        }
    } else {
        panic!("Velomorph can only be derived for structs.");
    };

    let field_mappings = fields.iter().map(|f| {
        let target_name = &f.ident;
        // Allow renaming of the source field via: #[morph(from = "source_field_name")]
        let source_name = resolve_source_field_ident(f);

        let is_target_option = is_type_name(&f.ty, "Option");
        let is_target_cow = is_type_name(&f.ty, "Cow");

        if is_target_cow {
            // Zero-copy: borrow data into a Cow.
            quote! { #target_name: std::borrow::Cow::from(old.#source_name) }
        } else if is_target_option {
            // Passthrough: preserve optional state.
            quote! { #target_name: old.#source_name }
        } else {
            // Strict: requires a value or returns MorphError.
            let err_field = target_name.as_ref().unwrap().to_string();
            quote! {
                #target_name: old.#source_name.ok_or(velomorph::MorphError::MissingField(#err_field.to_string()))?
            }
        }
    });

    let expanded = quote! {
        impl #impl_generics velomorph::TryMorph<#name #ty_generics> for #from_type #where_clause {
            fn try_morph(mut self, janitor: &velomorph::Janitor) -> Result<#name #ty_generics, velomorph::MorphError> {
                let mut old = self;

                // Automatically offload a heavy field (for example: payload).
                // In a fuller implementation this would be selected via #[morph(offload)].
                if let Some(heavy) = old.payload.take() {
                    janitor.offload(heavy);
                }

                Ok(#name {
                    #(#field_mappings),*
                })
            }
        }
    };

    TokenStream::from(expanded)
}

fn is_type_name(ty: &Type, target: &str) -> bool {
    if let Type::Path(tp) = ty {
        return tp.path.segments.last().is_some_and(|s| s.ident == target);
    }
    false
}

/// Resolve which source-field identifier should back a given target field.
///
/// Defaults to using the target field's own name, but can be overridden with
/// a field attribute:
///
/// ```ignore
/// #[morph(from = "uuid_v4")]
/// pub id: u64,
/// ```
fn resolve_source_field_ident(field: &syn::Field) -> syn::Ident {
    for attr in &field.attrs {
        if !attr.path().is_ident("morph") {
            continue;
        }

        let mut result: Option<syn::Ident> = None;

        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                let lit: LitStr = meta.value()?.parse()?;
                let name = lit.value();
                // Use the target field's span for better error reporting.
                if let Some(target_ident) = &field.ident {
                    result = Some(syn::Ident::new(&name, target_ident.span()));
                } else {
                    result = Some(syn::Ident::new(&name, meta.path.span()));
                }
            }
            Ok(())
        });

        if let Some(ident) = result {
            return ident;
        }
    }

    // Fallback: use the field's own identifier (standard 1:1 mapping).
    field
        .ident
        .clone()
        .expect("Velomorph requires named fields on the target struct")
}

fn resolve_from_type(
    input: &DeriveInput,
    ty_generics: &syn::TypeGenerics<'_>,
) -> Option<proc_macro2::TokenStream> {
    let mut result = None;

    for attr in &input.attrs {
        if !attr.path().is_ident("morph") {
            continue;
        }

        // Example: #[morph(from = "SourceType")]
        let _ = attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("from") {
                let lit: LitStr = meta.value()?.parse()?;
                if let Ok(path) = syn::parse_str::<syn::Path>(&lit.value()) {
                    // If the provided path has no explicit generic arguments
                    // (e.g. "Packet"), apply the target type generics by default.
                    let has_generics = path
                        .segments
                        .last()
                        .is_some_and(|segment| !segment.arguments.is_empty());

                    result = Some(if has_generics {
                        quote!(#path)
                    } else {
                        quote!(#path #ty_generics)
                    });
                }
            }
            Ok(())
        });
    }

    result
}
