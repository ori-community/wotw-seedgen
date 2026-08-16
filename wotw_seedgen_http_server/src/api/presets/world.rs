use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use constcat::concat;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use wotw_seedgen::data::{WorldSettings, assets::WorldPreset};

use crate::{
    RouterState,
    api::assets::AssetOrigin,
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
    responses((status = OK, body = FxHashMap<String, WorldPresetInfo>)),
)]
async fn list(State(cache): State<RouterState>) -> Json<FxHashMap<String, WorldPresetInfo>> {
    Json(cache.read().await.world_preset_info.clone())
}

/// Information about a world preset
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct WorldPresetInfo {
    /// Where this world preset came from
    pub origin: AssetOrigin,
    /// The world preset
    pub content: WorldPreset,
}

/// Apply world presets to world settings.
/// If no settings are given, presets are applied on top of the default world settings.
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
    Json(body): Json<WorldPresetApplyBody>,
) -> Result<Json<WorldSettings>> {
    let cache = cache.read().await;

    let mut settings = body.settings.unwrap_or_default();

    for (index, preset) in body.presets.into_iter().enumerate() {
        preset
            .apply(&mut settings, &cache.base)
            .map_err(|err| Error::ApplyPreset(format!("at index {index}: {err}")))?;
    }

    Ok(Json(settings))
}

#[derive(Deserialize, ToSchema)]
pub struct WorldPresetApplyBody {
    /// World settings to apply presets on.
    /// Omit to use default world settings.
    pub settings: Option<WorldSettings>,
    /// Presets to apply
    pub presets: Vec<WorldPreset>,
}
