use std::iter::Map;

use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[derive(Default)]
struct VBehaviour {
    wrap: bool,
    auto_type: bool,
}

fn v_target(ty: &mut syn::Type) -> &mut syn::Type {
    if let syn::Type::Path(path) = ty {
        let last = path
            .path
            .segments
            .last_mut()
            .expect("Cannot use VType on empty Path");
        if last.ident != "Box" && last.ident != "Option" {
            return ty;
        }
    } else {
        panic!("VType attribute is only supported for Path types");
    };

    if let syn::Type::Path(path) = ty {
        let last = path
            .path
            .segments
            .last_mut()
            .expect("Cannot use VType on empty Path");
        if let syn::PathArguments::AngleBracketed(args) = &mut last.arguments {
            for arg in &mut args.args {
                if let syn::GenericArgument::Type(ty) = arg {
                    return v_target(ty);
                }
            }
            unreachable!();
        } else {
            panic!("Failed to create Wrapper types");
        }
    } else {
        unreachable!();
    }
}

type VBehaviourMap<I> =
    Map<I, fn(&mut syn::Field) -> (&syn::Field, Option<(VBehaviour, syn::Type)>)>;
fn v_fields<'a, I>(fields: I) -> VBehaviourMap<I>
where
    I: Iterator<Item = &'a mut syn::Field>,
{
    fields.map(|field| {
        let mut behaviour = VBehaviour::default();
        for index in field
            .attrs
            .iter()
            .enumerate()
            .filter_map(|(index, attribute)| {
                if let Ok(syn::Meta::Path(path)) = attribute.parse_meta() {
                    if path.is_ident("VWrap") {
                        behaviour.wrap = true;
                        return Some(index);
                    } else if path.is_ident("VType") {
                        behaviour.auto_type = true;
                        return Some(index);
                    }
                }
                None
            })
            .rev()
            .collect::<Vec<_>>()
        {
            field.attrs.remove(index);
        }

        if behaviour.wrap || behaviour.auto_type {
            let ty = &mut field.ty;
            let original_ty = ty.clone();

            let target_type = v_target(ty);
            if let syn::Type::Path(path) = target_type {
                let path = &mut path.path;
                let last_ident = &mut path
                    .segments
                    .last_mut()
                    .expect("Cannot use VType on empty Path")
                    .ident;
                let span = last_ident.span();
                if behaviour.auto_type {
                    *last_ident = format_ident!("V{last_ident}");
                }
                if behaviour.wrap {
                    let inner_path = path.clone();
                    *path = syn::Path {
                        leading_colon: None,
                        segments: [
                            syn::PathSegment {
                                ident: syn::Ident::new("crate", span),
                                arguments: syn::PathArguments::None,
                            },
                            syn::PathSegment {
                                ident: syn::Ident::new("header", span),
                                arguments: syn::PathArguments::None,
                            },
                            syn::PathSegment {
                                ident: syn::Ident::new("V", span),
                                arguments: syn::PathArguments::AngleBracketed(
                                    syn::AngleBracketedGenericArguments {
                                        colon2_token: None,
                                        lt_token: syn::Token![<](span),
                                        args: [syn::GenericArgument::Type(syn::Type::Path(
                                            syn::TypePath {
                                                qself: None,
                                                path: inner_path,
                                            },
                                        ))]
                                        .into_iter()
                                        .collect(),
                                        gt_token: syn::Token![>](span),
                                    },
                                ),
                            },
                        ]
                        .into_iter()
                        .collect(),
                    }
                }
            } else {
                panic!("VType attribute is only supported for Path types");
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
    input.attrs.clear();

    let implementation = match &mut input.data {
        syn::Data::Struct(data_struct) => {
            match &mut data_struct.fields {
                syn::Fields::Named(fields) => {
                    let fields = v_fields(fields.named.iter_mut()).map(|(field, behaviour)| {
                        let field_ident = field.ident.as_ref().unwrap();
                        match behaviour {
                            Some((_, original_ty)) => quote! {
                                #field_ident: {
                                    fn boundary(check: &impl crate::header::VResolve<#original_ty>) {}
                                    boundary(&self.#field_ident);
                                    self.#field_ident.resolve(parameters)?
                                }
                            },
                            None => quote! { #field_ident: self.#field_ident },
                        }
                    });

                    quote! { #ident { #(#fields),* } }
                }
                syn::Fields::Unnamed(fields) => {
                    let fields = v_fields(fields.unnamed.iter_mut()).enumerate().map(|(index, (_, behaviour))| {
                        let index = syn::Index::from(index);
                        match behaviour {
                            Some((_, original_ty)) => quote! { {
                                fn boundary(check: &impl crate::header::VResolve<#original_ty>) {}
                                boundary(&self.#index);
                                self.#index.resolve(parameters)?
                            } },
                            None => quote! { self.#index },
                        }
                    });

                    quote! { #ident(#(#fields),*) }
                }
                syn::Fields::Unit => panic!("Unit structs can't have meaningful VVariants"),
            }
        }
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
                                        fn boundary(check: &impl crate::header::VResolve<#original_ty>) {}
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
                                    fn boundary(check: &impl crate::header::VResolve<#original_ty>) {}
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
        }
        syn::Data::Union(_) => panic!("Expected Struct or Enum"),
    };

    quote! {
        #[doc = #doc]
        #[derive(Debug, Clone)]
        #input
        impl crate::header::VResolve<#ident> for #v_ident {
            #[doc = " resolve all parameters and parse the resulting values"]
            fn resolve(self, parameters: &::rustc_hash::FxHashMap<String, String>) -> Result<#ident, String> {
                let res = { #implementation };
                Ok(res)
            }
        }
    }.into()
}
