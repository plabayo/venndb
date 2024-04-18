#![allow(dead_code)]

use {
    proc_macro2::{Span, TokenStream},
    quote::ToTokens,
    std::cell::RefCell,
};

/// Produce functions to expect particular literals in `syn::Expr`
macro_rules! expect_lit_fn {
    ($(($fn_name:ident, $syn_type:ident, $variant:ident, $lit_name:literal),)*) => {
        $(
            pub fn $fn_name<'a>(&self, e: &'a syn::Expr) -> Option<&'a syn::$syn_type> {
                if let syn::Expr::Lit(syn::ExprLit { lit: syn::Lit::$variant(inner), .. }) = e {
                    Some(inner)
                } else {
                    self.unexpected_lit($lit_name, e);
                    None
                }
            }
        )*
    }
}

/// Produce functions to expect particular variants of `syn::Meta`
macro_rules! expect_meta_fn {
    ($(($fn_name:ident, $syn_type:ident, $variant:ident, $meta_name:literal),)*) => {
        $(
            pub fn $fn_name<'a>(&self, meta: &'a syn::Meta) -> Option<&'a syn::$syn_type> {
                if let syn::Meta::$variant(inner) = meta {
                    Some(inner)
                } else {
                    self.unexpected_meta($meta_name, meta);
                    None
                }
            }
        )*
    }
}

/// A type for collecting procedural macro errors.
#[derive(Default)]
pub struct Errors {
    errors: RefCell<Vec<syn::Error>>,
}

impl Errors {
    expect_lit_fn![
        (expect_lit_str, LitStr, Str, "string"),
        (expect_lit_char, LitChar, Char, "character"),
        (expect_lit_int, LitInt, Int, "integer"),
    ];

    expect_meta_fn![
        (expect_meta_word, Path, Path, "path"),
        (expect_meta_list, MetaList, List, "list"),
        (
            expect_meta_name_value,
            MetaNameValue,
            NameValue,
            "name-value pair"
        ),
    ];

    pub fn expect_path<'a>(&self, e: &'a syn::Expr) -> Option<&'a syn::Path> {
        if let syn::Expr::Path(path) = e {
            Some(&path.path)
        } else {
            self.unexpected_value("path", e);
            None
        }
    }

    fn unexpected_lit(&self, expected: &str, found: &syn::Expr) {
        fn lit_kind(lit: &syn::Lit) -> &'static str {
            use syn::Lit::{Bool, Byte, ByteStr, Char, Float, Int, Str, Verbatim};
            match lit {
                Str(_) => "string",
                ByteStr(_) => "bytestring",
                Byte(_) => "byte",
                Char(_) => "character",
                Int(_) => "integer",
                Float(_) => "float",
                Bool(_) => "boolean",
                Verbatim(_) => "unknown (possibly extra-large integer)",
                _ => "unknown literal kind",
            }
        }

        if let syn::Expr::Lit(syn::ExprLit { lit, .. }) = found {
            self.err(
                found,
                &[
                    "Expected ",
                    expected,
                    " literal, found ",
                    lit_kind(lit),
                    " literal",
                ]
                .concat(),
            )
        } else {
            self.err(
                found,
                &[
                    "Expected ",
                    expected,
                    " literal, found non-literal expression.",
                ]
                .concat(),
            )
        }
    }

    fn unexpected_meta(&self, expected: &str, found: &syn::Meta) {
        fn meta_kind(meta: &syn::Meta) -> &'static str {
            use syn::Meta::{List, NameValue, Path};
            match meta {
                Path(_) => "path",
                List(_) => "list",
                NameValue(_) => "name-value pair",
            }
        }

        self.err(
            found,
            &[
                "Expected ",
                expected,
                " attribute, found ",
                meta_kind(found),
                " attribute",
            ]
            .concat(),
        )
    }

    fn unexpected_value(&self, expected: &str, found: &syn::Expr) {
        fn expr_kind(expr: &syn::Expr) -> &'static str {
            use syn::Expr::{
                Array, Assign, Async, Await, Binary, Block, Break, Call, Cast, Closure, Const,
                Continue, Field, ForLoop, Group, If, Index, Infer, Let, Lit, Loop, Macro, Match,
                MethodCall, Paren, Path, Range, Reference, Repeat, Return, Struct, Try, TryBlock,
                Tuple, Unary, Unsafe, Verbatim, While, Yield,
            };
            match expr {
                Array(_) => "array",
                Assign(_) => "assignment",
                Async(_) => "async block",
                Await(_) => "await",
                Binary(_) => "binary operation",
                Block(_) => "block",
                Break(_) => "break",
                Call(_) => "function call",
                Cast(_) => "cast",
                Closure(_) => "closure",
                Const(_) => "const",
                Continue(_) => "continue",
                Field(_) => "field access",
                ForLoop(_) => "for loop",
                Group(_) => "group",
                If(_) => "if",
                Index(_) => "index",
                Infer(_) => "inferred type",
                Let(_) => "let",
                Lit(_) => "literal",
                Loop(_) => "loop",
                Macro(_) => "macro",
                Match(_) => "match",
                MethodCall(_) => "method call",
                Paren(_) => "parentheses",
                Path(_) => "path",
                Range(_) => "range",
                Reference(_) => "reference",
                Repeat(_) => "repeat",
                Return(_) => "return",
                Struct(_) => "struct",
                Try(_) => "try",
                TryBlock(_) => "try block",
                Tuple(_) => "tuple",
                Unary(_) => "unary operation",
                Unsafe(_) => "unsafe block",
                Verbatim(_) => "verbatim",
                While(_) => "while",
                Yield(_) => "yield",
                _ => "unknown expression kind",
            }
        }

        self.err(
            found,
            &[
                "Expected ",
                expected,
                " attribute, found ",
                found.to_token_stream().to_string().as_str(),
                " attribute (",
                expr_kind(found),
                ")",
            ]
            .concat(),
        )
    }

    /// Issue an error relating to a particular `Spanned` structure.
    pub fn err(&self, spanned: &impl syn::spanned::Spanned, msg: &str) {
        self.err_span(spanned.span(), msg);
    }

    /// Issue an error relating to a particular `Span`.
    pub fn err_span(&self, span: Span, msg: &str) {
        self.push(syn::Error::new(span, msg));
    }

    /// Issue an error spanning over the given syntax tree node.
    pub fn err_span_tokens<T: ToTokens>(&self, tokens: T, msg: &str) {
        self.push(syn::Error::new_spanned(tokens, msg));
    }

    /// Push a `syn::Error` onto the list of errors to issue.
    pub fn push(&self, err: syn::Error) {
        self.errors.borrow_mut().push(err);
    }

    /// Convert a `syn::Result` to an `Option`, logging the error if present.
    pub fn ok<T>(&self, r: syn::Result<T>) -> Option<T> {
        match r {
            Ok(v) => Some(v),
            Err(e) => {
                self.push(e);
                None
            }
        }
    }
}

impl ToTokens for Errors {
    /// Convert the errors into tokens that, when emit, will cause
    /// the user of the macro to receive compiler errors.
    fn to_tokens(&self, tokens: &mut TokenStream) {
        tokens.extend(self.errors.borrow().iter().map(|e| e.to_compile_error()));
    }
}
