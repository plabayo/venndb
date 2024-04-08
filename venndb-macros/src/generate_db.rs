use crate::field::{FieldInfo, StructField};
use proc_macro2::TokenStream;
use quote::{format_ident, quote, ToTokens};
use syn::Ident;

/// Generate the venndb logic
pub fn generate_db(
    name: &Ident,
    name_db: &Ident,
    vis: &syn::Visibility,
    fields: &[StructField],
) -> TokenStream {
    let fields: Vec<_> = fields.iter().filter_map(StructField::info).collect();

    let db_error = DbError::new(&fields[..]);

    let db_struct = generate_db_struct(name, name_db, vis, &fields[..]);
    let db_struct_methods = generate_db_struct_methods(name, name_db, vis, &db_error, &fields[..]);

    let db_query = generate_query_struct(name, name_db, vis, &fields[..]);

    let db_error_definitions = db_error.generate_definitions(name_db, vis);

    quote! {
        #db_struct

        #db_struct_methods

        #db_query

        #db_error_definitions
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
    db_error: &DbError,
    fields: &[FieldInfo],
) -> TokenStream {
    let method_new = generate_db_struct_method_new(name, name_db, vis, fields);
    let method_with_capacity = generate_db_struct_method_with_capacity(name, name_db, vis, fields);
    let method_from_rows =
        generate_db_struct_method_from_rows(name, name_db, vis, db_error, fields);
    let field_methods = generate_db_struct_field_methods(name, name_db, vis, fields);
    let method_append = generate_db_struct_method_append(name, name_db, vis, db_error, fields);

    quote! {
        #[allow(clippy::unused_unit)]
        impl #name_db {
            #method_new

            #method_with_capacity

            #method_from_rows

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

            /// Return an iterator over the rows in the database.
            #vis fn iter(&self) -> impl ::std::iter::Iterator<Item = &#name> {
                self.rows.iter()
            }

            #field_methods

            #method_append

            /// Consumes the database and returns the rows.
            #vis fn into_rows(self) -> Vec<#name> {
                self.rows
            }
        }
    }
}

fn generate_db_struct_method_new(
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

fn generate_db_struct_method_with_capacity(
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

fn generate_db_struct_method_from_rows(
    name: &Ident,
    name_db: &Ident,
    vis: &syn::Visibility,
    db_error: &DbError,
    _fields: &[FieldInfo],
) -> TokenStream {
    let method_doc = format!(
        "Construct a new database from the given set of [`{}`] rows.",
        name
    );

    let return_type = db_error.generate_fn_output(name_db, quote! { Vec<#name> }, quote! { Self });
    let append_internal_call = db_error.generate_fn_error_kind_usage(
        name_db,
        quote! {
            db.append_internal(row, index)
        },
        quote! {
            rows
        },
    );
    let fn_result = db_error.generate_fn_return_value_ok(quote! { db });

    quote! {
        #[doc=#method_doc]
        #vis fn from_rows(rows: Vec<#name>) -> #return_type {
            let mut db = Self::with_capacity(rows.len());
            for (index, row) in rows.iter().enumerate() {
                #append_internal_call
            }
            db.rows = rows;
            #fn_result
        }
    }
}

fn generate_db_struct_method_append(
    name: &Ident,
    name_db: &Ident,
    vis: &syn::Visibility,
    db_error: &DbError,
    fields: &[FieldInfo],
) -> TokenStream {
    let method_doc = format!("Append a new instance of [`{}`] to the database.", name);

    let db_field_insert_checks: Vec<_> = fields
        .iter()
        .filter_map(|info| match info {
            FieldInfo::Key(field) => {
                let map_name = field.map_name();
                let field_name = field.name();
                let entry_field_name = format_ident!("entry_{}", field_name);
                let db_duplicate_error_kind_creation = DbError::generate_duplicate_key_error_kind_creation(
                    name_db,
                );

                Some(quote! {
                    // TODO: handle duplicate key,
                    // but only have error if we have possible error cases
                    let #entry_field_name = match self.#map_name.entry(data.#field_name.clone()) {
                        ::venndb::__internal::hash_map::Entry::Occupied(_) => return Err(#db_duplicate_error_kind_creation),
                        ::venndb::__internal::hash_map::Entry::Vacant(entry) => entry,
                    };
                })
            }
            FieldInfo::Filter(_) =>  None,
        })
        .collect();

    let db_field_insert_commits: Vec<_> = fields
        .iter()
        .map(|info| match info {
            FieldInfo::Key(field) => {
                let field_name = field.name();
                let entry_field_name = format_ident!("entry_{}", field_name);

                quote! {
                    #entry_field_name.insert(index);
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

    let append_return_type = db_error.generate_fn_output(name_db, quote! { #name }, quote! { () });
    let append_kind_return_type = db_error.generate_fn_kind_output(name_db, quote! { () });

    let append_internal_call = db_error.generate_fn_error_kind_usage(
        name_db,
        quote! {
            self.append_internal(&data, index)
        },
        quote! { data },
    );

    let append_return_output = db_error.generate_fn_return_value_ok(quote! { () });

    quote! {
        #[doc=#method_doc]
        #vis fn append(&mut self, data: #name) -> #append_return_type {
            let index = self.rows.len();
            #append_internal_call
            self.rows.push(data);
            #append_return_output
        }

        fn append_internal(&mut self, data: &#name, index: usize) -> #append_kind_return_type {
            #(#db_field_insert_checks)*
            #(#db_field_insert_commits)*
            #append_return_output
        }
    }
}

fn generate_db_struct_field_methods(
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

    let query_method_doc = format!(
        "Return a new [`{}`] for filtering instances of [`{}`].",
        name_query, name
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
            #[doc=#query_method_doc]
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

    let filter_resetters: Vec<_> = fields
        .iter()
        .filter_map(|info| match info {
            FieldInfo::Filter(field) => {
                let name = field.name();
                Some(quote! {
                    self.#name = None;
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

    let query_result_method_doc_first = format!(
        "Return the first instance of [`{}`] found by the query.",
        name
    );
    let query_result_method_doc_any = format!(
        "Return a random instance of [`{}`] found by the query.",
        name
    );
    let query_result_method_doc_iter = format!(
        "Return an iterator over the instances of [`{}`] found by the query.",
        name
    );

    quote! {
        impl<'a> #name_query<'a> {
            #(#filter_setters)*

            /// Reset the query to its initial values.
            #vis fn reset(&mut self) -> &mut Self {
                #(#filter_resetters)*
                self
            }

            // TODO: support a filter on the result based on a predicate

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
            #[doc=#query_result_method_doc_first]
            #vis fn first(&self) -> &'a #name {
                let index = self.v.iter_ones().next().expect("should contains at least one result");
                &self.rows[index]
            }

            #[doc=#query_result_method_doc_any]
            #vis fn any(&self) -> &'a #name {
                let n = ::venndb::__internal::rand_usize() % self.v.count_ones();
                let index = self.v.iter_ones().nth(n).unwrap();
                &self.rows[index]
            }

            #[doc=#query_result_method_doc_iter]
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

#[derive(Debug)]
/// Used to generate the optional error logic,
/// which we only which to generate in case operations are possible to fail.
///
/// Example: duplicate key, in case a key field is used
struct DbError {
    error_kinds: Vec<DbErrorKind>,
}

#[derive(Debug)]
enum DbErrorKind {
    DuplicateKey,
}

impl ToTokens for DbErrorKind {
    fn to_tokens(&self, tokens: &mut TokenStream) {
        match self {
            Self::DuplicateKey => {
                tokens.extend(quote! {
                    DuplicateKey,
                });
            }
        }
    }
}

impl DbError {
    fn new(fields: &[FieldInfo]) -> Self {
        let error_duplicate_key = fields.iter().any(|info| matches!(info, FieldInfo::Key(_)));
        let error_kinds = if error_duplicate_key {
            vec![DbErrorKind::DuplicateKey]
        } else {
            Vec::new()
        };

        Self { error_kinds }
    }

    fn generate_duplicate_key_error_kind_creation(name_db: &Ident) -> TokenStream {
        let ident_error_kind = format_ident!("{}ErrorKind", name_db);
        quote! {
            #ident_error_kind::DuplicateKey
        }
    }

    fn generate_fn_error_kind_usage(
        &self,
        name_db: &Ident,
        original: TokenStream,
        input: TokenStream,
    ) -> TokenStream {
        if self.error_kinds.is_empty() {
            return quote! {
                #original;
            };
        }

        let ident_error = format_ident!("{}Error", name_db);

        quote! {
            if let Err(kind) = #original {
                return Err(#ident_error::new(kind, #input, index));
            }
        }
    }

    fn generate_fn_return_value_ok(&self, output: TokenStream) -> TokenStream {
        if self.error_kinds.is_empty() {
            return output;
        }
        quote! {
            Ok(#output)
        }
    }

    fn generate_fn_output(
        &self,
        name_db: &Ident,
        input: TokenStream,
        original: TokenStream,
    ) -> TokenStream {
        if self.error_kinds.is_empty() {
            return original;
        }

        let ident_error = format_ident!("{}Error", name_db);
        quote! {
            Result<#original, #ident_error<#input>>
        }
    }

    fn generate_fn_kind_output(&self, name_db: &Ident, original: TokenStream) -> TokenStream {
        if self.error_kinds.is_empty() {
            return original;
        }

        let ident_error_kind = format_ident!("{}ErrorKind", name_db);
        quote! {
            Result<#original, #ident_error_kind>
        }
    }

    fn generate_definitions(&self, name_db: &Ident, vis: &syn::Visibility) -> TokenStream {
        if self.error_kinds.is_empty() {
            return TokenStream::new();
        }

        let ident_error = format_ident!("{}Error", name_db);
        let ident_error_kind = format_ident!("{}Kind", ident_error);
        let ident_error_debug = format!("{}", ident_error);

        let error_kinds = &self.error_kinds;

        let doc_error_kind = format!(
            "The kind of error that occurred when appending a row to the [`{}`].",
            name_db
        );
        let doc_error = format!(
            "The error type that can be returned when appending a row to the [`{}`].",
            name_db
        );
        let doc_error_kind_method = format!(
            "The [`{}`] that occurred when appending a row to the [`{}`].",
            ident_error_kind, name_db
        );

        quote! {
            #[derive(Debug, PartialEq, Eq, Clone, Copy)]
            #[doc = #doc_error_kind]
            #vis enum #ident_error_kind {
                #(#error_kinds)*
            }

            #[doc = #doc_error]
            #vis struct #ident_error<T> {
                kind: #ident_error_kind,
                input: T,
                row_index: usize,
            }

            impl<T> ::std::fmt::Debug for #ident_error<T> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    f.debug_struct(#ident_error_debug)
                        .field("kind", &self.kind)
                        .field("row_index", &self.row_index)
                        .finish()
                }
            }

            impl<T> #ident_error<T> {
                /// Create a new error.
                fn new(kind: #ident_error_kind, input: T, row_index: usize) -> Self {
                    Self {
                        kind,
                        input,
                        row_index,
                    }
                }

                #[doc = #doc_error_kind_method]
                #vis fn kind(&self) -> #ident_error_kind {
                        self.kind
                }

                /// Return a reference to the input that caused the error.
                #vis fn input(&self) -> &T {
                    &self.input
                }

                /// Consume this error and return the input that caused the error.
                #vis fn into_input(self) -> T {
                    self.input
                }

                /// Return the index of the row that caused the error.
                #vis fn row_index(&self) -> usize {
                    self.row_index
                }
            }

            impl<T> ::std::fmt::Display for #ident_error<T> {
                fn fmt(&self, f: &mut ::std::fmt::Formatter<'_>) -> ::std::fmt::Result {
                    write!(f, "{}: {:?}", #ident_error_debug, self.kind)
                }
            }

            impl<T> ::std::error::Error for #ident_error<T> {}
        }
    }
}
