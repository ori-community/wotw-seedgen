use crate::{
    helpers::{find_attributes, require_str, unrecognized_attr, Case},
    Result,
};
use proc_macro2::TokenStream;
use quote::quote;
use syn::punctuated::Punctuated;

pub fn names_impl(input: syn::ItemEnum) -> Result<proc_macro::TokenStream> {
    let syn::ItemEnum {
        attrs,
        ident,
        variants,
        ..
    } = &input;

    let mut impls = TokenStream::default();

    for meta in find_attributes(&attrs, "names")? {
        let syn::Meta::List(meta) = meta else {
            return Err(syn::Error::new_spanned(
                meta,
                "Expected '<name>(<attrs>...)'",
            ));
        };

        let name = meta.path.require_ident()?;

        let mut case = None;
        let mut suffix = String::new();

        for inner in meta
            .parse_args_with(Punctuated::<syn::MetaNameValue, syn::Token![,]>::parse_terminated)?
        {
            let syn::MetaNameValue { path, value, .. } = inner;

            match path.require_ident()?.to_string().as_str() {
                "case" => case = Some(Case::from_expr(value)?),
                "suffix" => suffix = require_str(value)?.value(),
                _ => return Err(unrecognized_attr(value)),
            }
        }

        let Some(case) = case else {
            return Err(syn::Error::new_spanned(
                meta,
                "missing required case attribute",
            ));
        };

        let variants = variants
            .iter()
            .map(|variant| {
                let syn::Variant { ident, fields, .. } = variant;

                if fields.is_empty() {
                    let mut name = case.apply(&ident.to_string());

                    name.push_str(&suffix);

                    Ok(quote! {
                        Self::#ident => #name,
                    })
                } else {
                    Err(syn::Error::new_spanned(
                        fields,
                        format!("{name} can only be derived for Unit variants"),
                    ))
                }
            })
            .collect::<Result<Vec<_>>>()?;

        impls.extend(quote! {
            #[automatically_derived]
            impl #ident {
                pub fn #name(self) -> &'static str {
                    match self {
                        #(#variants)*
                    }
                }
            }
        });
    }

    if impls.is_empty() {
        return Err(syn::Error::new_spanned(
            input,
            "no names were specified in names attribute",
        ));
    }

    Ok(impls.into())
}
