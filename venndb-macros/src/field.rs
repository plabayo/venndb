//! Struct Field Info

use crate::{
    errors::Errors,
    parse_attrs::{FieldAttrs, FieldKind},
};
use quote::format_ident;
use syn::Ident;

/// A field of a `#![derive(VennDB)]` struct with attributes and some other
/// notable metadata appended.
pub struct StructField<'a> {
    /// The original parsed field
    field: &'a syn::Field,
    /// The parsed attributes of the field
    attrs: FieldAttrs,
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
}

impl<'a> KeyField<'a> {
    pub fn name(&'a self) -> &'a Ident {
        self.name
    }

    pub fn ty(&'a self) -> &'a syn::Type {
        self.ty
    }

    pub fn method_name(&self) -> Ident {
        format_ident!("get_by_{}", self.name)
    }

    pub fn map_name(&self) -> Ident {
        format_ident!("map_{}", self.name)
    }
}

pub struct FilterField<'a> {
    pub name: &'a Ident,
}

impl<'a> FilterField<'a> {
    pub fn name(&'a self) -> &'a Ident {
        self.name
    }

    pub fn filter_name(&self) -> Ident {
        format_ident!("filter_{}", self.name)
    }

    pub fn filter_not_name(&self) -> Ident {
        format_ident!("filter_not_{}", self.name)
    }
}

impl<'a> StructField<'a> {
    /// Attempts to parse a field of a `#[derive(VennDB)]` struct, pulling out the
    /// fields required for code generation.
    pub fn new(_errors: &Errors, field: &'a syn::Field, attrs: FieldAttrs) -> Option<Self> {
        let name = field.ident.as_ref().expect("missing ident for named field");
        Some(StructField { field, attrs, name })
    }

    /// Return the method name for this struct field.
    pub fn info(&self) -> Option<FieldInfo> {
        self.attrs.kind.as_ref().map(|kind| match kind {
            FieldKind::Key => FieldInfo::Key(KeyField {
                name: self.name,
                ty: &self.field.ty,
            }),
            FieldKind::Filter => FieldInfo::Filter(FilterField { name: self.name }),
            FieldKind::FilterMap => FieldInfo::FilterMap(FilterMapField {
                name: self.name,
                ty: &self.field.ty,
            }),
        })
    }
}

pub struct FilterMapField<'a> {
    pub name: &'a Ident,
    pub ty: &'a syn::Type,
}

impl<'a> FilterMapField<'a> {
    pub fn name(&'a self) -> &'a Ident {
        self.name
    }

    pub fn ty(&'a self) -> &'a syn::Type {
        self.ty
    }

    pub fn filter_map_name(&self) -> Ident {
        format_ident!("filter_map_{}", self.name)
    }

    pub fn filter_vec_name(&self) -> Ident {
        format_ident!("filter_vec_{}", self.name)
    }
}
