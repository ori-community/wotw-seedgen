use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use constcat::concat;
use rustc_hash::FxHashMap;
use serde::Deserialize;
use utoipa::{OpenApi, ToSchema};
use wotw_seedgen::data::{UniverseSettings, assets::UniversePreset};

use crate::{
    RouterState,
    error::{Error, Result},
};

pub const TAG: &str = "universe";
pub const UNIVERSE: &str = concat!("/", TAG);

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

/// Get a list of available universe presets
#[utoipa::path(
    get,
    path = LIST,
    responses((status = OK, body = FxHashMap<String, UniversePreset>)),
)]
async fn list(State(cache): State<RouterState>) -> Json<FxHashMap<String, UniversePreset>> {
    Json(cache.read().await.base.universe_presets.clone())
}

/// Apply a universe preset to universe settings
#[utoipa::path(
    post,
    path = APPLY,
    responses(
        (status = OK, body = UniverseSettings),
        (status = UNPROCESSABLE_ENTITY, body = String),
    ),
)]
async fn apply(
    State(cache): State<RouterState>,
    Json(mut body): Json<ApplyBody>,
) -> Result<Json<UniverseSettings>> {
    let cache = cache.read().await;

    for (index, preset) in body.presets.into_iter().enumerate() {
        preset
            .apply(&mut body.settings, &cache.base)
            .map_err(|err| {
                Error::ApplyPreset(format!("failed to apply preset at index {index}: {err}"))
            })?;
    }

    Ok(Json(body.settings))
}

#[derive(Deserialize, ToSchema)]
pub struct ApplyBody {
    /// Current settings
    pub settings: UniverseSettings,
    /// Presets to apply
    pub presets: Vec<UniversePreset>,
}
