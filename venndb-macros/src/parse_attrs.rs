use quote::ToTokens;

use crate::errors::Errors;

/// Attributes applied to a field of a `#![derive(VennDB)]` struct.
#[derive(Default)]
pub struct FieldAttrs<'a> {
    pub kind: Option<FieldKind>,
    pub option_ty: Option<&'a syn::Type>,
}

pub enum FieldKind {
    /// A unique key, optionally computed from the row by a function.
    Key {
        with: Option<syn::ExprPath>,
    },
    Filter,
    FilterMap {
        any: Option<By>,
        set: Option<By>,
        item: Option<Box<syn::Type>>,
    },
}

/// How a property of a field is found.
pub enum By {
    /// By the trait made for it.
    Trait,
    /// By the function at this path.
    Path(syn::ExprPath),
}

impl<'a> FieldAttrs<'a> {
    pub fn parse(errors: &Errors, field: &'a syn::Field) -> Self {
        let mut this = Self::default();

        let mut skipped = false;
        let mut key: Option<Option<syn::ExprPath>> = None;
        let mut is_filter = false;
        let mut any: Option<By> = None;
        let mut set: Option<By> = None;
        let mut item: Option<Box<syn::Type>> = None;

        for attr in &field.attrs {
            let ml: Vec<_> = if let Some(ml) = venndb_attr_to_meta_list(errors, attr) {
                ml.into_iter().collect()
            } else {
                continue;
            };

            if ml.iter().any(|meta| meta.path().is_ident("skip")) {
                // check first to avoid any other invalid combinations
                skipped = true;
            } else {
                for meta in ml {
                    let name = meta.path();
                    if name.is_ident("key") {
                        if is_filter {
                            errors.err(
                                &meta,
                                concat!(
                                    "Invalid field-level `venndb` attribute\n",
                                    "Cannot have both `key` and `filter`",
                                ),
                            );
                        } else if any.is_some() {
                            errors.err(
                                &meta,
                                concat!(
                                    "Invalid field-level `venndb` attribute\n",
                                    "Cannot have both `key` and `any`",
                                ),
                            );
                        } else {
                            key = Some(path_value(errors, &meta));
                        }
                    } else if name.is_ident("filter") {
                        if key.is_some() {
                            errors.err(
                                &meta,
                                concat!(
                                    "Invalid field-level `venndb` attribute\n",
                                    "Cannot have both `key` and `filter`",
                                ),
                            );
                        } else {
                            is_filter = true;
                        }
                    } else if name.is_ident("any") {
                        if key.is_some() {
                            errors.err(
                                &meta,
                                concat!(
                                    "Invalid field-level `venndb` attribute\n",
                                    "Cannot have both `key` and `any`",
                                ),
                            );
                        } else {
                            any = Some(path_value(errors, &meta).map_or(By::Trait, By::Path));
                        }
                    } else if name.is_ident("set") {
                        if errors.expect_meta_word(&meta).is_some() && set.is_none() {
                            set = Some(By::Trait);
                        }
                    } else if name.is_ident("members") {
                        if let Some(path) = path_value(errors, &meta) {
                            set = Some(By::Path(path));
                        }
                    } else if name.is_ident("item") {
                        if let Some(m) = errors.expect_meta_name_value(&meta) {
                            item = match &m.value {
                                syn::Expr::Lit(syn::ExprLit {
                                    lit: syn::Lit::Str(lit),
                                    ..
                                }) => errors.ok(lit.parse()).map(Box::new),
                                value => errors
                                    .ok(syn::parse2(value.to_token_stream()))
                                    .map(Box::new),
                            };
                        }
                    } else {
                        errors.err(
                            &meta,
                            concat!(
                                "Invalid field-level `venndb` attribute\n",
                                "Expected one of: `key`, `filter`, `any`, `set`, ",
                                "`members`, `item`, `skip`",
                            ),
                        );
                    }
                }
            }
        }

        this.option_ty = ty_inner(&["Option"], &field.ty);

        if skipped {
            this.kind = None;
        } else if let Some(with) = key {
            if set.is_some() {
                errors.err(
                    &field.ty,
                    concat!(
                        "Invalid field-level `venndb` attribute\n",
                        "Cannot have both `key` and `set`",
                    ),
                );
            } else if this.option_ty.is_some() {
                errors.err(
                    &field.ty,
                    concat!(
                        "Invalid field-level `venndb` attribute\n",
                        "`key` fields cannot be `Option`",
                    ),
                );
            } else {
                this.kind = Some(FieldKind::Key { with });
            }
        } else if is_bool(this.option_ty.unwrap_or(&field.ty)) {
            if any.is_some() {
                errors.err(
                    &field.ty,
                    concat!(
                        "Invalid field-level `venndb` attribute\n",
                        "`any` cannot be used with `bool`",
                    ),
                );
            } else if set.is_some() {
                errors.err(
                    &field.ty,
                    concat!(
                        "Invalid field-level `venndb` attribute\n",
                        "`set` cannot be used with `bool`",
                    ),
                );
            } else {
                this.kind = Some(FieldKind::Filter);
            }
        } else if is_filter {
            // bool filters are to be seen as regular filters, even when made explicitly so!
            if any.is_some() && set.is_some() {
                errors.err(
                    &field.ty,
                    concat!(
                        "Invalid field-level `venndb` attribute\n",
                        "`any` cannot be used with `set`",
                    ),
                );
            } else {
                this.kind = Some(FieldKind::FilterMap { any, set, item });
            }
        } else if any.is_some() {
            errors.err(
                &field.ty,
                concat!(
                    "Invalid field-level `venndb` attribute\n",
                    "`any` can only be used with `filter`",
                ),
            );
        } else if set.is_some() {
            errors.err(
                &field.ty,
                concat!(
                    "Invalid field-level `venndb` attribute\n",
                    "`set` can only be used with `filter`",
                ),
            );
        }

        this
    }
}

/// The path of a `name = path` attribute, `None` for just `name`.
///
/// Kept as an expression, as a path such as `<[T]>::iter` starts with a type.
fn path_value(errors: &Errors, meta: &syn::Meta) -> Option<syn::ExprPath> {
    if let syn::Meta::Path(_) = meta {
        return None;
    }
    match &errors.expect_meta_name_value(meta)?.value {
        syn::Expr::Path(path) => Some(path.clone()),
        value => {
            errors.expect_path(value);
            None
        }
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
    pub validator: Option<syn::Path>,
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
                } else if name.is_ident("validator") {
                    if let Some(m) = errors.expect_meta_name_value(&meta) {
                        this.validator = errors.expect_path(&m.value).cloned();
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

/// Returns `Some(T)` if a type is `Wrapper<T>`, whatever the wrapper.
pub fn ty_inner_any(ty: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(path) = ty
        && path.qself.is_none()
        && let syn::PathArguments::AngleBracketed(args) = &path.path.segments.last()?.arguments
        && let Some(syn::GenericArgument::Type(ty)) = args.args.first()
    {
        return Some(ty);
    }
    None
}

/// Returns `Some(T)` if a type is `wrapper_name<T>` for any `wrapper_name` in `wrapper_names`.
fn ty_inner<'a>(wrapper_names: &[&str], ty: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(path) = ty {
        if path.qself.is_some() {
            return None;
        }
        // Since we only check the last path segment, it isn't necessarily the case that
        // we're referring to `std::vec::Vec` or `std::option::Option`, but there isn't
        // a fool proof way to check these since name resolution happens after macro expansion,
        // so this is likely "good enough" (so long as people don't have their own types called
        // `Option` or `Vec` that take one generic parameter they're looking to parse).
        let last_segment = path.path.segments.last()?;
        if !wrapper_names.iter().any(|name| last_segment.ident == *name) {
            return None;
        }
        if let syn::PathArguments::AngleBracketed(gen_args) = &last_segment.arguments {
            let generic_arg = gen_args.args.first()?;
            if let syn::GenericArgument::Type(ty) = &generic_arg {
                return Some(ty);
            }
        }
    }
    None
}
