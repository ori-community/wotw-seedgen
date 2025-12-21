use axum::{
    Json, Router,
    extract::{Query, State},
    routing::post,
};
use serde::Deserialize;
use utoipa::{IntoParams, OpenApi};
use utoipa_swagger_ui::SwaggerUi;
use wotw_seedgen::data::UniverseSettings;

use crate::{RouterState, error::Result, generate};

pub mod logic;
pub mod presets;
pub mod settings;
pub mod snippets;

const GENERATE: &str = "/generate";

pub fn router(cache: RouterState) -> Router {
    Router::new()
        .route(GENERATE, post(generate))
        .nest(logic::LOGIC, logic::router())
        .nest(settings::SETTINGS, settings::router())
        .nest(presets::PRESETS, presets::router())
        .nest(snippets::SNIPPETS, snippets::router())
        .merge(SwaggerUi::new("/docs").url("/docs/wotw-seedgen-openapi.json", Docs::openapi()))
        .with_state(cache)
}

#[derive(OpenApi)]
#[openapi(
    paths(generate),
    nest(
        (path = logic::LOGIC, api = logic::Docs, tags = [logic::TAG]),
        (path = settings::SETTINGS, api = settings::Docs, tags = [settings::TAG]),
        (path = presets::PRESETS, api = presets::Docs, tags = [presets::TAG]),
        (path = snippets::SNIPPETS, api = snippets::Docs, tags = [snippets::TAG]),
    )
)]
struct Docs;

/// Generate a seed
///
/// Response will be in CBOR format
///
/// ```cddl
/// universe = {
///     worlds: [ +bstr ],
///     ? json_spoiler: tstr,
///     ? text_spoiler: tstr,
/// }
/// ```
#[utoipa::path(
    post,
    path = GENERATE,
    params(GenerateQuery),
    responses(
        (status = OK, body = Vec<u8>),
        (status = INTERNAL_SERVER_ERROR, body = String),
    ),
)]
async fn generate(
    State(cache): State<RouterState>,
    Query(query): Query<GenerateQuery>,
    Json(body): Json<UniverseSettings>,
) -> Result<Vec<u8>> {
    let cache = cache.read().await;

    generate::generate(query, &body, cache)
}

#[derive(Deserialize, IntoParams)]
pub struct GenerateQuery {
    pub json_spoiler: Option<bool>,
    pub text_spoiler: Option<bool>,
}
