use axum::{Json, Router, extract::State, routing::get};
use constcat::concat;
use rustc_hash::FxHashMap;
use utoipa::{OpenApi};
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
    responses((status = OK, body = FxHashMap<String, Metadata>)),
)]
async fn info(State(cache): State<RouterState>) -> Json<FxHashMap<String, Metadata>> {
    Json(cache.read().await.snippet_info.clone())
}
