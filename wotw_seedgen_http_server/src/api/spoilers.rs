use axum::{Json, Router, routing::post};
use constcat::concat;
use utoipa::OpenApi;
use wotw_seedgen::spoiler::SeedSpoiler;

use crate::RouterState;

pub const TAG: &str = "spoilers";
pub const SPOILERS: &str = concat!("/", TAG);

const RENDER: &str = "/render";

pub fn router() -> Router<RouterState> {
    Router::new().route(RENDER, post(render))
}

#[derive(OpenApi)]
#[openapi(paths(render))]
pub struct Docs;

/// Render a JSON spoiler into plaintext form
#[utoipa::path(
    post,
    path = RENDER,
    responses((status = OK, body = String)),
)]
async fn render(Json(body): Json<SeedSpoiler>) -> String {
    body.to_string()
}
