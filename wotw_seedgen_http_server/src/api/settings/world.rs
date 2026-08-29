use axum::{Json, Router, extract::State, routing::get};
use constcat::concat;
use rand::thread_rng;
use utoipa::OpenApi;
use wotw_seedgen::data::WorldSettings;

use crate::RouterState;

pub const TAG: &str = "world";
pub const WORLD: &str = concat!("/", TAG);

const DEFAULT: &str = "/default";
const RANDOM: &str = "/random";

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(DEFAULT, get(default))
        .route(RANDOM, get(random))
}

#[derive(OpenApi)]
#[openapi(paths(default, random))]
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
