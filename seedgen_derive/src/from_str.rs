use proc_macro::TokenStream;
use quote::quote;
use syn;

pub fn from_str_impl(input: syn::DeriveInput) -> TokenStream {
    let name = &input.ident;

    let mut parse_from_ident = false;
    let mut repr_ident = None;
    for attribute in input.attrs {
        if let Ok(meta) = attribute.parse_meta() {
            match meta {
                syn::Meta::Path(path) => {
                    if path.is_ident("ParseFromIdentifier") {
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
        let variants = match input.data {
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

    quote! {
        impl std::str::FromStr for #name {
            #implementation
        }
    }.into()
}
