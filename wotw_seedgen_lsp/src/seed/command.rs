use itertools::Itertools;
use serde::de::DeserializeOwned;
use serde_json::{json, Value};
use strum::{EnumString, VariantNames};
use tower_lsp::jsonrpc::{Error, Result};
use wotw_seedgen_data::{
    assets::UberStateData,
    parse::Source,
    seed_language::ast::{self, parse_seed_ast},
};

use crate::seed::helpers::uber_identifier_info;

#[derive(EnumString, VariantNames)]
#[strum(serialize_all = "camelCase")]
pub enum Command {
    ResolveUberState,
}

pub fn execute_command(
    command: &str,
    arguments: Vec<Value>,
    uber_state_data: &UberStateData,
) -> Result<Option<Value>> {
    let command = command
        .parse()
        .map_err(|_| Error::invalid_params(format!("unknown command {command}")))?;

    match command {
        Command::ResolveUberState => resolve_uber_state(arguments, uber_state_data),
    }
}

fn resolve_uber_state(
    arguments: Vec<Value>,
    uber_state_data: &UberStateData,
) -> Result<Option<Value>> {
    let [input] = parse_arguments::<[String; 1]>(arguments)?;
    let source = Source::new("input".to_string(), input);

    let result = parse_seed_ast::<ast::UberIdentifier>(&source.content);

    let response = if !result.errors.is_empty() {
        let message = format!(
            "failed to parse \"{source}\": {errors}",
            source = source.content,
            errors = result
                .errors
                .into_iter()
                .format_with(", ", |err, f| f(&err.with_source(&source)))
        );

        json!({ "error": message })
    } else {
        match result
            .parsed
            .and_then(|uber_identifier| uber_identifier_info(&uber_identifier, uber_state_data))
        {
            None => json!({ "error": format!("unknown UberIdentifier {}", source.content) }),
            Some(info) => json!({ "info": info }),
        }
    };

    Ok(Some(response))
}

fn parse_arguments<T: DeserializeOwned>(arguments: Vec<Value>) -> Result<T> {
    serde_json::from_value(Value::Array(arguments))
        .map_err(|err| Error::invalid_params(format!("invalid arguments: {err}")))
}
