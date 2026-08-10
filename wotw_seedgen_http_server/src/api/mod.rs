use axum::response::IntoResponse;
use axum::{
    Json, Router,
    extract::{Query, State},
    routing::post,
};
use serde::{Deserialize, Serialize};
use tower_http::cors::{Any, CorsLayer};
use utoipa::{IntoParams, OpenApi, ToSchema, openapi};
use utoipa_swagger_ui::SwaggerUi;
use wotw_seedgen::data::UniverseSettings;
use wotw_seedgen::data::seed_language::output::{CommandZone, Trigger};

use crate::{RouterState, error::Result, generate};

pub mod logic;
pub mod presets;
pub mod settings;
pub mod snippets;
pub mod spoilers;

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
        .nest(spoilers::SPOILERS, spoilers::router())
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
        (path = spoilers::SPOILERS, api = spoilers::Docs, tags = [spoilers::TAG]),
    ),
    // manually add things here that get missed by automatic collection
    components(schemas(
        // Trigger is missed because for no_recursion it makes more sense to consider it high in the hierarchy
        // but in current practice it's a relatively low leaf of spoiler and gets filtered
        Trigger,
        // CommandZone is missed because it's currently only referenced inside an operation which is forced to
        // add no_recursion indiscriminately because it's impossible to add the attribute on specific generic
        // types passed to it
        CommandZone,
        // LogLevelFilter is missed because oriShrug
        LogLevelFilter
    )),
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
            ]
            .into_iter()
            .flatten()
            {
                operation.operation_id = None;
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
///     logs: [ *record ],
/// }
///
/// record = {
///     level: level,
///     message: tstr
/// }
///
/// level = "ERROR" / "WARN" / "INFO" / "DEBUG" / "TRACE"
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
    pub max_log_level: Option<LogLevelFilter>,
}

#[derive(Default, Serialize, Deserialize, ToSchema)]
#[serde(rename_all = "UPPERCASE")]
#[schema(default = LogLevelFilter::default)]
pub enum LogLevelFilter {
    Off,
    Error,
    Warn,
    #[default]
    Info,
    Debug,
    Trace,
}

impl From<LogLevelFilter> for log::LevelFilter {
    fn from(value: LogLevelFilter) -> Self {
        match value {
            LogLevelFilter::Off => log::LevelFilter::Off,
            LogLevelFilter::Error => log::LevelFilter::Error,
            LogLevelFilter::Warn => log::LevelFilter::Warn,
            LogLevelFilter::Info => log::LevelFilter::Info,
            LogLevelFilter::Debug => log::LevelFilter::Debug,
            LogLevelFilter::Trace => log::LevelFilter::Trace,
        }
    }
}
