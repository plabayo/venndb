#![forbid(unsafe_code)]

use proc_macro::TokenStream;
use quote::{format_ident, quote};
use syn::{Data::Struct, DataStruct, DeriveInput, Fields::Named, FieldsNamed};

#[proc_macro_derive(VennDB)]
pub fn venndb(item: TokenStream) -> TokenStream {
    let ast: DeriveInput = syn::parse2(item.into()).unwrap();

    let name_db = format_ident!("{}DB", ast.ident);

    let _fields = match ast.data {
        Struct(DataStruct {
            fields: Named(FieldsNamed { ref named, .. }),
            ..
        }) => named,
        _ => {
            return syn::Error::new_spanned(ast, "Only Structs with named fields are supported")
                .to_compile_error()
                .into()
        }
    };

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
    .into()
}
