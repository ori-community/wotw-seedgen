use crate::Result;
use itertools::Itertools;
use quote::quote;

const ONLY_UNIT_ENUMS_MESSAGE: &str = "Can only derive FromStr on enums with unit variants";

pub fn from_str_impl(input: syn::DeriveInput) -> Result<proc_macro::TokenStream> {
    let syn::DeriveInput { data, ident, .. } = &input;

    let syn::Data::Enum(data) = &data else {
        return Err(syn::Error::new_spanned(input, ONLY_UNIT_ENUMS_MESSAGE));
    };

    let variants = data
        .variants
        .iter()
        .map(|variant| {
            let syn::Variant {
                ident: variant_ident,
                fields,
                ..
            } = variant;

            if !fields.is_empty() {
                return Err(syn::Error::new_spanned(variant, ONLY_UNIT_ENUMS_MESSAGE));
            }

            let variant_string = variant_ident.to_string();
            Ok(quote! {
                #variant_string => Ok(#ident::#variant_ident),
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let error = format!(
        "Expected {}",
        data.variants
            .iter()
            .format_with(" or ", |variant, f| f(&format_args!("{}", variant.ident)))
    );

    Ok(quote! {
        #[automatically_derived]
        impl ::std::str::FromStr for #ident {
            type Err = String;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                match s {
                    #(#variants)*
                    _ => Err(#error.to_string())
                }
            }
        }
    }
    .into())
}
