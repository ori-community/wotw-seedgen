use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use constcat::concat;
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema};
use wotw_seedgen::data::{UniverseSettings, assets::UniversePreset};

use crate::{
    RouterState,
    api::assets::AssetOrigin,
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
    responses((status = OK, body = FxHashMap<String, UniversePresetInfo>)),
)]
async fn list(State(cache): State<RouterState>) -> Json<FxHashMap<String, UniversePresetInfo>> {
    Json(cache.read().await.universe_preset_info.clone())
}

/// Information about a universe preset
#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct UniversePresetInfo {
    /// Where this universe preset came from
    pub origin: AssetOrigin,
    /// The universe preset
    pub content: UniversePreset,
}

/// Apply a universe preset to universe settings
#[utoipa::path(
    post,
    path = APPLY,
    responses(
        (status = OK, body = UniversePresetApplyBody),
        (status = UNPROCESSABLE_ENTITY, body = String),
    ),
)]
async fn apply(
    State(cache): State<RouterState>,
    Json(body): Json<UniversePresetApplyBody>,
) -> Result<Json<UniverseSettings>> {
    let cache = cache.read().await;

    let mut settings = match body.settings {
        UniversePresetApplyBodySettings::Full(full) => full.settings,
        UniversePresetApplyBodySettings::Seed(seed) => UniverseSettings::new(seed.seed),
    };

    for (index, preset) in body.presets.into_iter().enumerate() {
        preset
            .apply(&mut settings, &cache.base)
            .map_err(|err| Error::ApplyPreset(format!("at index {index}: {err}")))?;
    }
    Ok(Json(settings))
}

#[derive(Deserialize, ToSchema)]
pub struct UniversePresetApplyBody {
    /// Current settings
    #[serde(flatten)]
    pub settings: UniversePresetApplyBodySettings,
    /// Presets to apply
    pub presets: Vec<UniversePreset>,
}

#[derive(Deserialize, ToSchema)]
#[serde(untagged)]
pub enum UniversePresetApplyBodySettings {
    Full(UniversePresetApplyBodySettingsFull),
    Seed(UniversePresetApplyBodySettingsSeed),
}

#[derive(Deserialize, ToSchema)]
pub struct UniversePresetApplyBodySettingsFull {
    pub settings: UniverseSettings,
}

#[derive(Deserialize, ToSchema)]
pub struct UniversePresetApplyBodySettingsSeed {
    pub seed: String,
}
