use super::{Compile, SnippetCompiler};
use crate::{
    assets::{UberStateAlias, UberStateData, UberStateNameEntry},
    seed_language::{ast, compile::helpers::suggestion, output::Literal},
    UberIdentifier,
};
use itertools::Itertools;
use rustc_hash::FxHashMap;
use wotw_seedgen_parse::{Error, Span, Spanned, SpannedOption};

impl<'source> Compile<'source> for ast::Literal<'source> {
    type Output = Option<Literal>;

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_, '_>) -> Self::Output {
        match self {
            ast::Literal::UberIdentifier(uber_identifier) => uber_identifier
                .compile(compiler)
                .map(Literal::UberIdentifier),
            ast::Literal::Boolean(bool) => Some(Literal::Boolean(bool)),
            ast::Literal::Integer(int) => Some(Literal::Integer(int)),
            ast::Literal::Float(float) => Some(Literal::Float(float)),
            ast::Literal::String(string) => Some(Literal::String(string.into())),
            ast::Literal::Constant(constant) => Some(Literal::Constant(constant)),
        }
    }
}

impl<'source> Compile<'source> for ast::UberIdentifier<'source> {
    type Output = Option<UberStateAlias>;

    fn compile(self, compiler: &mut SnippetCompiler<'source, '_, '_, '_, '_>) -> Self::Output {
        let uber_state = self.resolve(compiler)?;

        if uber_state.uber_identifier.is_custom() {
            compiler.errors.push(Error::error(
                "Cannot use this group directly. Use !state instead".to_string(),
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
                .push(Error::error("Unknown UberState".to_string(), self.span()));

            None
        }
    }
}

impl ast::UberIdentifier<'_> {
    pub(crate) fn resolve(&self, compiler: &mut SnippetCompiler) -> Option<UberStateAlias> {
        match self {
            ast::UberIdentifier::Numeric(numeric) => {
                numeric.member.value.as_option().map(|member| {
                    UberStateAlias::identifier(UberIdentifier::new(numeric.group.data, member.data))
                })
            }
            ast::UberIdentifier::Name(name) => name.resolve(compiler),
        }
    }
}

impl ast::UberIdentifierName<'_> {
    fn resolve(&self, compiler: &mut SnippetCompiler) -> Option<UberStateAlias> {
        let Some(group) = compiler
            .global
            .uber_state_data
            .name_lookup
            .get(self.group.data.0)
        else {
            compiler.errors.push(Error {
                help: suggestion(
                    self.group.data.0,
                    compiler.global.uber_state_data.name_lookup.keys(),
                ),
                ..Error::error("Unknown UberState group".to_string(), self.group.span())
            });

            return None;
        };

        let member = self.member.value.as_option()?;

        let Some(entry) = group.get(member.data.0) else {
            let error = unknown_member_error(compiler.global.uber_state_data, group, member);
            compiler.errors.push(error);
            return None;
        };

        match entry {
            UberStateNameEntry::Vanilla(uber_identifiers) => {
                if matches!(self.pickup, SpannedOption::Some(_)) {
                    compiler.errors.push(Error::error(
                        "Vanilla UberStates can only have two parts".to_string(),
                        self.span(),
                    ));

                    return None;
                }

                if uber_identifiers.len() > 1 {
                    compiler.errors.push(Error::error(
                        format!(
                            "Ambiguous name: matches {}",
                            uber_identifiers.iter().format(", ")
                        ),
                        self.span(),
                    ));

                    return None;
                }

                uber_identifiers
                    .first()
                    .copied()
                    .map(UberStateAlias::identifier)
            }
            UberStateNameEntry::Rando(rando_group) => match &self.pickup {
                SpannedOption::Some(pickup) => {
                    let pickup = pickup.identifier.value.as_option()?;

                    let Some(alias) = rando_group.members.get(pickup.data.0) else {
                        let error = unknown_member_error(
                            compiler.global.uber_state_data,
                            &rando_group.members,
                            pickup,
                        );
                        compiler.errors.push(error);
                        return None;
                    };

                    Some(alias.clone())
                }
                SpannedOption::None(_) => {
                    if rando_group.root_member.is_none() {
                        compiler.errors.push(Error {
                            help: unknown_member_help(
                                compiler.global.uber_state_data,
                                group,
                                member.data.0,
                            ),
                            ..Error::error(
                                format!(
                                    "Group \"{zone}.{region}\" contains members, but does not name anything by itself",
                                    zone = self.group.data.0,
                                    region = member.data.0
                                ),
                                member.span()
                            )
                        });
                    }

                    rando_group.root_member.clone()
                }
            },
        }
    }
}

fn unknown_member_error<V>(
    uber_state_data: &UberStateData,
    group: &FxHashMap<String, V>,
    member: &Spanned<ast::Identifier>,
) -> Error {
    Error {
        help: unknown_member_help(uber_state_data, group, member.data.0),
        ..Error::error("Unknown UberState member".to_string(), member.span())
    }
}

fn unknown_member_help<V>(
    uber_state_data: &UberStateData,
    group: &FxHashMap<String, V>,
    member: &str,
) -> Option<String> {
    let other_groups = find_member(uber_state_data, member);
    if other_groups.is_empty() {
        suggestion(member, group.keys())
    } else {
        Some(if other_groups.len() == 1 {
            format!("It exists in another group: {}", other_groups[0])
        } else {
            format!(
                "It exists in other groups: {}",
                other_groups.into_iter().format(", ")
            )
        })
    }
}

fn find_member(uber_state_data: &UberStateData, member: &str) -> Vec<String> {
    let mut other_groups = Vec::new();

    for (group, members) in &uber_state_data.name_lookup {
        for (member_or_region, entry) in members {
            if member_or_region == member {
                other_groups.push(format!("\"{group}.{member}\""));
            }

            if let UberStateNameEntry::Rando(rando_group) = entry {
                if rando_group.members.contains_key(member) {
                    other_groups.push(format!("\"{group}.{member_or_region}.{member}\""));
                }
            }
        }
    }

    other_groups
}
