use crate::field::{FieldInfo, StructField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::Ident;

/// Generate the venndb logic
pub fn generate_db(
    name: &Ident,
    name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[StructField],
) -> TokenStream {
    let fields: Vec<_> = fields.iter().filter_map(StructField::info).collect();

    let db_struct = generate_db_struct(name, name_db, vis, &fields[..]);
    let db_struct_methods = generate_db_struct_methods(name, name_db, vis, &fields[..]);

    let db_query = generate_query_struct(name, name_db, vis, &fields[..]);

    quote! {
        #db_struct

        #db_struct_methods

        #db_query
    }
}

fn generate_db_struct(
    name: &Ident,
    name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let db_fields: Vec<_> = fields
        .iter()
        .map(|info| match info {
            FieldInfo::Key(field) => {
                let field_name = field.map_name();
                let ty: &syn::Type = field.ty();
                quote! {
                    #field_name: ::venndb::__internal::HashMap<#ty, usize>,
                }
            }
            FieldInfo::Filter(field) => {
                let field_name = field.filter_name();
                let field_name_not = field.filter_not_name();
                quote! {
                    #field_name: ::venndb::__internal::BitVec,
                    #field_name_not: ::venndb::__internal::BitVec,
                }
            }
        })
        .collect();

    let db_doc = format!(
        "An in-memory database for storing instances of [`{}`], generated by `#[derive(VennDB)]`.",
        name
    );
    quote! {
        #[doc=#db_doc]
        #[derive(Debug, Default)]
        #vis struct #name_db {
            rows: Vec<#name>,
            #(#db_fields)*
        }
    }
}

fn generate_db_struct_methods(
    name: &Ident,
    name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let method_new = generate_db_struct_method_new(name, name_db, vis, fields);
    let method_with_capacity = generate_db_struct_method_with_capacity(name, name_db, vis, fields);
    let field_methods = generate_db_struct_field_methods(name, name_db, vis, fields);
    let method_append = generate_db_struct_method_append(name, name_db, vis, fields);

    quote! {
        impl #name_db {
            #method_new

            #method_with_capacity

            /// Return the number of rows in the database.
            #vis fn len(&self) -> usize {
                self.rows.len()
            }

            /// Return the capacity of the database,
            /// which is automatically grown as needed.
            #vis fn capacity(&self) -> usize {
                self.rows.capacity()
            }

            /// Return `true` if the database is empty.
            #vis fn is_empty(&self) -> bool {
                self.rows.is_empty()
            }

            #field_methods

            #method_append
        }
    }
}

pub fn generate_db_struct_method_new(
    name: &Ident,
    _name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let method_doc = format!(
        "Construct a new empty database for storing instances of [`{}`].",
        name
    );

    let db_fields_initialisers: Vec<_> = fields
        .iter()
        .map(|info| match info {
            FieldInfo::Key(field) => {
                let name = field.map_name();
                quote! {
                    #name: ::venndb::__internal::HashMap::new(),
                }
            }
            FieldInfo::Filter(field) => {
                let name = field.filter_name();
                let name_not = field.filter_not_name();
                quote! {
                    #name: ::venndb::__internal::BitVec::new(),
                    #name_not: ::venndb::__internal::BitVec::new(),
                }
            }
        })
        .collect();

    quote! {
        #[doc=#method_doc]
        #vis fn new() -> Self {
            Self {
                rows: Vec::new(),
                #(#db_fields_initialisers)*
            }
        }
    }
}

pub fn generate_db_struct_method_with_capacity(
    name: &Ident,
    _name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let method_doc = format!(
        "Construct a new empty database for storing instances of [`{}`] with a given capacity.",
        name
    );

    let db_fields_initialisers_with_capacity: Vec<_> = fields
        .iter()
        .map(|info| match info {
            FieldInfo::Key(field) => {
                let name = field.map_name();
                quote! {
                    #name: ::venndb::__internal::HashMap::with_capacity(capacity),
                }
            }
            FieldInfo::Filter(field) => {
                let name = field.filter_name();
                let name_not = field.filter_not_name();
                quote! {
                    #name: ::venndb::__internal::BitVec::with_capacity(capacity),
                    #name_not: ::venndb::__internal::BitVec::with_capacity(capacity),
                }
            }
        })
        .collect();

    quote! {
        #[doc=#method_doc]
        #vis fn with_capacity(capacity: usize) -> Self {
            Self {
                rows: Vec::new(),
                #(#db_fields_initialisers_with_capacity)*
            }
        }
    }
}

pub fn generate_db_struct_method_append(
    name: &Ident,
    _name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let method_doc = format!("Append a new instance of [`{}`] to the database.", name);

    let db_field_inserts: Vec<_> = fields
        .iter()
        .map(|info| match info {
            FieldInfo::Key(field) => {
                let map_name = field.map_name();
                let field_name = field.name();

                quote! {
                    self.#map_name.insert(data.#field_name.clone(), index);
                }
            }
            FieldInfo::Filter(field) => {
                let name = field.name();
                let field_name = field.filter_name();
                let field_name_not = field.filter_not_name();
                quote! {
                    self.#field_name.push(data.#name);
                    self.#field_name_not.push(!data.#name);
                }
            }
        })
        .collect();

    quote! {
        #[doc=#method_doc]
        #vis fn append(&mut self, data: #name) {
            let index = self.rows.len();

            #(#db_field_inserts)*

            self.rows.push(data);
        }
    }
}

pub fn generate_db_struct_field_methods(
    name: &Ident,
    _name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let db_key_methods: Vec<_> = fields
        .iter()
        .filter_map(|info| match info {
            FieldInfo::Key(field) => {
                let map_name = field.map_name();
                let ty = field.ty();
                let method_name = field.method_name();
                let doc = format!(
                    "Get an instance of [`{}`] by its key `{}`, if it exists in the database.",
                    name,
                    field.name()
                );
                Some(quote! {
                    #[doc=#doc]
                    #vis fn #method_name<Q>(&self, key: &Q) -> ::std::option::Option<&#name>
                        where
                            #ty: ::std::borrow::Borrow<Q>,
                            Q: ::std::hash::Hash + ::std::cmp::Eq + ?::std::marker::Sized,
                    {
                        self.#map_name.get(key).and_then(|index| self.rows.get(*index))
                    }
                })
            }
            FieldInfo::Filter(_) => None,
        })
        .collect();

    quote! {
        #(#db_key_methods)*
    }
}

fn generate_query_struct(
    name: &Ident,
    name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let name_query = format_ident!("{}Query", name_db);

    let query_fields: Vec<_> = fields
        .iter()
        .filter_map(|info| match info {
            FieldInfo::Filter(field) => {
                let name = field.name();
                Some(quote! {
                    #name: Option<bool>,
                })
            }
            FieldInfo::Key(_) => None,
        })
        .collect();

    if query_fields.is_empty() {
        return TokenStream::new();
    }

    let query_field_initialisers: Vec<_> = fields
        .iter()
        .filter_map(|info| match info {
            FieldInfo::Filter(field) => {
                let name = field.name();
                Some(quote! {
                    #name: None,
                })
            }
            FieldInfo::Key(_) => None,
        })
        .collect();

    let query_impl = generate_query_struct_impl(name, name_db, &name_query, vis, fields);

    let query_doc = format!(
        "A query object for filtering instances of [`{}`], within [`{}`], generated by `#[derive(VennDB)]`.",
        name, name_db
    );

    quote! {
        #[doc=#query_doc]
        #[derive(Debug)]
        #vis struct #name_query<'a> {
            db: &'a #name_db,
            #(#query_fields)*
        }

        impl<'a> #name_query<'a> {
            fn new(db: &'a #name_db) -> Self {
                Self {
                    db,
                    #(#query_field_initialisers)*
                }
            }
        }

        #query_impl

        impl #name_db {
            #vis fn query(&self) -> #name_query {
                #name_query::new(&self)
            }
        }
    }
}

fn generate_query_struct_impl(
    name: &Ident,
    _name_db: &Ident,
    name_query: &Ident,
    vis: &syn::Visibility,
    fields: &[FieldInfo],
) -> TokenStream {
    let filter_setters: Vec<_> = fields
        .iter()
        .filter_map(|info| match info {
            FieldInfo::Filter(field) => {
                let name = field.name();
                let doc = format!("Enable and set the `{}` filter.", name);
                Some(quote! {
                    #[doc=#doc]
                    #vis fn #name(&mut self, value: bool) -> &mut Self {
                        self.#name = Some(value);
                        self
                    }
                })
            }
            FieldInfo::Key(_) => None,
        })
        .collect();

    let filters: Vec<_> = fields
        .iter()
        .filter_map(|info| match info {
            FieldInfo::Filter(field) => {
                let name = field.name();
                let filter_name: Ident = field.filter_name();
                let filter_not_name: Ident = field.filter_not_name();
                Some(quote! {
                    match self.#name {
                        Some(true) => filter &= &self.db.#filter_name,
                        Some(false) => filter &= &self.db.#filter_not_name,
                        None => (),
                    };
                })
            }
            FieldInfo::Key(_) => None,
        })
        .collect();

    let name_query_result = format_ident!("{}Result", name_query);

    let name_query_result_doc = format!(
        "Contains a reference to the found instances of [`{}`] if there is at least one found, queried using [`{}`], generated by `#[derive(VennDB)]`.",
        name, name_query
    );

    let name_query_result_iter = format_ident!("{}Iter", name_query_result);

    let name_query_result_iter_doc = format!(
        "An iterator over the found instances of [`{}`] queried using [`{}`], generated by `#[derive(VennDB)]`.",
        name, name_query
    );

    quote! {
        impl<'a> #name_query<'a> {
            #(#filter_setters)*

            /// Execute the query on the database, returning an iterator over the results.
            #vis fn execute(&self) -> Option<#name_query_result<'a>> {
                let mut filter = ::venndb::__internal::bitvec![1; self.db.rows.len()];

                #(#filters)*

                if filter.any() {
                    Some(#name_query_result {
                        rows: &self.db.rows,
                        v: filter,
                    })
                } else {
                    None
                }
            }
        }

        #[doc=#name_query_result_doc]
        #[derive(Debug)]
        #vis struct #name_query_result<'a> {
            rows: &'a [#name],
            v: ::venndb::__internal::BitVec,
        }

        impl<'a> #name_query_result<'a> {
            #vis fn first(&self) -> &'a #name {
                let index = self.v.iter_ones().next().expect("should contains at least one result");
                &self.rows[index]
            }

            #vis fn iter(&self) -> #name_query_result_iter<'a, '_> {
                #name_query_result_iter {
                    rows: self.rows,
                    iter_ones: self.v.iter_ones(),
                }
            }
        }

        #[doc=#name_query_result_iter_doc]
        #vis struct #name_query_result_iter<'a, 'b> {
            rows: &'a [#name],
            iter_ones: ::venndb::__internal::IterOnes<'b, usize, ::venndb::__internal::Lsb0>,
        }

        impl<'a, 'b> Iterator for #name_query_result_iter<'a, 'b> {
            type Item = &'a #name;

            fn next(&mut self) -> Option<Self::Item> {
                self.iter_ones.next().map(|index| &self.rows[index])
            }
        }
    }
}
