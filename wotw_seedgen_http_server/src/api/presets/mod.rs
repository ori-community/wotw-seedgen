use axum::Router;
use constcat::concat;
use utoipa::OpenApi;

use crate::RouterState;

pub mod universe;
pub mod world;

pub const TAG: &str = "presets";
pub const PRESETS: &str = concat!("/", TAG);

pub fn router() -> Router<RouterState> {
    Router::new()
        .nest(world::WORLD, world::router())
        .nest(universe::UNIVERSE, universe::router())
}

#[derive(OpenApi)]
#[openapi(nest(
    (path = world::WORLD, api = world::Docs, tags = [TAG, world::TAG]),
    (path = universe::UNIVERSE, api = universe::Docs, tags = [TAG, universe::TAG]),
))]
pub struct Docs;
