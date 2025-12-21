use axum::{Json, Router, extract::State, routing::post};
use constcat::concat;
use utoipa::OpenApi;
use wotw_seedgen::data::UniverseSettings;

use crate::{RouterState, settings::inline_universe_snippets};

pub const TAG: &str = "universe";
pub const UNIVERSE: &str = concat!("/", TAG);

const INLINE_SNIPPETS: &str = "/inline-snippets";

pub fn router() -> Router<RouterState> {
    Router::new().route(INLINE_SNIPPETS, post(inline_snippets))
}

#[derive(OpenApi)]
#[openapi(paths(inline_snippets))]
pub struct Docs;

/// Inline all snippets originating from the data directory
#[utoipa::path(
    post,
    path = INLINE_SNIPPETS,
    responses((status = OK, body = UniverseSettings)),
)]
async fn inline_snippets(
    State(cache): State<RouterState>,
    Json(mut body): Json<UniverseSettings>,
) -> Json<UniverseSettings> {
    let cache = cache.read().await;

    inline_universe_snippets(&mut body, &cache);

    Json(body)
}
