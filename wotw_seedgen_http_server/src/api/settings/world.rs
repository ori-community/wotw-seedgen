use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use constcat::concat;
use utoipa::OpenApi;
use wotw_seedgen::data::WorldSettings;

use crate::RouterState;

pub const TAG: &str = "world";
pub const WORLD: &str = concat!("/", TAG);

const DEFAULT: &str = "/default";
const RANDOM: &str = "/random";
const CHECK: &str = "/check";

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(DEFAULT, get(default))
        .route(RANDOM, get(random))
        .route(CHECK, post(check))
}

#[derive(OpenApi)]
#[openapi(paths(default, random, check))]
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

    Json(WorldSettings::random(&mut rand::rng(), &*cache))
}

/// Check world settings
#[utoipa::path(
    post,
    path = CHECK,
    responses(
        (status = OK),
        (status = UNPROCESSABLE_ENTITY, body = String),
    ),
)]
async fn check(Json(body): Json<WorldSettings>) {
    let _ = body; // If we don't pretend to use the Json extractor above, utoipa will misbehave :)
}
