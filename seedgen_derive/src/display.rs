use std::iter;

use proc_macro::TokenStream;
use quote::quote;
use syn;

pub fn display_impl(input: syn::DeriveInput) -> TokenStream {
    let name = input.ident;

    let variants = match input.data {
        syn::Data::Enum(data_enum) => {
            data_enum.variants.into_iter().map(|variant| {
                let identifier = variant.ident;
                let mut display = identifier.to_string();

                let indices = display.match_indices(char::is_uppercase)
                    .filter_map(|(index, _)| if index > 0 { Some(index) } else { None })
                    .rev().collect::<Vec<_>>();
                for index in indices {
                    display.insert(index, ' ');
                }

                let fields = match variant.fields {
                    syn::Fields::Named(_) => return quote! { #name::#identifier { .. } => #display },
                    syn::Fields::Unnamed(fields) => fields.unnamed,
                    syn::Fields::Unit => return quote! { #name::#identifier => #display },
                };
                let eat_fields = iter::repeat("_")
                    .take(fields.len())
                    .collect::<Vec<_>>();
                quote! {
                    #name::#identifier(#(#eat_fields),*) => #display
                }
            }).collect::<Vec<_>>()
        },
        _ => panic!("Expected enum"),
    };

    quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let display = match self {
                    #(#variants),*
                };
                write!(f, "{}", display)
            }
        }
    }.into()
}
