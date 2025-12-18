use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::RouterState;

pub mod logic;
mod schemas;

pub fn router(cache: RouterState) -> Router {
    Router::new()
        .nest(logic::LOGIC, logic::router())
        .merge(SwaggerUi::new("/docs").url("/docs/wotw-seedgen-openapi.json", Docs::openapi()))
        .with_state(cache)
}

#[derive(OpenApi)]
#[openapi(nest(
    (path = logic::LOGIC, api = logic::Docs, tags = [logic::TAG]),
))]
struct Docs;
