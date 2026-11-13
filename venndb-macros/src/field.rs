//! Struct Field Info

use crate::{
    errors::Errors,
    parse_attrs::{By, FieldAttrs, FieldKind, ty_inner_any},
};
use quote::format_ident;
use syn::Ident;

/// A field of a `#![derive(VennDB)]` struct with attributes and some other
/// notable metadata appended.
pub struct StructField<'a> {
    /// The original parsed field
    field: &'a syn::Field,
    /// The parsed attributes of the field
    attrs: FieldAttrs<'a>,
    /// The field name. This is contained optionally inside `field`,
    /// but is duplicated non-optionally here to indicate that all field that
    /// have reached this point must have a field name, and it no longer
    /// needs to be unwrapped.
    name: &'a syn::Ident,
}

pub enum FieldInfo<'a> {
    Key(KeyField<'a>),
    Filter(FilterField<'a>),
    FilterMap(FilterMapField<'a>),
}

pub struct KeyField<'a> {
    pub name: &'a Ident,
    pub ty: &'a syn::Type,
    /// The function computing the key from the row, if not the field itself.
    pub with: Option<&'a syn::Path>,
}

impl KeyField<'_> {
    pub fn method_name(&self) -> Ident {
        format_ident!("get_by_{}", self.name)
    }

    pub fn index_method_name(&self) -> Ident {
        format_ident!("index_by_{}", self.name)
    }

    pub fn column_name(&self) -> Ident {
        format_ident!("key_{}", self.name)
    }
}

pub struct FilterField<'a> {
    pub name: &'a Ident,
    pub optional: bool,
}

impl FilterField<'_> {
    pub fn column_name(&self) -> Ident {
        format_ident!("filter_{}", self.name)
    }
}

pub struct FilterMapField<'a> {
    pub name: &'a Ident,
    /// The type of the field, without `Option`.
    pub ty: &'a syn::Type,
    pub optional: bool,
    pub any: Option<&'a By>,
    pub set: Option<&'a By>,
    /// The type of the values of a set, if it has to be told.
    pub item: Option<&'a syn::Type>,
}

impl FilterMapField<'_> {
    pub fn column_name(&self) -> Ident {
        format_ident!("filter_{}", self.name)
    }

    pub fn where_method_name(&self) -> Ident {
        format_ident!("{}_where", self.name)
    }
}

impl<'a> StructField<'a> {
    /// Attempts to parse a field of a `#[derive(VennDB)]` struct, pulling out the
    /// fields required for code generation.
    pub fn new(errors: &Errors, field: &'a syn::Field, attrs: FieldAttrs<'a>) -> Option<Self> {
        let name = field.ident.as_ref().expect("missing ident for named field");
        if let Some(FieldKind::FilterMap {
            set: Some(By::Path(_)),
            item: None,
            ..
        }) = &attrs.kind
            && ty_inner_any(attrs.option_ty.unwrap_or(&field.ty)).is_none()
        {
            errors.err(
                &field.ty,
                concat!(
                    "Invalid field-level `venndb` attribute\n",
                    "`members` needs `item = Type` to know the type of the values",
                ),
            );
            return None;
        }
        Some(StructField { field, attrs, name })
    }

    /// Return the method name for this struct field.
    pub fn info(&self) -> Option<FieldInfo<'_>> {
        let ty = self.attrs.option_ty.unwrap_or(&self.field.ty);
        let optional = self.attrs.option_ty.is_some();
        self.attrs.kind.as_ref().map(|kind| match kind {
            FieldKind::Key { with } => FieldInfo::Key(KeyField {
                name: self.name,
                ty,
                with: with.as_ref(),
            }),
            FieldKind::Filter => FieldInfo::Filter(FilterField {
                name: self.name,
                optional,
            }),
            FieldKind::FilterMap { any, set, item } => FieldInfo::FilterMap(FilterMapField {
                name: self.name,
                ty,
                optional,
                any: any.as_ref(),
                set: set.as_ref(),
                item: item.as_deref().or_else(|| ty_inner_any(ty)),
            }),
        })
    }
}
