use crate::{add_bound, find_attributes, Result};
use proc_macro2::TokenStream;
use quote::{quote, ToTokens};
use syn::spanned::Spanned;

pub fn span_impl(input: syn::DeriveInput) -> Result<proc_macro::TokenStream> {
    let syn::DeriveInput {
        ident,
        mut generics,
        data,
        attrs,
        ..
    } = input;
    let attrs = Attributes::find(&attrs)?;

    let implementation = match data {
        syn::Data::Struct(data) => span_fields::<false>(data.fields),
        syn::Data::Enum(data) => span_enum(data),
        syn::Data::Union(union) => Err(syn::Error::new(
            union.union_token.span(),
            "Cannot implement Span on union",
        )),
    }?;

    add_bound(&mut generics, quote! { wotw_seedgen_parse::Span });
    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    // TODO instead of providing this bound attribute we could also be a lot smarter about it by looking at the type
    let bound = attrs.bound.map_or_else(
        || impl_generics.into_token_stream(),
        |bound| quote! { < #bound > },
    );

    Ok(quote! {
        impl #bound wotw_seedgen_parse::Span for #ident #type_generics #where_clause {
            fn span(&self) -> std::ops::Range<usize> {
                #implementation
            }
        }
    }
    .into())
}

#[derive(Default)]
struct Attributes {
    bound: Option<syn::Expr>,
}
impl Attributes {
    fn find(attrs: &[syn::Attribute]) -> Result<Self> {
        let mut attributes = Self::default();

        for meta in find_attributes(attrs, "span")? {
            match meta {
                syn::Meta::NameValue(meta) if meta.path.is_ident("bound") => {
                    attributes.bound = Some(meta.value);
                }
                _ => return Err(syn::Error::new_spanned(meta, "Unrecognized attribute")),
            }
        }

        Ok(attributes)
    }
}

const EMPTY_FIELDS_MESSAGE: &str = "Cannot implement Span without any fields";

fn span_fields<const IN_PATTERN: bool>(fields: syn::Fields) -> Result<TokenStream> {
    if fields.is_empty() {
        return Err(syn::Error::new(fields.span(), EMPTY_FIELDS_MESSAGE));
    }

    Ok(match fields {
        syn::Fields::Named(fields) => span_fields_named::<IN_PATTERN>(fields),
        syn::Fields::Unnamed(fields) => span_fields_unnamed::<IN_PATTERN>(fields),
        syn::Fields::Unit => unreachable!(),
    })
}
fn span_fields_named<const IN_PATTERN: bool>(fields: syn::FieldsNamed) -> TokenStream {
    let len = fields.named.len();
    if len == 1 {
        let field = fields.named[0].ident.as_ref().unwrap();
        let binding = IN_PATTERN.then(|| quote! { { #field } => });
        let prefix = (!IN_PATTERN).then(|| quote! { self. });
        quote! { #binding wotw_seedgen_parse::Span::span(& #prefix #field) }
    } else {
        let first = fields.named[0].ident.as_ref().unwrap();
        let last = fields.named[len - 1].ident.as_ref().unwrap();
        let binding = IN_PATTERN.then(|| quote! { { #first, .., #last } => });
        let prefix = (!IN_PATTERN).then(|| quote! { self. });
        quote! { #binding wotw_seedgen_parse::Span::span(& #prefix #first).start..wotw_seedgen_parse::Span::span(& #prefix #last).end }
    }
}
fn span_fields_unnamed<const IN_PATTERN: bool>(fields: syn::FieldsUnnamed) -> TokenStream {
    let len = fields.unnamed.len();
    if len == 1 {
        let binding = IN_PATTERN.then(|| quote! { (first) => });
        let value = if IN_PATTERN {
            quote! { first }
        } else {
            quote! { self.0 }
        };

        quote! { #binding wotw_seedgen_parse::Span::span(& #value) }
    } else {
        let binding = IN_PATTERN.then(|| quote! { (first, .., last) => });
        let (first, last) = if IN_PATTERN {
            (quote! { first }, quote! { last })
        } else {
            let last = len - 1;
            (quote! { self.0 }, quote! { self.#last })
        };

        quote! { #binding wotw_seedgen_parse::Span::span(& #first).start..wotw_seedgen_parse::Span::span(& #last).end }
    }
}
fn span_enum(data: syn::DataEnum) -> Result<TokenStream> {
    let variants = data
        .variants
        .into_iter()
        .map(|variant| {
            let syn::Variant { ident, fields, .. } = variant;
            let fields = span_fields::<true>(fields)?;

            Ok(quote! {
                Self::#ident #fields
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        match self {
            #(#variants),*
        }
    })
}
