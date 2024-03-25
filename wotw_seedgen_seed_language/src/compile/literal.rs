use super::{Compile, SnippetCompiler};
use crate::{
    ast,
    output::intermediate::{Constant, ConstantDiscriminants, Literal},
};
use itertools::Itertools;
use ordered_float::OrderedFloat;
use wotw_seedgen_assets::UberStateAlias;
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_parse::{Error, Span};

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
            ast::UberIdentifier::Numeric(numeric) => Some(UberStateAlias {
                uber_identifier: UberIdentifier::new(numeric.group.data, numeric.member.data),
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
        let group = group?;
        let ids = group.get(self.member.data.0);
        if ids.is_none() {
            let mut error =
                Error::custom("Unknown UberState member".to_string(), self.member.span());

            let other_groups = compiler
                .global
                .uber_state_data
                .name_lookup
                .iter()
                .filter(|(_, v)| v.contains_key(self.member.data.0))
                .map(|(group_name, _)| format!("\"{}.{}\"", group_name, self.member.data.0))
                .collect::<Vec<_>>();

            error.help = if other_groups.is_empty() {
                suggestion(self.member.data.0, group.keys())
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
        let kind = compiler.consume_result(self.kind.data.0.parse().map_err(|_| {
            Error::custom(
                "Unknown Constant Kind".to_string(), // TODO more helpful pls?
                self.kind.span,
            )
        }))?;
        let variant = self.variant.data.0;

        // TODO list possible variants on wrong input?
        let constant = match kind {
            ConstantDiscriminants::Skill => variant
                .parse()
                .map_err(|_| Error::custom("Unknown Skill".to_string(), self.variant.span))
                .map(Constant::Skill),
            ConstantDiscriminants::Shard => variant
                .parse()
                .map_err(|_| Error::custom("Unknown Shard".to_string(), self.variant.span))
                .map(Constant::Shard),
            ConstantDiscriminants::Teleporter => variant
                .parse()
                .map_err(|_| Error::custom("Unknown Teleporter".to_string(), self.variant.span))
                .map(Constant::Teleporter),
            ConstantDiscriminants::WeaponUpgrade => variant
                .parse()
                .map_err(|_| Error::custom("Unknown WeaponUpgrade".to_string(), self.variant.span))
                .map(Constant::WeaponUpgrade),
            ConstantDiscriminants::Equipment => variant
                .parse()
                .map_err(|_| Error::custom("Unknown Equipment".to_string(), self.variant.span))
                .map(Constant::Equipment),
            ConstantDiscriminants::Zone => variant
                .parse()
                .map_err(|_| Error::custom("Unknown Zone".to_string(), self.variant.span))
                .map(Constant::Zone),
            ConstantDiscriminants::OpherIcon => variant
                .parse()
                .map_err(|_| Error::custom("Unknown OpherIcon".to_string(), self.variant.span))
                .map(Constant::OpherIcon),
            ConstantDiscriminants::LupoIcon => variant
                .parse()
                .map_err(|_| Error::custom("Unknown LupoIcon".to_string(), self.variant.span))
                .map(Constant::LupoIcon),
            ConstantDiscriminants::GromIcon => variant
                .parse()
                .map_err(|_| Error::custom("Unknown GromIcon".to_string(), self.variant.span))
                .map(Constant::GromIcon),
            ConstantDiscriminants::TuleyIcon => variant
                .parse()
                .map_err(|_| Error::custom("Unknown TuleyIcon".to_string(), self.variant.span))
                .map(Constant::TuleyIcon),
            ConstantDiscriminants::MapIcon => variant
                .parse()
                .map_err(|_| Error::custom("Unknown MapIcon".to_string(), self.variant.span))
                .map(Constant::MapIcon),
            ConstantDiscriminants::EquipSlot => variant
                .parse()
                .map_err(|_| Error::custom("Unknown EquipSlot".to_string(), self.variant.span))
                .map(Constant::EquipSlot),
            ConstantDiscriminants::WheelItemPosition => variant
                .parse()
                .map_err(|_| {
                    Error::custom("Unknown WheelItemPosition".to_string(), self.variant.span)
                })
                .map(Constant::WheelItemPosition),
            ConstantDiscriminants::WheelBind => variant
                .parse()
                .map_err(|_| Error::custom("Unknown WheelBind".to_string(), self.variant.span))
                .map(Constant::WheelBind),
            ConstantDiscriminants::Alignment => variant
                .parse()
                .map_err(|_| Error::custom("Unknown Alignment".to_string(), self.variant.span))
                .map(Constant::Alignment),
            ConstantDiscriminants::ScreenPosition => variant
                .parse()
                .map_err(|_| Error::custom("Unknown ScreenPosition".to_string(), self.variant.span))
                .map(Constant::ScreenPosition),
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
