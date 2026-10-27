use std::ops::Range;

use wotw_seedgen_parse::{Error, Span, Spanned};

use crate::{
    assets::{UberStateData, UberStateNameEntry},
    seed_language::{
        ast::{
            self, Comparator, Expression, ExpressionValue, Handler, Literal, Operation, Operator,
        },
        compile::{
            lint::{Fix, FixKind},
            SourceErrors,
        },
    },
};

pub struct PreferredNames<'u> {
    uber_state_data: &'u UberStateData,
    errors: Vec<Error>,
    fixes: Option<Vec<Fix>>,
}

impl<'u> PreferredNames<'u> {
    pub fn new(uber_state_data: &'u UberStateData, collect_fixes: bool) -> Self {
        Self {
            uber_state_data,
            errors: Vec::new(),
            fixes: collect_fixes.then(Vec::new),
        }
    }

    pub fn finish(mut self, errors: &mut SourceErrors) {
        errors.errors.append(&mut self.errors);

        if let Some(mut fixes) = self.fixes {
            errors.fixes.append(&mut fixes);
        }
    }

    fn check_uber_identifier(&self, uber_identifier: &ast::UberIdentifier) -> Option<&'u String> {
        match uber_identifier {
            ast::UberIdentifier::Numeric(numeric) => {
                let id_entry = numeric.inspect(self.uber_state_data)?;

                id_entry.rando_name.as_ref().or_else(|| {
                    let (group, member) = id_entry.name.split_once('.')?;
                    let name_entry = self.uber_state_data.name_lookup.get(group)?.get(member)?;

                    let UberStateNameEntry::Vanilla(ids) = name_entry else {
                        return None;
                    };

                    (ids.len() == 1).then_some(&id_entry.name)
                })
            }
            ast::UberIdentifier::Name(name) => {
                let name_entry = name.inspect(self.uber_state_data)?;

                let UberStateNameEntry::Vanilla(ids) = name_entry else {
                    return None;
                };

                if ids.len() != 1 {
                    return None;
                }
                let id_entry = self.uber_state_data.id_lookup.get(ids.first()?)?;

                id_entry.rando_name.as_ref()
            }
        }
    }

    fn check_expression(
        &self,
        uber_identifier: &ast::UberIdentifier,
        value: i32,
    ) -> Option<&'u String> {
        let alias = uber_identifier.inspect_resolve(self.uber_state_data)?;

        if alias.value.is_some() {
            return None;
        }

        let id_entry = &self.uber_state_data.id_lookup[&alias.uber_identifier];
        let alias = id_entry
            .expression_aliases
            .iter()
            .find(|alias| alias.value == value)?;

        Some(&alias.name)
    }

    fn push_fix(&mut self, span: &Range<usize>, content: &String) {
        if let Some(fixes) = &mut self.fixes {
            fixes.push(Fix {
                kind: FixKind::PreferredName,
                span: span.clone(),
                content: content.clone(),
            });
        }
    }
}

impl Handler for PreferredNames<'_> {
    fn uber_identifier(&mut self, uber_identifier: &ast::UberIdentifier) {
        if let Some(clearer_name) = self.check_uber_identifier(uber_identifier) {
            let span = uber_identifier.span();

            self.push_fix(&span, clearer_name);

            self.errors.push(Error::warning(
                format!("This UberState has a clearer name available: {clearer_name}"),
                span,
            ));
        }
    }

    fn expression(&mut self, expression: &Expression) {
        let Expression::Operation(operation) = expression else {
            return;
        };

        let Operation {
            left:
                Expression::Value(ExpressionValue::Literal(Spanned {
                    data: Literal::UberIdentifier(uber_identifier),
                    ..
                })),
            operator:
                Spanned {
                    data: Operator::Comparator(Comparator::GreaterOrEqual),
                    ..
                },
            right:
                Expression::Value(ExpressionValue::Literal(Spanned {
                    data: Literal::Integer(value),
                    ..
                })),
        } = &**operation
        else {
            return;
        };

        if let Some(alias) = self.check_expression(uber_identifier, *value) {
            let span = expression.span();

            self.push_fix(&span, alias);

            self.errors.push(Error::warning(
                format!("This Expression has an alias available: {alias}"),
                span,
            ));
        }
    }
}
