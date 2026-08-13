use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use serde::Serialize;
use tokio::sync::RwLockReadGuard;
use wotw_seedgen::{
    data::{
        assets::{ChainedSnippetAccess, InlineSnippets},
        parse::Source,
        seed_language::{
            compile::{self, Compiler},
            output::postprocess,
        },
    },
    log_capture::{LogCapture, Record},
    seed::Seed,
};

use crate::{api::plando::CompileQuery, assets::Cache};

pub type CompileResult = Result<Vec<u8>, CompileError>;

pub struct CompileError(Vec<String>);

impl IntoResponse for CompileError {
    fn into_response(self) -> Response {
        (StatusCode::UNPROCESSABLE_ENTITY, Json(self.0)).into_response()
    }
}

#[derive(Serialize)]
pub struct CompileOutput {
    pub seed: ciborium::Value,
    pub logs: Vec<Record>,
}

pub fn compile(
    query: CompileQuery,
    snippets: FxHashMap<String, Source>,
    cache: RwLockReadGuard<Cache>,
) -> CompileResult {
    let CompileQuery {
        debug,
        max_log_level,
    } = query;

    let debug = debug.unwrap_or_default();
    let log_capture = LogCapture::new().with_max_level(max_log_level.unwrap_or_default().into());

    let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);

    let inline_snippets = InlineSnippets::new(snippets);
    let snippet_access = ChainedSnippetAccess::new(&inline_snippets, &cache.base);

    let mut compiler = Compiler::new(
        &mut rng,
        &snippet_access,
        &cache.base.loc_data,
        &cache.base.uber_state_data,
    )
    .with_debug(debug)
    .with_lint(true)
    .with_log_capture(&log_capture);

    for identifier in inline_snippets.keys() {
        compiler
            .compile_snippet(identifier)
            .map_err(|err| CompileError(vec![err]))?;
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
        let placeholder_map = postprocess(&mut [&mut output], &cache.base.loc_data, &mut rng)
            .pop()
            .unwrap();

        let seed = Seed::new(output, placeholder_map, debug);

        let output = CompileOutput {
            seed: ciborium::Value::Bytes(seed.package_into_bytes(!debug)),
            logs: log_capture.finish(),
        };

        let mut bytes = vec![];
        ciborium::into_writer(&output, &mut bytes).unwrap();

        Ok(bytes)
    } else {
        Err(CompileError(errors))
    }
}
