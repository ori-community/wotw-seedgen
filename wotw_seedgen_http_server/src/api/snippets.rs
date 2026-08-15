use axum::{Json, Router, extract::State, routing::get};
use constcat::concat;
use rustc_hash::FxHashMap;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};
use wotw_seedgen::data::seed_language::metadata::Metadata;

use crate::RouterState;

pub const TAG: &str = "snippets";
pub const SNIPPETS: &str = concat!("/", TAG);

const INFO: &str = "/info";

pub fn router() -> Router<RouterState> {
    Router::new().route(INFO, get(info))
}

#[derive(OpenApi)]
#[openapi(paths(info))]
pub struct Docs;

/// Get detailed info about available snippets
#[utoipa::path(
    get,
    path = INFO,
    responses((status = OK, body = FxHashMap<String, SnippetInfo>)),
)]
async fn info(State(cache): State<RouterState>) -> Json<FxHashMap<String, SnippetInfo>> {
    Json(cache.read().await.snippet_info.clone())
}

/// Selection of relevant information about a snippet
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct SnippetInfo {
    /// Where this snippet came from
    pub origin: SnippetOrigin,
    /// Metadata defined in the snippet
    pub metadata: Metadata,
}

/// Origin of a snippet
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub enum SnippetOrigin {
    /// This snippet was found within the seedgen executable's parent directory
    ExecutableDir,
    /// This snippet was found within the user data directory
    UserDataDir,
}
