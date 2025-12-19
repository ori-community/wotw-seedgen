use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use constcat::concat;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};
use wotw_seedgen::{assets::WorldPreset, settings::WorldSettings};

use crate::{
    RouterState,
    error::{Error, Result},
};

pub const TAG: &str = "world";
pub const WORLD: &str = concat!("/", TAG);

const LIST: &str = "/list";
const APPLY: &str = "/apply";

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(LIST, get(list))
        .route(APPLY, post(apply))
}

#[derive(OpenApi)]
#[openapi(paths(list, apply))]
pub struct Docs;

/// Get a list of available world presets
#[utoipa::path(
    get,
    path = LIST,
    responses((status = OK, body = FxHashMap<String, WorldPreset>)),
)]
async fn list(State(cache): State<RouterState>) -> Json<FxHashMap<String, WorldPreset>> {
    Json(cache.read().await.base.world_presets.clone())
}

/// Apply a world preset to world settings
#[utoipa::path(
    post,
    path = APPLY,
    responses(
        (status = OK, body = WorldSettings),
        (status = UNPROCESSABLE_ENTITY, body = String),
    ),
)]
async fn apply(
    State(cache): State<RouterState>,
    Json(mut body): Json<ApplyBody>,
) -> Result<Json<WorldSettings>> {
    let cache = cache.read().await;

    body.preset
        .apply(&mut body.settings, &cache.base)
        .map_err(Error::ApplyPreset)?;

    Ok(Json(body.settings))
}

#[derive(Deserialize, ToSchema)]
pub struct ApplyBody {
    /// Current settings
    pub settings: WorldSettings,
    /// Preset to apply
    pub preset: WorldPreset,
}
