use std::fmt::Display;

use crate::{add_bound, find_attributes, Result};
use heck::{
    ToKebabCase, ToLowerCamelCase, ToPascalCase, ToShoutyKebabCase, ToShoutySnekCase, ToSnekCase,
    ToTitleCase, ToTrainCase,
};
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

    let implementation = match data {
        syn::Data::Struct(data) => ast_fields::<false, false>(&ident, data.fields, &attrs),
        syn::Data::Enum(data) => ast_enum(&ident, data, &attrs),
        syn::Data::Union(data) => Err(syn::Error::new(
            data.union_token.span(),
            "Deriving Ast on union is not supported",
        )),
    }?;

    let implementation =
        with_debug::<true, _, _>(implementation, attrs.debug, || format!("`{ident}`"));

    Ok(quote! {
        impl #impl_generics wotw_seedgen_parse::Ast<'source, Tokenizer> for #ident #type_generics {
            fn ast_impl<M: wotw_seedgen_parse::Mode>(parser: &mut wotw_seedgen_parse::Parser<'source, Tokenizer>) -> std::ops::ControlFlow<M::Error, Self> {
                use wotw_seedgen_parse::Ast;
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
    case: Option<for<'a> fn(&'a str) -> String>,
    rename: Option<syn::LitStr>,
    with: Option<syn::LitStr>,
}

impl Attributes {
    fn find(attrs: &[syn::Attribute]) -> Result<Self> {
        let mut attributes = Self::default();

        for meta in find_attributes(attrs, "ast")? {
            match meta {
                syn::Meta::Path(path) if path.is_ident("debug") => attributes.debug = true,
                syn::Meta::NameValue(meta) if meta.path.is_ident("token") => {
                    attributes.token = Some(meta.value);
                }
                syn::Meta::NameValue(meta) if meta.path.is_ident("case") => {
                    let case = match meta.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit),
                            ..
                        }) => match lit.value().as_str() {
                            "camelCase" => str::to_lower_camel_case,
                            "PascalCase" => str::to_pascal_case,
                            "kebab-case" => str::to_kebab_case,
                            "snake_case" => str::to_snek_case,
                            "SCREAMING_SNAKE_CASE" => str::TO_SHOUTY_SNEK_CASE,
                            "SCREAMING-KEBAB-CASE" => str::to_shouty_kebab_case,
                            "lowercase" => str::to_lowercase,
                            "UPPERCASE" => str::to_uppercase,
                            "title_case" => str::to_title_case,
                            "mixed_case" => str::to_lower_camel_case,
                            "Train-Case" => str::to_train_case,
                            _ => return Err(syn::Error::new_spanned(lit, "Unsupported case")),
                        },
                        other => {
                            return Err(syn::Error::new_spanned(other, "Expected string literal"))
                        }
                    };

                    attributes.case = Some(case);
                }
                syn::Meta::NameValue(meta) if meta.path.is_ident("rename") => {
                    let rename = match meta.value {
                        syn::Expr::Lit(syn::ExprLit {
                            lit: syn::Lit::Str(lit),
                            ..
                        }) => lit,
                        other => {
                            return Err(syn::Error::new_spanned(other, "Expected string literal"))
                        }
                    };

                    attributes.rename = Some(rename);
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
            rename: inner.rename.or(self.rename),
            with: inner.with.or(self.with),
        }
    }
}

fn ast_fn<const FORCE_OPTION_MODE: bool>(
    ty: syn::Type,
    with: Option<syn::LitStr>,
) -> Result<TokenStream> {
    with.map_or_else(
        || {
            let mode = if FORCE_OPTION_MODE {
                quote! { wotw_seedgen_parse::OptionMode }
            } else {
                quote! { M }
            };

            Ok(quote! {
                <#ty as wotw_seedgen_parse::Ast<'source, Tokenizer>>::ast_impl::<#mode>
            })
        },
        |with| parse_with(&with),
    )
}

fn parse_with(with: &syn::LitStr) -> Result<TokenStream> {
    let value = with.value();
    let path =
        syn::parse_str::<syn::Path>(&value).map_err(|err| syn::Error::new_spanned(with, err))?;

    Ok(quote! {
        #path
    })
}

// TODO use ast_option instead of ast_impl::<OptionMode> in enums
fn ast_fields<const IN_ENUM: bool, const FORCE_OPTION_MODE: bool>(
    ident: &syn::Ident,
    fields: syn::Fields,
    attrs: &Attributes,
) -> Result<TokenStream> {
    let needs_backup = fields.len() > 1;

    let fields = match fields {
        syn::Fields::Named(fields) => ast_fields_named::<FORCE_OPTION_MODE>(ident, fields, attrs),
        syn::Fields::Unnamed(fields) => {
            ast_fields_unnamed::<FORCE_OPTION_MODE>(ident, fields, attrs)
        }
        syn::Fields::Unit => return ast_fields_unit::<IN_ENUM, FORCE_OPTION_MODE>(ident, attrs),
    }?;

    let variant_construction = IN_ENUM.then(|| quote! { ::#ident });
    let mut flow = quote! {
        std::ops::ControlFlow::Continue(Self #variant_construction
            #fields
        )
    };

    if needs_backup {
        flow = quote! {
            {
                let before = parser.position();
                let flow = (|| #flow)();
                if flow.is_break() {
                    parser.jump(before);
                }
                flow
            }
        };
    }

    Ok(flow)
}

fn ast_fields_named<const FORCE_OPTION_MODE: bool>(
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
            let ident = ident.as_ref().unwrap();
            let attrs = Attributes::find(&attrs)?;

            let ast = ast_fn::<FORCE_OPTION_MODE>(ty, attrs.with)?;

            let value =
                with_debug::<true, _, _>(quote! { #ast(parser) }, outer_attrs.debug, || {
                    format!("field `{outer_ident}.{ident}`")
                });

            Ok(quote! { #ident: #value? })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        { #(#fields),* }
    })
}

fn ast_fields_unnamed<const FORCE_OPTION_MODE: bool>(
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

            let ast = ast_fn::<FORCE_OPTION_MODE>(ty, attrs.with)?;

            let value =
                with_debug::<true, _, _>(quote! { #ast(parser) }, outer_attrs.debug, || {
                    format!("field `{outer_ident}.{index}`")
                });

            Ok(quote! { #value? })
        })
        .collect::<Result<Vec<_>>>()?;

    Ok(quote! {
        (#(#field_asts),*)
    })
}

fn ast_fields_unit<const IN_ENUM: bool, const FORCE_OPTION_MODE: bool>(
    ident: &syn::Ident,
    attrs: &Attributes,
) -> Result<TokenStream> {
    let variant_construction = IN_ENUM.then(|| quote! { ::#ident });

    if let Some(token) = &attrs.token {
        let err = err::<FORCE_OPTION_MODE>(quote! {
            M::err(|| parser.error(wotw_seedgen_parse::ErrorKind::ExpectedToken(#token.to_string())))
        });

        return Ok(quote! {
            match parser.current().0 {
                #token => {
                    parser.step();
                    std::ops::ControlFlow::Continue(Self #variant_construction)
                }
                _ => std::ops::ControlFlow::Break(#err)
            }
        });
    }

    if let Some(with) = &attrs.with {
        return parse_with(with);
    }

    let str = match &attrs.rename {
        None => {
            let mut str = ident.to_string();
            if let Some(case) = attrs.case {
                // TODO should we make assumptions about the original case?
                str = case(&str);
            }
            str
        }
        Some(rename) => rename.value(),
    };

    let err = err::<FORCE_OPTION_MODE>(quote! {
        M::err(|| parser.error(wotw_seedgen_parse::ErrorKind::ExpectedToken(format!(concat!('"', #str, '"')))))
    });

    Ok(quote! {
        if parser.current_slice() == #str {
            parser.step();
            std::ops::ControlFlow::Continue(Self #variant_construction)
        } else {
            std::ops::ControlFlow::Break(#err)
        }
    })
}

fn err<const FORCE_OPTION_MODE: bool>(gen: TokenStream) -> TokenStream {
    if FORCE_OPTION_MODE {
        quote! { () }
    } else {
        gen
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

            Ok((ident, fields, attrs))
        })
        .collect::<Result<Vec<_>>>()?;

    let happy_variants = enum_variants::<true, _>(&variants, outer_ident, |flow, _| {
        quote! {
            if let std::ops::ControlFlow::Continue(value) = #flow {
                return std::ops::ControlFlow::Continue(value);
            }
        }
    })?;

    let sad_variants = enum_variants::<false, _>(&variants, outer_ident, |flow, description| {
        let fmt_string =
            format!("unexpected continue after parse already failed - parsing {description}. {{}}");

        quote! {
            {
                match #flow {
                    std::ops::ControlFlow::Continue(value) => panic!(#fmt_string, parser.debug_state()),
                    std::ops::ControlFlow::Break(err) => err,
                }
            }
        }
    })?;

    Ok(quote! {
        {
            #(#happy_variants)*

            let errors = vec![#(#sad_variants),*];
            std::ops::ControlFlow::Break(M::combine_errors(errors))
        }
    })
}

fn enum_variants<const FORCE_OPTION_MODE: bool, F>(
    variants: &[(syn::Ident, syn::Fields, Attributes)],
    outer_ident: &syn::Ident,
    mut gen: F,
) -> Result<Vec<TokenStream>>
where
    F: FnMut(TokenStream, String) -> TokenStream,
{
    variants
        .iter()
        .map(|(ident, fields, attrs)| -> Result<_> {
            let description = format!("branch `{outer_ident}::{ident}`");

            let fields = ast_fields::<true, FORCE_OPTION_MODE>(ident, fields.clone(), attrs)?;

            let flow =
                with_debug::<false, _, _>(quote! { (|| { #fields })() }, attrs.debug, || {
                    &description
                });

            Ok(gen(flow, description))
        })
        .collect()
}

fn with_debug<const WITH_PARSER_STATE: bool, F, D>(
    statement: TokenStream,
    debug: bool,
    description: F,
) -> TokenStream
where
    F: FnOnce() -> D,
    D: Display,
{
    if debug {
        let description = description();

        let before = {
            let (fmt_string_suffix, fmt_arg) = if WITH_PARSER_STATE {
                (" - {}", quote! { , parser.debug_state() })
            } else {
                ("", TokenStream::new())
            };

            let fmt_string = format!("parsing {description}{fmt_string_suffix}");

            quote! { eprintln!(#fmt_string #fmt_arg); }
        };

        let after = {
            let fmt_string = format!("finished {description} ({{}})");
            quote! { eprintln!(#fmt_string, if flow.is_continue() { "Continue" } else { "Break" }); }
        };

        quote! {
            {
                #before
                let flow = #statement;
                #after
                flow
            }
        }
    } else {
        statement
    }
}
