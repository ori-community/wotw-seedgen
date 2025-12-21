use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use constcat::concat;
use rand::thread_rng;
use utoipa::OpenApi;
use wotw_seedgen::data::WorldSettings;

use crate::{RouterState, settings::inline_world_snippets};

pub const TAG: &str = "world";
pub const WORLD: &str = concat!("/", TAG);

const DEFAULT: &str = "/default";
const RANDOM: &str = "/random";
const INLINE_SNIPPETS: &str = "/inline-snippets";

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(DEFAULT, get(default))
        .route(RANDOM, get(random))
        .route(INLINE_SNIPPETS, post(inline_snippets))
}

#[derive(OpenApi)]
#[openapi(paths(default, random, inline_snippets))]
pub struct Docs;

/// Get the default world settings
#[utoipa::path(
    get,
    path = DEFAULT,
    responses((status = OK, body = WorldSettings)),
)]
async fn default() -> Json<WorldSettings> {
    Json(WorldSettings::default())
}

/// Get random world settings
#[utoipa::path(
    get,
    path = RANDOM,
    responses((status = OK, body = WorldSettings)),
)]
async fn random(State(cache): State<RouterState>) -> Json<WorldSettings> {
    let cache = cache.read().await;

    Json(WorldSettings::random(&mut thread_rng(), &*cache))
}

/// Inline all snippets originating from the data directory
#[utoipa::path(
    post,
    path = INLINE_SNIPPETS,
    responses((status = OK, body = WorldSettings)),
)]
async fn inline_snippets(
    State(cache): State<RouterState>,
    Json(mut body): Json<WorldSettings>,
) -> Json<WorldSettings> {
    let cache = cache.read().await;

    inline_world_snippets(&mut body, &cache);

    Json(body)
}
