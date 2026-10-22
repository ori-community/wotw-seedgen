use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnekCase, ToSnekCase,
    ToTitleCase, ToTrainCase,
};
use itertools::Itertools;
use quote::ToTokens;
use syn::{parse::Parse, punctuated::Punctuated};

use crate::Result;

pub fn add_bound(generics: &mut syn::Generics, bound: syn::TypeParamBound) {
    for type_param in generics.type_params_mut() {
        type_param.bounds.push(bound.clone());
    }
}

pub fn find_attributes<T: Parse>(attrs: &[syn::Attribute], ident: &str) -> Result<Vec<T>> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident(ident))
        .map(|attr| attr.parse_args_with(Punctuated::<T, syn::Token![,]>::parse_terminated))
        .flatten_ok()
        .collect()
}

pub fn require_str(expr: syn::Expr) -> Result<syn::LitStr> {
    match expr {
        syn::Expr::Lit(syn::ExprLit {
            lit: syn::Lit::Str(str),
            ..
        }) => Ok(str),
        _ => Err(syn::Error::new_spanned(expr, "Expected string literal")),
    }
}

pub fn unrecognized_attr(tokens: impl ToTokens) -> syn::Error {
    syn::Error::new_spanned(tokens, "Unrecognized attribute")
}

#[derive(Clone, Copy)]
pub struct Case(for<'a> fn(&'a str) -> String);

impl Case {
    pub fn from_expr(expr: syn::Expr) -> Result<Self> {
        let str = require_str(expr)?;

        let f = match str.value().as_str() {
            "camelCase" => str::to_lower_camel_case,
            "PascalCase" => str::to_pascal_case,
            "kebab-case" => str::to_kebab_case,
            "snake_case" => str::to_snek_case,
            "SCREAMING_SNAKE_CASE" => str::TO_SHOUTY_SNEK_CASE,
            "SCREAMING-KEBAB-CASE" => str::to_shouty_kebab_case,
            "lowercase" => str::to_lowercase,
            "UPPERCASE" => str::to_uppercase,
            "title_case" => str::to_title_case,
            "mixed_case" => str::to_lower_camel_case,
            "Train-Case" => str::to_train_case,
            _ => return Err(syn::Error::new_spanned(str, "Unsupported case")),
        };

        Ok(Self(f))
    }

    pub fn apply(self, s: &str) -> String {
        self.0(s)
    }
}
