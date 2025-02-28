use crate::{find_attributes, Result};
use itertools::Itertools;
use quote::quote;

pub fn token_display_impl(input: syn::ItemEnum) -> Result<proc_macro::TokenStream> {
    let ident = input.ident;

    let variants = input
        .variants
        .into_iter()
        .map(|variant| {
            let syn::Variant {
                attrs,
                ident,
                fields,
                ..
            } = variant;

            let token_attrs = find_attributes(&attrs, "token")?
                .into_iter()
                .filter_map(|expr| match expr {
                    syn::Expr::Lit(syn::ExprLit {
                        lit: syn::Lit::Str(lit),
                        ..
                    }) => Some(lit.token()),
                    _ => None,
                })
                .join(" or ");

            let display = if token_attrs.is_empty() {
                ident.to_string()
            } else {
                token_attrs
            };
            Ok(quote! {
                Self::#ident #fields => write!(f, #display)
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        #[automatically_derived]
        impl std::fmt::Display for #ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                match self {
                    #(#variants),*
                }
            }
        }
    }
    .into())
}
