use std::{fmt::Display, ops::Range, str::FromStr};

use super::{Compile, SnippetCompiler};
use crate::{
    ast,
    output::{Constant, ConstantDiscriminants, Literal},
    parse::Result,
};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use strum::VariantArray;
use wotw_seedgen_assets::UberStateAlias;
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_parse::{Error, Identifier, Span, Spanned};

impl<'source> Compile<'source> for ast::Literal<'source> {
    type Output = Option<Literal>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Literal::UberIdentifier(uber_identifier) => uber_identifier
                .compile(compiler)
                .map(Literal::UberIdentifier),
            ast::Literal::Boolean(bool) => Some(Literal::Boolean(bool)),
            ast::Literal::Integer(int) => Some(Literal::Integer(int)),
            ast::Literal::Float(float) => Some(Literal::Float(float)),
            ast::Literal::String(string) => Some(Literal::String(string.into())),
            ast::Literal::Constant(constant) => constant.compile(compiler).map(Literal::Constant),
        }
    }
}

impl<'source> Compile<'source> for ast::UberIdentifier<'source> {
    type Output = Option<UberStateAlias>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let uber_state = self.resolve(compiler)?;

        if uber_state.uber_identifier.group == 9 {
            compiler.errors.push(Error::custom(
                "Cannot use group 9 directly. Use !state instead".to_string(),
                self.span(),
            ));

            None
            // TODO why is there an extra check here?
        } else if compiler
            .global
            .uber_state_data
            .id_lookup
            .contains_key(&uber_state.uber_identifier)
        {
            Some(uber_state)
        } else {
            compiler
                .errors
                .push(Error::custom("Unknown UberState".to_string(), self.span()));

            None
        }
    }
}

impl ast::UberIdentifier<'_> {
    pub(crate) fn resolve(&self, compiler: &mut SnippetCompiler) -> Option<UberStateAlias> {
        match self {
            ast::UberIdentifier::Numeric(numeric) => compiler
                .consume_result(numeric.member.result.clone())
                .map(|member| UberStateAlias {
                    uber_identifier: UberIdentifier::new(numeric.group.data, member.data),
                    value: None,
                }),
            ast::UberIdentifier::Name(name) => name.resolve(compiler),
        }
    }
}

impl ast::UberIdentifierName<'_> {
    fn resolve(&self, compiler: &mut SnippetCompiler) -> Option<UberStateAlias> {
        let group = compiler
            .global
            .uber_state_data
            .name_lookup
            .get(self.group.data.0);

        if group.is_none() {
            let mut error = Error::custom("Unknown UberState group".to_string(), self.group.span());
            error.help = suggestion(
                self.group.data.0,
                compiler.global.uber_state_data.name_lookup.keys(),
            );
            compiler.errors.push(error);
        }

        let member = compiler.consume_result(self.member.result.clone());

        let group = group?;
        let member = member?;

        let ids = group.get(member.data.0);
        if ids.is_none() {
            let mut error = Error::custom("Unknown UberState member".to_string(), member.span());

            let other_groups = compiler
                .global
                .uber_state_data
                .name_lookup
                .iter()
                .filter(|(_, v)| v.contains_key(member.data.0))
                .map(|(group_name, _)| format!("\"{}.{}\"", group_name, member.data.0))
                .collect::<Vec<_>>();

            error.help = if other_groups.is_empty() {
                suggestion(member.data.0, group.keys())
            } else {
                let help = if other_groups.len() == 1 {
                    format!("It exists in another group: {}", other_groups[0])
                } else {
                    format!(
                        "It exists in other groups: {}",
                        other_groups.into_iter().format(", ")
                    )
                };

                Some(help)
            };

            compiler.errors.push(error);
        }
        let ids = ids?;

        if ids.len() == 1 {
            ids.first().cloned()
        } else {
            compiler.errors.push(Error::custom(
                format!("Ambiguous name: matches {}", ids.iter().format(", ")),
                self.span(),
            ));

            None
        }
    }
}

impl<'source> Compile<'source> for ast::Constant<'source> {
    type Output = Option<Constant>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let kind = compiler.consume_result(
            self.kind
                .data
                .0
                .parse()
                .map_err(|err| Error::custom(err, self.kind.span)),
        );

        let variant = compiler.consume_result(self.variant.result);

        let kind = kind?;
        let Spanned {
            data: Identifier(variant),
            span,
        } = variant?;

        fn parse_variant<T1, T2, F>(variant: &str, span: Range<usize>, f: F) -> Result<T2>
        where
            T1: FromStr<Err = String> + VariantArray + Display,
            F: FnOnce(T1) -> T2,
        {
            variant
                .parse()
                .map(f)
                .map_err(|err| Error::custom(err, span))
        }

        let constant = match kind {
            ConstantDiscriminants::Skill => parse_variant(variant, span, Constant::Skill),
            ConstantDiscriminants::Shard => parse_variant(variant, span, Constant::Shard),
            ConstantDiscriminants::Teleporter => parse_variant(variant, span, Constant::Teleporter),
            ConstantDiscriminants::WeaponUpgrade => {
                parse_variant(variant, span, Constant::WeaponUpgrade)
            }
            ConstantDiscriminants::Equipment => parse_variant(variant, span, Constant::Equipment),
            ConstantDiscriminants::Zone => parse_variant(variant, span, Constant::Zone),
            ConstantDiscriminants::OpherIcon => parse_variant(variant, span, Constant::OpherIcon),
            ConstantDiscriminants::LupoIcon => parse_variant(variant, span, Constant::LupoIcon),
            ConstantDiscriminants::GromIcon => parse_variant(variant, span, Constant::GromIcon),
            ConstantDiscriminants::TuleyIcon => parse_variant(variant, span, Constant::TuleyIcon),
            ConstantDiscriminants::MapIcon => parse_variant(variant, span, Constant::MapIcon),
            ConstantDiscriminants::EquipSlot => parse_variant(variant, span, Constant::EquipSlot),
            ConstantDiscriminants::WheelItemPosition => {
                parse_variant(variant, span, Constant::WheelItemPosition)
            }
            ConstantDiscriminants::WheelBind => parse_variant(variant, span, Constant::WheelBind),
            ConstantDiscriminants::Alignment => parse_variant(variant, span, Constant::Alignment),

            ConstantDiscriminants::HorizontalAnchor => {
                parse_variant(variant, span, Constant::HorizontalAnchor)
            }
            ConstantDiscriminants::VerticalAnchor => {
                parse_variant(variant, span, Constant::VerticalAnchor)
            }
            ConstantDiscriminants::ScreenPosition => {
                parse_variant(variant, span, Constant::ScreenPosition)
            }
            ConstantDiscriminants::CoordinateSystem => {
                parse_variant(variant, span, Constant::CoordinateSystem)
            }
        };

        compiler.consume_result(constant)
    }
}

fn suggestion<T, I>(input: &str, options: I) -> Option<String>
where
    T: AsRef<str>,
    I: IntoIterator<Item = T>,
{
    let mut distances = options
        .into_iter()
        .map(|option| (strsim::jaro(input, option.as_ref()), option))
        .filter(|(confidence, option)| *confidence > 0.7 || option.as_ref().contains(input))
        .collect::<Vec<_>>();

    (!distances.is_empty()).then(|| {
        if distances.len() == 1 {
            format!("Did you mean \"{}\"?", distances[0].1.as_ref())
        } else {
            distances.sort_unstable_by(|a, b| OrderedFloat(b.0).cmp(&OrderedFloat(a.0)));

            format!(
                "Did you mean one of these? {}",
                distances
                    .into_iter()
                    .map(|(_, option)| format!("\"{}\"", option.as_ref()))
                    .format(", ")
            )
        }
    })
}
