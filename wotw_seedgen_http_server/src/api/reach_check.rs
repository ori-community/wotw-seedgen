use axum::{
    Json, Router,
    extract::State,
    routing::{get, post},
};
use constcat::concat;
use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};
use utoipa::{OpenApi, ToSchema, schema};
use wotw_seedgen::{
    data::{self, Position, UberIdentifier},
    seed::SeedgenInfo,
    seed_language::ast::Comparator,
};

use crate::{
    RouterState,
    api::schemas::{ComparatorSchema, MapIconSchema, PositionSchema, UberIdentifierSchema},
    error::{Error, Result},
    reach_check::reachable,
};

const TAG: &str = "reach-check";
const REACH_CHECK: &str = concat!("/", TAG);
const MAP_ICONS: &str = concat!(REACH_CHECK, "/map-icons");
const RELEVANT_UBER_STATES: &str = concat!(REACH_CHECK, "/relevant-uber-states");

pub fn router() -> Router<RouterState> {
    Router::new()
        .route(MAP_ICONS, get(map_icons))
        .route(RELEVANT_UBER_STATES, get(relevant_uber_states))
        .route(REACH_CHECK, post(reach_check))
}

#[derive(OpenApi)]
#[openapi(paths(reach_check, map_icons, relevant_uber_states))]
pub struct Docs;

/// Get the list of logically relevant map icons
#[utoipa::path(
    get,
    path = MAP_ICONS,
    responses((status = OK, body = MapIcons)),
)]
async fn map_icons(State(cache): State<RouterState>) -> Json<MapIcons> {
    Json(cache.read().await.map_icons.clone())
}

#[derive(Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MapIcons {
    /// List of logically relevant map icons
    pub map_icons: Vec<MapIcon>,
    /// Hash of `map_icons`
    pub hash: u64,
}

#[derive(Clone, Hash, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MapIcon {
    pub label: String,
    #[schema(value_type = MapIconSchema)]
    pub kind: data::MapIcon,
    #[schema(value_type = Vec<PositionSchema>)]
    pub positions: Vec<Position>,
    pub visible_if_any: Vec<MapIconCondition>,
}

#[derive(Clone, Hash, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct MapIconCondition {
    #[schema(value_type = UberIdentifierSchema)]
    pub uber_identifier: UberIdentifier,
    #[schema(value_type = ComparatorSchema)]
    pub comparator: Comparator,
    #[schema(value_type = f32)]
    pub value: OrderedFloat<f32>,
}

/// Get a list of logically relevant UberStates
#[utoipa::path(
    get,
    path = RELEVANT_UBER_STATES,
    responses((status = OK, body = RelevantUberStates)),
)]
async fn relevant_uber_states(State(cache): State<RouterState>) -> Json<RelevantUberStates> {
    Json(cache.read().await.relevant_uber_states.clone())
}

#[derive(Clone, Serialize, ToSchema)]
pub struct RelevantUberStates {
    /// List of logically relevant UberStates
    #[schema(value_type = Vec<UberIdentifierSchema>)]
    pub identifiers: Vec<UberIdentifier>,
    /// Hash of `identifiers`
    pub hash: u64,
}

/// Get a list of reachable nodes
#[utoipa::path(
    post,
    path = REACH_CHECK,
    responses((status = OK, body = ReachCheck)),
)]
async fn reach_check(
    State(cache): State<RouterState>,
    Json(body): Json<ReachCheckBody>,
) -> Result<Json<ReachCheck>> {
    let seedgen_info: SeedgenInfo =
        serde_json::from_str(&body.seedgen_info).map_err(Error::SeedgenInfo)?;

    let cache = cache.read().await;

    let reachable = reachable(&cache, body.uber_states, seedgen_info)?;

    Ok(Json(ReachCheck {
        reachable,
        map_icons_hash: cache.map_icons.hash,
        relevant_uber_states_hash: cache.relevant_uber_states.hash,
    }))
}

#[derive(Deserialize, ToSchema)]
pub struct ReachCheckBody {
    /// Current values of logically relevant UberStates
    #[schema(value_type = Vec<(UberIdentifierSchema, f32)>)]
    pub uber_states: Vec<(UberIdentifier, OrderedFloat<f32>)>,
    /// seedgen_info.json contents from within the seed
    pub seedgen_info: String,
}

#[derive(Serialize, ToSchema)]
struct ReachCheck {
    /// List of indices into logically reachable map icons
    reachable: Vec<usize>,
    /// Current hash of logically relevant map icons
    map_icons_hash: u64,
    /// Current hash of logically relevant UberStates
    relevant_uber_states_hash: u64,
}
