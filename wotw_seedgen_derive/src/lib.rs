#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

mod ast;
mod span;
mod token_display;

use itertools::Itertools;
use proc_macro::TokenStream;
use syn::{parse::Parse, punctuated::Punctuated};

type Result<T> = std::result::Result<T, syn::Error>;

fn add_bound(generics: &mut syn::Generics, bound: proc_macro2::TokenStream) {
    let bound = syn::parse2::<syn::TypeParamBound>(bound).unwrap();

    for type_param in generics.type_params_mut() {
        type_param.bounds.push(bound.clone());
    }
}

fn find_attributes<T: Parse>(attrs: &[syn::Attribute], ident: &str) -> Result<Vec<T>> {
    attrs
        .iter()
        .filter(|attr| attr.path().is_ident(ident))
        .map(|attr| attr.parse_args_with(Punctuated::<T, syn::Token![,]>::parse_terminated))
        .flatten_ok()
        .collect()
}

// TODO this is specifically for logos
#[proc_macro_derive(TokenDisplay, attributes(token))]
pub fn token_display_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemEnum);
    token_display::token_display_impl(ast).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_derive(Ast, attributes(ast))]
pub fn ast_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    ast::ast_impl(ast).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_derive(Span)]
pub fn span_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    span::span_impl(ast).unwrap_or_else(|err| err.into_compile_error().into())
}
