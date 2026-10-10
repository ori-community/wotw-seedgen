use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rand_pcg::Pcg64Mcg;
use serde::Serialize;
use tokio::sync::RwLockReadGuard;
use utoipa::ToSchema;
use wotw_seedgen::{
    data::{
        assets::{AssetCacheValues, ChainedSnippetAccess, InlineSnippets},
        seed_language::{
            compile::{self, Compiler},
            output::postprocess,
        },
    },
    log_capture::{LogCapture, Record},
    seed::{PlandoAttributes, Seed},
};

use crate::{
    api::plandos::{CompileBody, CompileQuery},
    assets::Cache,
};

pub type CompileResult = Result<Vec<u8>, CompileError>;

#[derive(Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompileError {
    pub errors: Vec<String>,
    pub logs: Vec<Record>,
}

impl IntoResponse for CompileError {
    fn into_response(self) -> Response {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(self)).into_response()
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
pub struct CompileOutput {
    pub seed: ciborium::Value,
    pub logs: Vec<Record>,
}

pub fn compile(
    query: CompileQuery,
    body: CompileBody,
    cache: RwLockReadGuard<Cache>,
) -> CompileResult {
    let CompileQuery {
        debug,
        max_log_level,
    } = query;

    let CompileBody { snippets, entry } = body;

    let mut errors = Vec::new();

    let loc_data = cache
        .loc_data()
        .map_err(|message| errors.push(message))
        .ok();
    let uber_state_data = cache
        .uber_state_data()
        .map_err(|message| errors.push(message))
        .ok();

    let (Some(loc_data), Some(uber_state_data)) = (loc_data, uber_state_data) else {
        return Err(CompileError {
            errors,
            logs: Vec::new(),
        });
    };

    let debug = debug.unwrap_or_default();
    let log_capture = LogCapture::new().with_max_level(max_log_level.unwrap_or_default().into());

    let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);

    let inline_snippets = InlineSnippets::new(snippets);
    let snippet_access = ChainedSnippetAccess::new(&inline_snippets, &cache.base);

    let mut compiler = Compiler::new(&mut rng, &snippet_access, loc_data, uber_state_data)
        .with_debug(debug)
        .with_lint(true)
        .with_log_capture(&log_capture);

    if let Err(err) = compiler.compile_snippet(&entry) {
        compiler.finish(); // errors will be empty - we failed to read the entry point

        return Err(CompileError {
            errors: vec![err],
            logs: log_capture.finish(),
        });
    }

    let compile::CompileResult { mut output, errors } = compiler.finish();

    let errors = errors
        .into_values()
        .flat_map(|(source, errors)| {
            errors
                .into_iter()
                .map(move |error| error.with_source(&source).to_string())
        })
        .collect::<Vec<_>>();

    if errors.is_empty() {
        let placeholder_map = postprocess(&mut [&mut output], loc_data, &mut rng)
            .pop()
            .unwrap();

        let seed = Seed::new(output, placeholder_map, debug).with_plando_attributes(
            PlandoAttributes::from_source(&inline_snippets.snippets[&entry].content),
        );

        let output = CompileOutput {
            seed: ciborium::Value::Bytes(seed.package_into_bytes()),
            logs: log_capture.finish(),
        };

        let mut bytes = vec![];
        ciborium::into_writer(&output, &mut bytes).unwrap();

        Ok(bytes)
    } else {
        Err(CompileError {
            errors,
            logs: log_capture.finish(),
        })
    }
}
