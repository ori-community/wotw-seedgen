use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::RouterState;

pub mod logic;
pub mod presets;
pub mod settings;
pub mod snippets;

pub fn router(cache: RouterState) -> Router {
    Router::new()
        .nest(logic::LOGIC, logic::router())
        .nest(settings::SETTINGS, settings::router())
        .nest(presets::PRESETS, presets::router())
        .nest(snippets::SNIPPETS, snippets::router())
        .merge(SwaggerUi::new("/docs").url("/docs/wotw-seedgen-openapi.json", Docs::openapi()))
        .with_state(cache)
}

#[derive(OpenApi)]
#[openapi(nest(
    (path = logic::LOGIC, api = logic::Docs, tags = [logic::TAG]),
    (path = settings::SETTINGS, api = settings::Docs, tags = [settings::TAG]),
    (path = presets::PRESETS, api = presets::Docs, tags = [presets::TAG]),
    (path = snippets::SNIPPETS, api = snippets::Docs, tags = [snippets::TAG]),
))]
struct Docs;
