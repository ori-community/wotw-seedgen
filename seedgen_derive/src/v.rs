use std::iter::Map;

use proc_macro::TokenStream;
use quote::{quote, format_ident};
use syn;

#[derive(PartialEq)]
enum VBehaviour {
    Wrap,
    AutoType,
}

fn v_fields<'a, I: Iterator<Item = &'a mut syn::Field>>(fields: I) -> Map<I, fn(&mut syn::Field) -> (&syn::Field, Option<(VBehaviour, syn::Type)>)> {
    fields.map(|field| {
        if let Some((index, behaviour)) = field.attrs.iter().enumerate().find_map(|(index, attribute)| {
            if let Ok(meta) = attribute.parse_meta() {
                match meta {
                    syn::Meta::Path(path) => {
                        if path.is_ident("VWrap") {
                            return Some((index, VBehaviour::Wrap));
                        } else if path.is_ident("VType") {
                            return Some((index, VBehaviour::AutoType));
                        }
                    },
                    _ => {},
                }
            }
            None
        }) {
            field.attrs.remove(index);
            let ty = &mut field.ty;
            let original_ty = ty.clone();

            let target_path = {
                if let syn::Type::Path(path) = ty {
                    let last = path.path.segments.last_mut().expect("Cannot use VType on empty Path");
                    if last.ident == "Box" || last.ident == "Option" {
                        if let syn::PathArguments::AngleBracketed(args) = &mut last.arguments {
                            args.args.iter_mut().find_map(|arg| {
                                if let syn::GenericArgument::Type(ty) = arg {
                                    if let syn::Type::Path(path) = ty {
                                        Some(path.path.segments.last_mut().expect("Cannot use VType on empty Path"))
                                    } else {
                                        panic!("VType attribute is only supported for Path types");
                                    }
                                } else { None }
                            }).unwrap()
                        } else {
                            panic!("Failed to create Wrapper types");
                        }
                    } else {
                        last
                    }
                } else {
                    panic!("VType attribute is only supported for Path types");
                }
            };

            match behaviour {
                VBehaviour::Wrap => *target_path = syn::PathSegment {
                    ident: syn::Ident::new("V", target_path.ident.span()),
                    arguments: syn::PathArguments::AngleBracketed(syn::AngleBracketedGenericArguments {
                        colon2_token: None,
                        lt_token: syn::Token![<](target_path.ident.span()),
                        args: syn::punctuated::Punctuated::from_iter([syn::GenericArgument::Type(syn::Type::Path(syn::TypePath {
                            qself: None,
                            path: syn::Path {
                                leading_colon: None,
                                segments: syn::punctuated::Punctuated::from_iter([target_path.clone()].into_iter()),
                            },
                        }))].into_iter()),
                        gt_token: syn::Token![>](target_path.ident.span()),
                    })
                },
                VBehaviour::AutoType => target_path.ident = format_ident!("V{}", target_path.ident),
            }

            (field, Some((behaviour, original_ty)))
        } else {
            (field, None)
        }
    })
}

pub fn v_impl(mut input: syn::DeriveInput) -> TokenStream {
    let ident = input.ident;
    let doc = format!(" [`{ident}`] with possibly contained [`V`]s");

    let v_ident = format_ident!("V{ident}");
    input.ident = v_ident.clone();

    let implementation = match &mut input.data {
        syn::Data::Struct(data_struct) => {
            match &mut data_struct.fields {
                syn::Fields::Named(fields) => {
                    let fields = v_fields(fields.named.iter_mut()).map(|(field, behaviour)| {
                        let field_ident = field.ident.as_ref().unwrap();
                        match behaviour {
                            Some((_, original_ty)) => quote! {
                                #field_ident: {
                                    fn boundary(check: &impl VResolve<#original_ty>) {}
                                    boundary(&self.#field_ident);
                                    self.#field_ident.resolve(parameters)?
                                }
                            },
                            None => quote! { #field_ident: self.#field_ident },
                        }
                    });

                    quote! { #ident { #(#fields),* } }
                },
                syn::Fields::Unnamed(fields) => {
                    let fields = v_fields(fields.unnamed.iter_mut()).enumerate().map(|(index, (_, behaviour))| {
                        let index = syn::Index::from(index);
                        match behaviour {
                            Some((_, original_ty)) => quote! { {
                                fn boundary(check: &impl VResolve<#original_ty>) {}
                                boundary(&self.#index);
                                self.#index.resolve(parameters)?
                            } },
                            None => quote! { self.#index },
                        }
                    });

                    quote! { #ident(#(#fields),*) }
                },
                syn::Fields::Unit => panic!("Unit structs can't have meaningful VVariants"),
            }
        },
        syn::Data::Enum(data_enum) => {
            let branches = data_enum.variants.iter_mut().map(|variant| {
                let variant_ident = &variant.ident;

                match &mut variant.fields {
                    syn::Fields::Named(fields) => {
                        let (fields_left, fields_right): (Vec<_>, Vec<_>) = v_fields(fields.named.iter_mut()).map(|(field, behaviour)| {
                            let field_ident = field.ident.as_ref().unwrap();
                            match behaviour {
                                Some((_, original_ty)) => (quote! { #field_ident }, quote! {
                                    #field_ident: {
                                        fn boundary(check: &impl VResolve<#original_ty>) {}
                                        boundary(&#field_ident);
                                        #field_ident.resolve(parameters)?
                                    }
                                }),
                                None => (quote! { #field_ident }, quote! { #field_ident: #field_ident }),
                            }
                        }).unzip();

                        quote! { #v_ident::#variant_ident { #(#fields_left),* } => #ident::#variant_ident { #(#fields_right),* } }
                    },
                    syn::Fields::Unnamed(fields) => {
                        let (fields_left, fields_right): (Vec<_>, Vec<_>) = v_fields(fields.unnamed.iter_mut()).enumerate().map(|(index, (_, behaviour))| {
                            let indexed_ident = format_ident!("field_{index}");
                            match behaviour {
                                Some((_, original_ty)) => (quote! { #indexed_ident }, quote! { {
                                    fn boundary(check: &impl VResolve<#original_ty>) {}
                                    boundary(&#indexed_ident);
                                    #indexed_ident.resolve(parameters)?
                                } }),
                                None => (quote! { #indexed_ident }, quote! { #indexed_ident }),
                            }
                        }).unzip();

                        quote! {
                            #v_ident::#variant_ident(#(#fields_left),*) => #ident::#variant_ident(#(#fields_right),*)
                        }
                    },
                    syn::Fields::Unit => quote! {
                        #v_ident::#variant_ident => #ident::#variant_ident
                    },
                }
            });

            quote! {
                match self {
                    #(#branches),*
                }
            }
        },
        _ => panic!("Expected Struct or Enum"),
    };

    quote! {
        #[doc = #doc]
        #[derive(Debug, Clone)]
        #input
        impl VResolve<#ident> for #v_ident {
            #[doc = " resolve all parameters and parse the resulting values"]
            fn resolve(self, parameters: &::rustc_hash::FxHashMap<::std::string::String, ::std::string::String>) -> ::core::result::Result<#ident, String> {
                let res = { #implementation };
                Ok(res)
            }
        }
    }.into()
}
