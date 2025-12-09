use axum::Router;
use utoipa::OpenApi;
use utoipa_swagger_ui::SwaggerUi;

use crate::RouterState;

pub mod reach_check;
mod schemas;

pub fn router(state: RouterState) -> Router {
    Router::new()
        .merge(reach_check::router())
        .merge(SwaggerUi::new("/docs").url(
            "/docs/wotw-seedgen-openapi.json",
            reach_check::Docs::openapi(),
        ))
        .with_state(state)
}
