#![forbid(unsafe_code)]

use errors::Errors;
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

mod errors;

/// Entrypoint for `#[derive(VennDB)]`.
#[proc_macro_derive(VennDB, attributes(venndb))]
pub fn venndb(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let gen = impl_from_args(&ast);
    gen.into()
}

/// Transform the input into a token stream containing any generated implementations,
/// as well as all errors that occurred.
fn impl_from_args(input: &syn::DeriveInput) -> TokenStream {
    let errors = &Errors::default();
    let mut output_tokens = match &input.data {
        syn::Data::Struct(ds) => impl_from_args_struct(errors, &input.ident, &input.generics, ds),
        syn::Data::Enum(_) => {
            errors.err(input, "`#[derive(VennDB)]` cannot be applied to enums");
            TokenStream::new()
        }
        syn::Data::Union(_) => {
            errors.err(input, "`#[derive(VennDB)]` cannot be applied to unions");
            TokenStream::new()
        }
    };
    errors.to_tokens(&mut output_tokens);
    output_tokens
}

/// Implements `VennDB` for a `#[derive(VennDB)]` struct.
fn impl_from_args_struct(
    errors: &Errors,
    name: &syn::Ident,
    _generic_args: &syn::Generics,
    ds: &syn::DataStruct,
) -> TokenStream {
    let _fields = match &ds.fields {
        syn::Fields::Named(fields) => fields,
        syn::Fields::Unnamed(_) => {
            errors.err(
                &ds.struct_token,
                "`#![derive(VennDB)]` is not currently supported on tuple structs",
            );
            return TokenStream::new();
        }
        syn::Fields::Unit => {
            errors.err(
                &ds.struct_token,
                "#![derive(VennDB)]` cannot be applied to unit structs",
            );
            return TokenStream::new();
        }
    };

    let name_db = format_ident!("{}DB", name);

    quote! {
        #[non_exhaustive]
        struct #name_db;

        impl #name_db {
            fn new() -> Self {
                Self
            }
        }

        impl Default for #name_db {
            fn default() -> Self {
                Self::new()
            }
        }
    }
}
