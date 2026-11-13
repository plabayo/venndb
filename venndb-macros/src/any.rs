//! `#[derive(Any)]`

use crate::errors::Errors;
use proc_macro2::TokenStream;
use quote::{ToTokens, quote};

/// Implement `venndb::Any` for an enum, of which the variants marked `#[any]` are the wildcards.
pub fn impl_any(input: &syn::DeriveInput) -> TokenStream {
    let errors = &Errors::default();
    let mut output = TokenStream::new();

    match &input.data {
        syn::Data::Enum(data) => {
            let patterns: Vec<_> = data
                .variants
                .iter()
                .filter(|variant| is_any(errors, variant))
                .map(|variant| {
                    let name = &variant.ident;
                    match &variant.fields {
                        syn::Fields::Unit => quote! { Self::#name },
                        syn::Fields::Unnamed(_) => quote! { Self::#name(..) },
                        syn::Fields::Named(_) => quote! { Self::#name { .. } },
                    }
                })
                .collect();
            if patterns.is_empty() {
                errors.err(
                    &input.ident,
                    "`#[derive(Any)]` needs a variant marked with `#[any]`",
                );
            } else {
                let name = &input.ident;
                let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
                output = quote! {
                    impl #impl_generics ::venndb::Any for #name #ty_generics #where_clause {
                        fn is_any(&self) -> bool {
                            ::std::matches!(self, #(#patterns)|*)
                        }
                    }
                };
            }
        }
        syn::Data::Struct(_) | syn::Data::Union(_) => errors.err(
            &input.ident,
            "`#[derive(Any)]` can only be applied to enums, implement `venndb::Any` for other types",
        ),
    }

    errors.to_tokens(&mut output);
    output
}

/// Whether the variant is marked with a plain `#[any]`.
fn is_any(errors: &Errors, variant: &syn::Variant) -> bool {
    let mut any = false;
    for attr in &variant.attrs {
        if attr.path().is_ident("any") {
            any = errors.expect_meta_word(&attr.meta).is_some();
        }
    }
    any
}
