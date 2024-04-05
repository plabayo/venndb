use quote::ToTokens;

use crate::errors::Errors;

/// Attributes applied to a field of a `#![derive(VennDB)]` struct.
#[derive(Default)]
pub struct FieldAttrs {
    pub kind: Option<FieldKind>,
}

pub enum FieldKind {
    Key,
    Filter,
}

impl FieldAttrs {
    pub fn parse(errors: &Errors, field: &syn::Field) -> Self {
        let mut this = Self::default();

        let mut skipped = false;
        let mut is_key = false;

        for attr in &field.attrs {
            let ml = if let Some(ml) = venndb_attr_to_meta_list(errors, attr) {
                ml
            } else {
                continue;
            };

            for meta in ml {
                let name = meta.path();
                if name.is_ident("key") {
                    is_key = true;
                } else if name.is_ident("skip") {
                    skipped = true;
                } else {
                    errors.err(
                        &meta,
                        concat!(
                            "Invalid field-level `venndb` attribute\n",
                            "Expected one of: `key`",
                        ),
                    );
                }
            }
        }

        if skipped {
            this.kind = None;
        } else if is_key {
            this.kind = Some(FieldKind::Key);
        } else if is_bool(&field.ty) {
            this.kind = Some(FieldKind::Filter);
        }

        this
    }
}

fn is_bool(ty: &syn::Type) -> bool {
    if let syn::Type::Path(syn::TypePath { path, .. }) = ty {
        path.is_ident("bool")
    } else {
        if ty.to_token_stream().to_string().contains("bool") {
            panic!(
                "Expected bool, found {:?}",
                ty.to_token_stream().to_string()
            );
        }
        false
    }
}

/// Represents a `#[derive(VennDB)]` type's top-level attributes.
#[derive(Default)]
pub struct TypeAttrs {
    pub name: Option<syn::LitStr>,
}

impl TypeAttrs {
    /// Parse top-level `#[venndb(...)]` attributes
    pub fn parse(errors: &Errors, derive_input: &syn::DeriveInput) -> Self {
        let mut this = Self::default();

        for attr in &derive_input.attrs {
            let ml = if let Some(ml) = venndb_attr_to_meta_list(errors, attr) {
                ml
            } else {
                continue;
            };

            for meta in ml {
                let name = meta.path();
                if name.is_ident("name") {
                    if let Some(m) = errors.expect_meta_name_value(&meta) {
                        this.name = errors.expect_lit_str(&m.value).cloned();
                    }
                } else {
                    errors.err(
                        &meta,
                        concat!(
                            "Invalid field-level `venndb` attribute\n",
                            "Expected one of: `name`",
                        ),
                    );
                }
            }
        }

        this
    }
}

/// Filters out non-`#[venndb(...)]` attributes and converts to a sequence of `syn::Meta`.
fn venndb_attr_to_meta_list(
    errors: &Errors,
    attr: &syn::Attribute,
) -> Option<impl IntoIterator<Item = syn::Meta>> {
    if !is_venndb_attr(attr) {
        return None;
    }
    let ml = errors.expect_meta_list(&attr.meta)?;
    errors.ok(ml.parse_args_with(
        syn::punctuated::Punctuated::<syn::Meta, syn::Token![,]>::parse_terminated,
    ))
}

// Whether the attribute is one like `#[<name> ...]`
fn is_matching_attr(name: &str, attr: &syn::Attribute) -> bool {
    attr.path().segments.len() == 1 && attr.path().segments[0].ident == name
}

/// Checks for `#[venndb ...]`
fn is_venndb_attr(attr: &syn::Attribute) -> bool {
    is_matching_attr("venndb", attr)
}
