use axum::{Json, Router, routing::get};
use constcat::concat;
use utoipa::OpenApi;
use wotw_seedgen::settings::WorldSettings;

use crate::RouterState;

pub const TAG: &str = "world";
pub const WORLD: &str = concat!("/", TAG);

const DEFAULT: &str = "/default";

pub fn router() -> Router<RouterState> {
    Router::new().route(DEFAULT, get(default))
}

#[derive(OpenApi)]
#[openapi(paths(default))]
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
