use std::{ops::Range, sync::LazyLock};

use regex::Regex;
use strum::VariantNames;
use wotw_seedgen_parse::Error;

use crate::{
    seed_language::{ast::Handler, compile::helpers::suggestion},
    InputAction,
};

pub struct FormatStrings<'e> {
    errors: &'e mut Vec<Error>,
}
impl<'e> FormatStrings<'e> {
    pub fn new(errors: &'e mut Vec<Error>) -> Self {
        Self { errors }
    }
}

impl Handler for FormatStrings<'_> {
    fn string_literal(&mut self, s: &str, span: &Range<usize>) {
        // Only match letters to minimize false positives on aesthetic bracket usage
        static INPUT_ACTION_SHAPED: LazyLock<Regex> =
            LazyLock::new(|| Regex::new(r"\[([a-zA-Z]*)\]").unwrap());

        let content_start = span.start + 1;

        for input_action in INPUT_ACTION_SHAPED.captures_iter(s) {
            let input_action = input_action.get(1).unwrap();
            let input_action_str = input_action.as_str();

            if !InputAction::VARIANTS.contains(&input_action_str) {
                let span = input_action.start() + content_start..input_action.end() + content_start;

                let mut error = Error::warning("Unknown Input Action".to_string(), span);

                if let Some(suggestion) = suggestion(input_action_str, InputAction::VARIANTS) {
                    error = error.with_help(suggestion);
                }

                self.errors.push(error);
            }
        }
    }
}
