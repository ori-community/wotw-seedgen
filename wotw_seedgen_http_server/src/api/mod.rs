use axum::http::Response;
use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::post,
};
use serde::Deserialize;
use tower_http::cors::{Any, CorsLayer};
use utoipa::{IntoParams, OpenApi, openapi};
use utoipa_swagger_ui::SwaggerUi;
use wotw_seedgen::data::UniverseSettings;

use crate::{RouterState, error::Result, generate};

pub mod logic;
pub mod presets;
pub mod settings;
pub mod snippets;

const GENERATE: &str = "/generate";

pub fn router(cache: RouterState) -> Router {
    let cors = CorsLayer::new()
        .allow_headers(Any)
        .allow_methods(Any)
        .allow_origin(Any);

    Router::new()
        .route(GENERATE, post(generate))
        .nest(logic::LOGIC, logic::router())
        .nest(settings::SETTINGS, settings::router())
        .nest(presets::PRESETS, presets::router())
        .nest(snippets::SNIPPETS, snippets::router())
        .layer(cors)
        .merge(SwaggerUi::new("/docs").url(
            "/docs/wotw-seedgen-openapi.json",
            Docs::openapi_no_operation_ids(),
        ))
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

impl Docs {
    /// `utoipa` [does not support `None` as `operation_id`][utoipa-attributes], rather it will default to the function name.
    /// That, however, creates duplicate ids which is [forbidden by OpenAPI][operationid].
    /// To resolve the issue, we purge all operation ids after `utoipa` is done generating.
    ///
    /// [utoipa-attributes]: https://docs.rs/utoipa/latest/utoipa/attr.path.html#path-attributes
    /// [operationid]: https://swagger.io/docs/specification/v3_0/paths-and-operations/#operationid
    fn openapi_no_operation_ids() -> openapi::OpenApi {
        let mut openapi = Self::openapi();

        for path in openapi.paths.paths.values_mut() {
            for operation in [
                &mut path.get,
                &mut path.put,
                &mut path.post,
                &mut path.delete,
                &mut path.options,
                &mut path.head,
                &mut path.patch,
                &mut path.trace,
            ] {
                if let Some(operation) = operation {
                    operation.operation_id = None;
                }
            }
        }

        openapi
    }
}

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
) -> Result<impl IntoResponse> {
    let cache = cache.read().await;

    Ok((
        [("Content-Type", "application/cbor")],
        generate::generate(query, &body, cache)?,
    ))
}

#[derive(Deserialize, IntoParams)]
pub struct GenerateQuery {
    pub json_spoiler: Option<bool>,
    pub text_spoiler: Option<bool>,
}
