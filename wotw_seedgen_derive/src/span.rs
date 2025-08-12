use crate::{add_bound, Result};
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

const UNION_MESSAGE: &str = "Cannot implement Span on union";
const EMPTY_FIELDS_MESSAGE: &str = "Cannot implement Span without any fields";

pub fn span_impl(input: syn::DeriveInput) -> Result<proc_macro::TokenStream> {
    let syn::DeriveInput {
        ident,
        generics,
        data,
        ..
    } = input;

    // TODO bounds could be smarter by analyzing which types actually need to implement traits

    let span = span_any_impl(
        &ident,
        generics.clone(),
        data.clone(),
        vec![
            quote! { wotw_seedgen_parse::Span },
            quote! { wotw_seedgen_parse::SpanStart },
            quote! { wotw_seedgen_parse::SpanEnd },
        ],
        quote! { wotw_seedgen_parse::Span },
        quote! { fn span(&self) -> std::ops::Range<usize> },
        span_fields_named::<false>,
        span_fields_named::<true>,
        span_fields_unnamed::<false>,
        span_fields_unnamed::<true>,
    )?;

    let span_start = span_any_impl(
        &ident,
        generics.clone(),
        data.clone(),
        vec![quote! { wotw_seedgen_parse::SpanStart }],
        quote! { wotw_seedgen_parse::SpanStart },
        quote! { fn span_start(&self) -> usize },
        span_start_fields_named::<false>,
        span_start_fields_named::<true>,
        span_start_fields_unnamed::<false>,
        span_start_fields_unnamed::<true>,
    )?;

    let span_end = span_any_impl(
        &ident,
        generics,
        data,
        vec![quote! { wotw_seedgen_parse::SpanEnd }],
        quote! { wotw_seedgen_parse::SpanEnd },
        quote! { fn span_end(&self) -> usize },
        span_end_fields_named::<false>,
        span_end_fields_named::<true>,
        span_end_fields_unnamed::<false>,
        span_end_fields_unnamed::<true>,
    )?;

    Ok(quote! {
        #span

        #span_start

        #span_end
    }
    .into())
}

fn span_any_impl<FN, FNP, FU, FUP>(
    ident: &syn::Ident,
    mut generics: syn::Generics,
    data: syn::Data,
    bounds: Vec<TokenStream>,
    span_trait: TokenStream,
    span_trait_signature: TokenStream,
    named: FN,
    named_pattern: FNP,
    unnamed: FU,
    unnamed_pattern: FUP,
) -> Result<TokenStream>
where
    FN: Fn(syn::FieldsNamed) -> TokenStream,
    FNP: Fn(syn::FieldsNamed) -> TokenStream,
    FU: Fn(syn::FieldsUnnamed) -> TokenStream,
    FUP: Fn(syn::FieldsUnnamed) -> TokenStream,
{
    let implementation =
        span_any_implementation(data, named, named_pattern, unnamed, unnamed_pattern)?;

    for bound in bounds {
        add_bound(&mut generics, bound);
    }

    let (impl_generics, type_generics, where_clause) = generics.split_for_impl();

    Ok(quote! {
        impl #impl_generics #span_trait for #ident #type_generics #where_clause {
            #span_trait_signature {
                #implementation
            }
        }
    })
}

fn span_any_implementation<FN, FNP, FU, FUP>(
    data: syn::Data,
    named: FN,
    named_pattern: FNP,
    unnamed: FU,
    unnamed_pattern: FUP,
) -> Result<TokenStream>
where
    FN: Fn(syn::FieldsNamed) -> TokenStream,
    FNP: Fn(syn::FieldsNamed) -> TokenStream,
    FU: Fn(syn::FieldsUnnamed) -> TokenStream,
    FUP: Fn(syn::FieldsUnnamed) -> TokenStream,
{
    match data {
        syn::Data::Struct(data) => span_fields(data.fields, named, unnamed),
        syn::Data::Enum(data) => span_enum(data, named_pattern, unnamed_pattern),
        syn::Data::Union(union) => Err(syn::Error::new(union.union_token.span(), UNION_MESSAGE)),
    }
}

fn span_fields<FN, FU>(fields: syn::Fields, named: FN, unnamed: FU) -> Result<TokenStream>
where
    FN: Fn(syn::FieldsNamed) -> TokenStream,
    FU: Fn(syn::FieldsUnnamed) -> TokenStream,
{
    if fields.is_empty() {
        return Err(syn::Error::new(fields.span(), EMPTY_FIELDS_MESSAGE));
    }

    Ok(match fields {
        syn::Fields::Named(fields) => named(fields),
        syn::Fields::Unnamed(fields) => unnamed(fields),
        syn::Fields::Unit => unreachable!(),
    })
}
fn span_any_fields_named<const IN_PATTERN: bool>(
    field: &syn::Ident,
    f: TokenStream,
) -> TokenStream {
    let binding = IN_PATTERN.then(|| quote! { { #field, .. } => });
    let prefix = (!IN_PATTERN).then(|| quote! { self. });
    quote! { #binding wotw_seedgen_parse::#f(& #prefix #field) }
}
fn span_fields_named<const IN_PATTERN: bool>(fields: syn::FieldsNamed) -> TokenStream {
    let len = fields.named.len();
    if len == 1 {
        let field = fields.named[0].ident.as_ref().unwrap();
        span_any_fields_named::<IN_PATTERN>(field, quote! { Span::span })
    } else {
        let first = fields.named[0].ident.as_ref().unwrap();
        let last: &syn::Ident = fields.named[len - 1].ident.as_ref().unwrap();
        let binding = IN_PATTERN.then(|| quote! { { #first, .., #last } => });
        let prefix = (!IN_PATTERN).then(|| quote! { self. });
        quote! { #binding wotw_seedgen_parse::SpanStart::span_start(& #prefix #first)..wotw_seedgen_parse::SpanEnd::span_end(& #prefix #last) }
    }
}
fn span_start_fields_named<const IN_PATTERN: bool>(fields: syn::FieldsNamed) -> TokenStream {
    let first = fields.named[0].ident.as_ref().unwrap();
    span_any_fields_named::<IN_PATTERN>(first, quote! { SpanStart::span_start })
}
fn span_end_fields_named<const IN_PATTERN: bool>(fields: syn::FieldsNamed) -> TokenStream {
    let last = fields.named.last().unwrap().ident.as_ref().unwrap();
    span_any_fields_named::<IN_PATTERN>(last, quote! { SpanEnd::span_end })
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

        quote! { #binding wotw_seedgen_parse::SpanStart::span_start(& #first)..wotw_seedgen_parse::SpanEnd::span_end(& #last) }
    }
}
fn span_start_fields_unnamed<const IN_PATTERN: bool>(_fields: syn::FieldsUnnamed) -> TokenStream {
    let binding = IN_PATTERN.then(|| quote! { (first, ..) => });
    let first = if IN_PATTERN {
        quote! { first }
    } else {
        quote! { self.0 }
    };

    quote! { #binding wotw_seedgen_parse::SpanStart::span_start(& #first) }
}
fn span_end_fields_unnamed<const IN_PATTERN: bool>(fields: syn::FieldsUnnamed) -> TokenStream {
    let binding = IN_PATTERN.then(|| quote! { (.., last) => });
    let last = if IN_PATTERN {
        quote! { last }
    } else {
        let last = syn::Index::from(fields.unnamed.len() - 1);
        quote! { self.#last }
    };

    quote! { #binding wotw_seedgen_parse::SpanEnd::span_end(& #last) }
}
fn span_enum<FN, FU>(data: syn::DataEnum, named: FN, unnamed: FU) -> Result<TokenStream>
where
    FN: Fn(syn::FieldsNamed) -> TokenStream,
    FU: Fn(syn::FieldsUnnamed) -> TokenStream,
{
    let variants = data
        .variants
        .into_iter()
        .map(|variant| {
            let syn::Variant { ident, fields, .. } = variant;
            let fields = span_fields(fields, &named, &unnamed)?;

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
