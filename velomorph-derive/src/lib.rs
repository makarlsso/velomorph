//! Derive macro crate for Velomorph.
//!
//! Use `#[derive(Morph)]` on a target struct to generate a `TryMorph`
//! implementation from a source input type.
//! The source defaults to `RawInput`, or can be configured with
//! `#[morph(from = "TypePath")]`.

use proc_macro::TokenStream;
use quote::quote;
use syn::{Data, DeriveInput, Fields, LitStr, Type, parse_macro_input};

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
        let f_name = &f.ident;
        let is_target_option = is_type_name(&f.ty, "Option");
        let is_target_cow = is_type_name(&f.ty, "Cow");

        if is_target_cow {
            // Zero-copy: borrow data into a Cow.
            quote! { #f_name: std::borrow::Cow::from(old.#f_name) }
        } else if is_target_option {
            // Passthrough: preserve optional state.
            quote! { #f_name: old.#f_name }
        } else {
            // Strict: requires a value or returns MorphError.
            let err_field = f_name.as_ref().unwrap().to_string();
            quote! {
                #f_name: old.#f_name.ok_or(velomorph::MorphError::MissingField(#err_field.to_string()))?
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
