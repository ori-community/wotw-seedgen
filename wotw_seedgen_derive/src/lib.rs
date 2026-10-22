#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

mod ast;
mod from_str;
mod helpers;
mod names;
mod span;
mod token_display;

use proc_macro::TokenStream;

type Result<T> = std::result::Result<T, syn::Error>;

// TODO this is specifically for logos
#[proc_macro_derive(TokenDisplay, attributes(token))]
pub fn token_display_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemEnum);
    token_display::token_display_impl(ast).unwrap_or_else(|err| err.into_compile_error().into())
}

#[proc_macro_derive(Names, attributes(names))]
pub fn name_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::ItemEnum);
    names::names_impl(ast).unwrap_or_else(|err| err.into_compile_error().into())
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

#[proc_macro_derive(FromStr)]
pub fn from_str_derive(input: TokenStream) -> TokenStream {
    let ast = syn::parse_macro_input!(input as syn::DeriveInput);
    from_str::from_str_impl(ast).unwrap_or_else(|err| err.into_compile_error().into())
}
