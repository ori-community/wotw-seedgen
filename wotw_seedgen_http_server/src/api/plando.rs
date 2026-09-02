use axum::{
    Json, Router,
    extract::{Query, State},
    routing::post,
};
use constcat::concat;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};
use wotw_seedgen::data::parse::Source;

use crate::{
    RouterState,
    api::LogLevelFilter,
    compile::{self, CompileError, CompileResult},
};

pub const TAG: &str = "plando";
pub const PLANDO: &str = concat!("/", TAG);

const COMPILE: &str = "/compile";

pub fn router() -> Router<RouterState> {
    Router::new().route(COMPILE, post(compile))
}

#[derive(OpenApi)]
#[openapi(paths(compile))]
pub struct Docs;

/// Compile a plandomizer
///
/// Response will be in CBOR format
///
/// ```cddl
/// output = {
///     seed: bstr,
///     logs: [ *record ],
/// }
///
/// record = {
///     level: level,
///     message: tstr
/// }
///
/// level = "ERROR" / "WARN" / "INFO" / "DEBUG" / "TRACE"
/// ```
#[utoipa::path(
    post,
    path = COMPILE,
    params(CompileQuery),
    responses(
        (status = OK, body = Vec<u8>),
        (status = UNPROCESSABLE_ENTITY, body = CompileError)
    ),
)]
async fn compile(
    State(cache): State<RouterState>,
    Query(query): Query<CompileQuery>,
    Json(body): Json<FxHashMap<String, Source>>,
) -> CompileResult {
    let cache = cache.read().await;

    compile::compile(query, body, cache)
}

#[derive(Deserialize, IntoParams)]
pub struct CompileQuery {
    pub debug: Option<bool>,
    pub max_log_level: Option<LogLevelFilter>,
}
