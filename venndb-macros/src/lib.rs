#![forbid(unsafe_code)]

mod errors;
mod field;
mod generate_db;
mod parse_attrs;

use errors::Errors;
use field::StructField;
use parse_attrs::{FieldAttrs, TypeAttrs};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};

/// Derive macro generating VennDB functionality for this struct.
///
/// See <https://docs.rs/venndb> for more information on how to use it.
/// Or check out the README and usage tests in [the repository][repo] of this macro.
///
/// [repo]: https://github.com/plabayo/venndb
#[proc_macro_derive(VennDB, attributes(venndb))]
pub fn venndb(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    let gen: TokenStream = impl_from_args(&ast);
    gen.into()
}

/// Transform the input into a token stream containing any generated implementations,
/// as well as all errors that occurred.
fn impl_from_args(input: &syn::DeriveInput) -> TokenStream {
    let errors = &Errors::default();
    let type_attrs = &TypeAttrs::parse(errors, input);
    let mut output_tokens = match &input.data {
        syn::Data::Struct(ds) => impl_from_args_struct(
            errors,
            &input.vis,
            &input.ident,
            type_attrs,
            &input.generics,
            ds,
        ),
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
    vis: &syn::Visibility,
    name: &syn::Ident,
    type_attrs: &TypeAttrs,
    _generic_args: &syn::Generics,
    ds: &syn::DataStruct,
) -> TokenStream {
    let fields = match &ds.fields {
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

    let fields: Vec<_> = fields
        .named
        .iter()
        .filter_map(|field| {
            let attrs = FieldAttrs::parse(errors, field);
            StructField::new(errors, field, attrs)
        })
        .collect();

    let name_db = match &type_attrs.name {
        Some(name) => format_ident!("{}", name.value()),
        None => format_ident!("{}DB", name),
    };

    let db_code = generate_db::generate_db(
        name,
        &name_db,
        type_attrs.validator.as_ref(),
        vis,
        &fields[..],
    );

    quote! {
        #db_code
    }
}
