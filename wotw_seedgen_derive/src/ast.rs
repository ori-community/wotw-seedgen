use crate::{add_bound, find_attributes, Result};
use convert_case::{Case, Casing};
use proc_macro2::TokenStream;
use quote::quote;
use syn::spanned::Spanned;

// TODO just found out about parse_quote

pub fn ast_impl(input: syn::DeriveInput) -> Result<proc_macro::TokenStream> {
    let syn::DeriveInput {
        ident,
        mut generics,
        data,
        attrs,
        ..
    } = input;
    let attrs = Attributes::find(&attrs)?;

    let implementation = match data {
        syn::Data::Struct(data) => ast_fields::<false>(&ident, data.fields, &attrs),
        syn::Data::Enum(data) => ast_enum(&ident, data, &attrs),
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span(),
            "Deriving Ast on union is not supported",
        )),
    }?;

    // TODO try out how using the // dbg! macro looks for the debug stuff?
    let debug = attrs.debug.then(|| {
        let message =
            format!("parsing `{ident}`. current token is `{{:?}}`. current slice is {{:?}}. upcoming is {{:?}}{{}}");
        quote! {
            let (token, span) = parser.current();
            let upcoming = parser.slice(span.start..);
            let (upcoming, more) = if upcoming.len() > 32 {
                (&upcoming[..32], "...")
            } else {
                (upcoming, "")
            };
            eprintln!(#message, token, parser.slice(span.clone()), upcoming, more);
        }
    });

    add_bound(
        &mut generics,
        quote! { wotw_seedgen_parse::Ast<'source, Tokenizer> },
    );
    let (mut impl_generics, type_generics, where_clause) = generics.split_for_impl();
    let type_generics = quote! { #type_generics #where_clause };
    if !generics
        .lifetimes()
        .any(|param| param.lifetime.ident == "source")
    {
        generics.params.push(syn::parse_quote! { 'source });
        impl_generics = generics.split_for_impl().0;
    }

    Ok(quote! {
        impl #impl_generics wotw_seedgen_parse::Ast<'source, Tokenizer> for #ident #type_generics {
            fn ast(parser: &mut wotw_seedgen_parse::Parser<'source, Tokenizer>) -> wotw_seedgen_parse::Result<Self> {
                use wotw_seedgen_parse::Ast;
                #debug
                #implementation
            }
        }
    }
    .into())
}

#[derive(Default, Clone)]
struct Attributes {
    debug: bool,
    token: Option<syn::Expr>,
    case: Option<Case>,
    with: Option<syn::LitStr>,
}
impl Attributes {
    fn find(attrs: &[syn::Attribute]) -> Result<Self> {
        let mut attributes = Self::default();

        for meta in find_attributes(attrs, "ast")? {
            match meta {
                syn::Meta::Path(path) if path.is_ident("debug") => attributes.debug = true,
                syn::Meta::NameValue(meta) if meta.path.is_ident("case") => {
                    let case = match meta.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit),
                            ..
                        }) => match lit.value().as_str() {
                            "upper" => Case::Upper,
                            "lower" => Case::Lower,
                            "camel" => Case::Camel,
                            "pascal" => Case::Pascal,
                            "snake" => Case::Snake,
                            "upper_snake" => Case::UpperSnake,
                            "kebab" => Case::Kebab,
                            "cobol" => Case::Cobol,
                            _ => return Err(syn::Error::new_spanned(lit, "Unsupported case")),
                        },
                        other => {
                            return Err(syn::Error::new_spanned(other, "Expected string literal"))
                        }
                    };
                    attributes.case = Some(case);
                }
                syn::Meta::NameValue(meta) if meta.path.is_ident("token") => {
                    attributes.token = Some(meta.value);
                }
                syn::Meta::NameValue(meta) if meta.path.is_ident("with") => {
                    let with = match meta.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit),
                            ..
                        }) => lit,
                        other => {
                            return Err(syn::Error::new_spanned(other, "Expected string literal"))
                        }
                    };
                    attributes.with = Some(with);
                }
                _ => return Err(syn::Error::new_spanned(meta, "Unrecognized attribute")),
            }
        }

        Ok(attributes)
    }
    fn apply(self, inner: Self) -> Self {
        Self {
            debug: inner.debug || self.debug,
            token: inner.token.or(self.token),
            case: inner.case.or(self.case),
            with: inner.with.or(self.with),
        }
    }
}

fn use_with_or_else<F: FnOnce() -> Result<TokenStream>>(
    default: F,
    with: Option<syn::LitStr>,
) -> Result<TokenStream> {
    with.map_or_else(default, |with| {
        let value = with.value();
        let path = syn::parse_str::<syn::Path>(&value)?;
        Ok(quote! {
            #path
        })
    })
}

fn ast_fields<const IN_ENUM: bool>(
    ident: &syn::Ident,
    fields: syn::Fields,
    attrs: &Attributes,
) -> Result<TokenStream> {
    let needs_backup = fields.len() > 1;

    let fields = match fields {
        syn::Fields::Named(fields) => ast_fields_named(ident, fields, attrs),
        syn::Fields::Unnamed(fields) => ast_fields_unnamed(ident, fields, attrs),
        syn::Fields::Unit => return Ok(ast_fields_unit::<IN_ENUM>(ident, attrs)),
    }?;

    let variant_construction = IN_ENUM.then(|| quote! { ::#ident });
    let mut result = quote! {
        Ok(Self #variant_construction
            #fields
        )
    };

    if needs_backup {
        result = quote! {
            let before = parser.position();
            let result = (|| #result)();
            if result.is_err() {
                parser.jump(before);
            }
            result
        };
    }

    Ok(result)
}
fn ast_fields_named(
    outer_ident: &syn::Ident,
    fields: syn::FieldsNamed,
    outer_attrs: &Attributes,
) -> Result<TokenStream> {
    let fields = fields
        .named
        .into_iter()
        .map(|field| {
            let syn::Field {
                ident, attrs, ty, ..
            } = field;
            let attrs = Attributes::find(&attrs)?;

            let ast = use_with_or_else(
                || {
                    Ok(quote! {
                        <#ty as wotw_seedgen_parse::Ast<'source, Tokenizer>>::ast
                    })
                },
                attrs.with,
            )?;

            let value = if outer_attrs.debug {
                let ident = ident.as_ref().unwrap();
                let fmt_string_before = format!("parsing field `{outer_ident}.{ident}`");
                let fmt_string_after =
                    format!("finished parsing field `{outer_ident}.{ident}` ({{}})");
                quote! { {
                    eprintln!(#fmt_string_before);
                    let result = #ast(parser);
                    eprintln!(#fmt_string_after, if result.is_ok() { "Ok" } else { "Err" });
                    result?
                } }
            } else {
                quote! { #ast(parser)? }
            };

            Ok(quote! {
                #ident: #value
            })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        { #(#fields),* }
    })
}
fn ast_fields_unnamed(
    outer_ident: &syn::Ident,
    fields: syn::FieldsUnnamed,
    outer_attrs: &Attributes,
) -> Result<TokenStream> {
    let field_asts = fields
        .unnamed
        .into_iter()
        .enumerate()
        .map(|(index, field)| {
            let syn::Field { ty, attrs, .. } = field;
            let attrs = Attributes::find(&attrs)?;

            let ast = use_with_or_else(
                || {
                    Ok(quote! {
                        <#ty as wotw_seedgen_parse::Ast<'source, Tokenizer>>::ast
                    })
                },
                attrs.with,
            )?;

            let value = if outer_attrs.debug {
                let fmt_string_before = format!("parsing field `{outer_ident}.{index}`");
                let fmt_string_after =
                    format!("finished parsing field `{outer_ident}.{index}` ({{}})");
                quote! { {
                    eprintln!(#fmt_string_before);
                    let result = #ast(parser);
                    eprintln!(#fmt_string_after, if result.is_ok() { "Ok" } else { "Err" });
                    result?
                } }
            } else {
                quote! { #ast(parser)? }
            };

            Ok(value)
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        (#(#field_asts),*)
    })
}
fn ast_fields_unit<const IN_ENUM: bool>(ident: &syn::Ident, attrs: &Attributes) -> TokenStream {
    let variant_construction = IN_ENUM.then(|| quote! { ::#ident });

    if let Some(token) = &attrs.token {
        return quote! {
            match parser.current().0 {
                #token => {
                    parser.step();
                    Ok(Self #variant_construction)
                }
                _ => Err(parser.error(wotw_seedgen_parse::ErrorKind::ExpectedToken(
                    #token.to_string()
                )))
            }
        };
    }

    let mut str = ident.to_string();
    if let Some(case) = attrs.case {
        // TODO should we make assumptions about the original case?
        str = str.to_case(case);
    }

    quote! {
        if parser.current_slice() == #str {
            parser.step();
            Ok(Self #variant_construction)
        } else {
            Err(parser.error(wotw_seedgen_parse::ErrorKind::ExpectedToken(format!(concat!('"', #str, '"')))))
        }
    }
}

fn ast_enum(
    outer_ident: &syn::Ident,
    data: syn::DataEnum,
    attrs: &Attributes,
) -> Result<TokenStream> {
    let variants = data
        .variants
        .into_iter()
        .map(|variant| -> Result<_> {
            let syn::Variant {
                ident,
                fields,
                attrs: variant_attrs,
                ..
            } = variant;
            let attrs = attrs.clone().apply(Attributes::find(&variant_attrs)?);
            let fields = ast_fields::<true>(&ident, fields, &attrs)?;

            let debug_before = attrs.debug.then(|| {
                let fmt_string = format!("attempting branch `{outer_ident}::{ident}`");
                quote! { eprintln!(#fmt_string); }
            });
            let debug_after = attrs.debug.then(|| {
                let fmt_string = format!("finished branch `{outer_ident}::{ident}` ({{}})");
                quote! { eprintln!(#fmt_string, if result.is_ok() { "Ok" } else { "Err" }); }
            });

            Ok(quote! {
                #debug_before
                let result = (|| { #fields })();
                #debug_after
                match result {
                    Ok(_) => return result,
                    Err(err) => errors.push(err),
                }
            })
        })
        .collect::<Result<Vec<_>>>()?;

    let variant_count = variants.len();

    Ok(quote! {
        let mut errors = Vec::with_capacity(#variant_count);

        #(#variants)*

        Err(wotw_seedgen_parse::Error::all_failed(errors))
    })
}
