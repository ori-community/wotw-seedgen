use axum::{Json, Router, routing::get};
use constcat::concat;
use serde::Serialize;
use strum::EnumMessage;
use utoipa::{OpenApi, ToSchema};
use wotw_seedgen::data::{Difficulty, Trick, VariantArray};

use crate::RouterState;

pub mod universe;
pub mod world;

pub const TAG: &str = "settings";
pub const SETTINGS: &str = concat!("/", TAG);

pub const DIFFICULTIES: &str = "/difficulties";
pub const TRICKS: &str = "/tricks";

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(DIFFICULTIES, get(difficulties))
        .route(TRICKS, get(tricks))
        .nest(world::WORLD, world::router())
        .nest(universe::UNIVERSE, universe::router())
}

#[derive(OpenApi)]
#[openapi(
    paths(difficulties, tricks),
    nest(
        (path = world::WORLD, api = world::Docs, tags = [TAG, world::TAG]),
        (path = universe::UNIVERSE, api = universe::Docs, tags = [TAG, universe::TAG]),
    )
)]
pub struct Docs;

/// Get the list of difficulties
#[utoipa::path(
    get,
    path = DIFFICULTIES,
    responses((status = OK, body = Vec<DifficultyInfo>)),
)]
async fn difficulties() -> Json<Vec<DifficultyInfo>> {
    Json(
        Difficulty::VARIANTS
            .iter()
            .copied()
            .map(DifficultyInfo::from)
            .collect(),
    )
}

#[derive(Serialize, ToSchema)]
pub struct DifficultyInfo {
    pub name: Difficulty,
    pub description: &'static str,
}

impl From<Difficulty> for DifficultyInfo {
    fn from(name: Difficulty) -> Self {
        let description = name.get_documentation().unwrap_or_default();

        Self { name, description }
    }
}

/// Get the list of tricks
#[utoipa::path(
    get,
    path = TRICKS,
    responses((status = OK, body = Vec<TrickInfo>)),
)]
async fn tricks() -> Json<Vec<TrickInfo>> {
    Json(
        Trick::VARIANTS
            .iter()
            .copied()
            .map(TrickInfo::from)
            .collect(),
    )
}

#[derive(Serialize, ToSchema)]
pub struct TrickInfo {
    pub name: Trick,
    pub description: &'static str,
    pub min_difficulty: Difficulty,
}

impl From<Trick> for TrickInfo {
    fn from(name: Trick) -> Self {
        let description = name.get_documentation().unwrap_or_default();
        let min_difficulty = name.min_difficulty();

        Self {
            name,
            description,
            min_difficulty,
        }
    }
}
