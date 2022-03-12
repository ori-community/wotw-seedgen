extern crate proc_macro;

use std::iter;

use proc_macro::TokenStream;
use quote::quote;
use syn;

#[proc_macro_derive(FromStr, attributes(ParseFromIdentifier))]
pub fn from_str_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);

    let name = &ast.ident;

    let mut parse_from_ident = false;
    let mut repr_ident = None;
    for attribute in ast.attrs {
        if let Ok(meta) = attribute.parse_meta() {
            match meta {
                syn::Meta::Path(path) => {
                    if path.get_ident().map_or(false, |ident| ident == "ParseFromIdentifier") {
                        parse_from_ident = true;
                    }
                },
                syn::Meta::List(list) => {
                    if list.path.get_ident().map_or(false, |ident| ident == "repr") {
                        if list.nested.len() != 1 || repr_ident.is_some() {
                            panic!("Expected exactly one repr argument");
                        }
                        let repr = list.nested.into_iter().next().unwrap();
                        if let syn::NestedMeta::Meta(syn::Meta::Path(repr)) = repr {
                            repr_ident = Some(repr.get_ident().expect("repr identifier").clone());
                        } else {
                            panic!("Invalid repr attribute")
                        }
                    }
                },
                _ => {},
            }
        }
    }

    let implementation = if parse_from_ident {
        let variants = match ast.data {
            syn::Data::Enum(data_enum) => {
                data_enum.variants.into_iter().map(|variant| {
                    if !matches!(variant.fields, syn::Fields::Unit) {
                        panic!("Expected unit variant");
                    }

                    let variant = variant.ident;
                    let variant_string = variant.to_string().to_lowercase();

                    quote! {
                        #variant_string => #name::#variant
                    }
                }).collect::<Vec<_>>()
            },
            _ => panic!("Expected enum"),
        };

        let name_string = name.to_string();

        quote! {
            type Err = String;
            fn from_str(string: &str) -> Result<#name, String> {
                let variant = match &string.to_lowercase()[..] {
                    #(#variants),*,
                    _ => return Err(format!("Unknown {} {}", #name_string, string))
                };
                Ok(variant)
            }
        }
    } else {
        let repr_ident = repr_ident.expect("Missing repr or ParseFromIdentifier attribute");
        quote! {
            type Err = Box<dyn std::error::Error>;
            fn from_str(string: &str) -> Result<#name, Box<dyn std::error::Error>> {
                use std::convert::TryFrom;
                let number = string.parse::<#repr_ident>()?;
                let variant = #name::try_from(number)?;
                Ok(variant)
            }
        }
    };

    let gen = quote! {
        impl std::str::FromStr for #name {
            #implementation
        }
    };
    gen.into()
}

#[proc_macro_derive(Display)]
pub fn display_derive(input: TokenStream) -> TokenStream {
    let ast: syn::DeriveInput = syn::parse_macro_input!(input);

    let name = ast.ident;

    let variants = match ast.data {
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
                    syn::Fields::Named(fields) => fields.named,
                    syn::Fields::Unnamed(fields) => fields.unnamed,
                    syn::Fields::Unit => return quote! { #name::#identifier => #display },
                };
                let field_storage = fields.iter().enumerate()
                    .map(|(index, _)| syn::Ident::new(&format!("field_{}", index), quote::__private::Span::call_site()))
                    .collect::<Vec<_>>();
                let format_literal = format!("{} ({})", display, iter::repeat("{}").take(field_storage.len()).collect::<Vec<_>>().join(", "));
                quote! {
                    #name::#identifier(#(#field_storage),*) => return write!(f, #format_literal, #(#field_storage),*)
                }
            }).collect::<Vec<_>>()
        },
        _ => panic!("Expected enum"),
    };

    let gen = quote! {
        impl std::fmt::Display for #name {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                let display = match self {
                    #(#variants),*
                };
                write!(f, "{}", display)
            }
        }
    };

    gen.into()
}
