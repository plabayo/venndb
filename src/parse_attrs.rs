use crate::errors::Errors;

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
    if !is_argh_attr(attr) {
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
fn is_argh_attr(attr: &syn::Attribute) -> bool {
    is_matching_attr("venndb", attr)
}