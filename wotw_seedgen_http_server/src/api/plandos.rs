use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use constcat::concat;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use utoipa::{IntoParams, OpenApi, ToSchema};
use wotw_seedgen::{data::parse::Source, seed::PlandoAttributes};

use crate::{
    RouterState,
    api::{LogLevelFilter, SchemaResult, assets::AssetOrigin},
    compile::{self, CompileError, CompileResult},
};

pub const TAG: &str = "plandos";
pub const PLANDO: &str = concat!("/", TAG);

const INFO: &str = "/info";
const COMPILE: &str = "/compile";

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(INFO, get(info))
        .route(COMPILE, post(compile))
}

#[derive(OpenApi)]
#[openapi(paths(info, compile))]
pub struct Docs;

/// Get detailed info about available plandos
#[utoipa::path(
    get,
    path = INFO,
    responses((status = OK, body = FxHashMap<String, SchemaResult<PlandoInfo, String>>)),
)]
async fn info(
    State(cache): State<RouterState>,
) -> Json<FxHashMap<String, SchemaResult<PlandoInfo, String>>> {
    Json(cache.read().await.plando_info.clone())
}

/// Information about a plando
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct PlandoInfo {
    /// Where this plando came from
    pub origin: AssetOrigin,
    /// Attributes specified by the plando author
    pub attributes: PlandoAttributes,
}

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
    Json(body): Json<CompileBody>,
) -> CompileResult {
    let cache = cache.read().await;

    compile::compile(query, body, cache)
}

#[derive(Deserialize, IntoParams)]
pub struct CompileQuery {
    pub debug: Option<bool>,
    pub max_log_level: Option<LogLevelFilter>,
}

#[derive(Deserialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct CompileBody {
    /// All necessary snippets for the compilation. Should include `entry` as key.
    pub snippets: FxHashMap<String, Source>,
    /// Entry point for compilation
    pub entry: String,
}
