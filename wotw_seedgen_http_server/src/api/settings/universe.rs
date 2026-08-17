use axum::{
    Json, Router,
    extract::{Query, State},
    routing::{get, post},
};
use constcat::concat;
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};
use wotw_seedgen::data::UniverseSettings;

use crate::{RouterState, error::Result, settings::inline_universe_snippets};

pub const TAG: &str = "universe";
pub const UNIVERSE: &str = concat!("/", TAG);

const NEW: &str = "/new";
const INLINE_SNIPPETS: &str = "/inline-snippets";

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(NEW, get(new))
        .route(INLINE_SNIPPETS, post(inline_snippets))
}

#[derive(OpenApi)]
#[openapi(paths(new, inline_snippets))]
pub struct Docs;

/// Start new universe settings
#[utoipa::path(
    get,
    path = NEW,
    params(NewQuery),
    responses((status = OK, body = UniverseSettings)),
)]
async fn new(Query(query): Query<NewQuery>) -> Json<UniverseSettings> {
    Json(UniverseSettings::new(query.seed))
}

#[derive(Deserialize, IntoParams)]
pub struct NewQuery {
    pub seed: String,
}

/// Inline all snippets originating from the data directory
#[utoipa::path(
    post,
    path = INLINE_SNIPPETS,
    responses(
        (status = OK, body = UniverseSettings),
        (status = UNPROCESSABLE_ENTITY, body = String),
    ),
)]
async fn inline_snippets(
    State(cache): State<RouterState>,
    Json(mut body): Json<UniverseSettings>,
) -> Result<Json<UniverseSettings>> {
    let cache = cache.read().await;

    inline_universe_snippets(&mut body, &cache)?;

    Ok(Json(body))
}
