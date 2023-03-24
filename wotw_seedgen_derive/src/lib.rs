#![allow(clippy::too_many_lines)]
#![allow(clippy::module_name_repetitions)]

extern crate proc_macro;

mod display;
mod from_str;
mod v;

use proc_macro::TokenStream;

#[proc_macro_derive(FromStr, attributes(ParseFromIdentifier, Ident))]
pub fn from_str_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);
    from_str::from_str_impl(ast)
}

#[proc_macro_derive(Display)]
pub fn display_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);
    display::display_impl(ast)
}

#[proc_macro_derive(VVariant, attributes(VWrap, VType))]
pub fn v_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);
    v::v_impl(ast)
}
