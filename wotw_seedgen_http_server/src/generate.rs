use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde::Serialize;
use tokio::sync::RwLockReadGuard;
use utoipa::ToSchema;
use wotw_seedgen::{data::UniverseSettings, log_capture::Record};

use crate::{api::GenerateQuery, assets::Cache};

#[derive(Serialize)]
pub struct Universe {
    pub worlds: Vec<ciborium::Value>,
    pub json_spoiler: Option<String>,
    pub text_spoiler: Option<String>,
    pub logs: Vec<Record>,
}

pub fn generate(
    query: GenerateQuery,
    settings: &UniverseSettings,
    cache: RwLockReadGuard<Cache>,
) -> GenerateResult<Vec<u8>> {
    let GenerateQuery {
        json_spoiler,
        text_spoiler,
        max_log_level,
    } = query;

    let max_log_level = max_log_level.unwrap_or_default().into();

    let (universe, logs) = cache.generate(settings, max_log_level)?;

    let worlds = universe
        .worlds
        .into_iter()
        .map(|seed| ciborium::Value::Bytes(seed.package_into_bytes()))
        .collect::<Vec<_>>();

    let json_spoiler = json_spoiler
        .unwrap_or_default()
        .then(|| serde_json::to_string(&universe.spoiler).unwrap());
    let text_spoiler = text_spoiler
        .unwrap_or_default()
        .then(|| universe.spoiler.to_string());

    let universe = Universe {
        worlds,
        json_spoiler,
        text_spoiler,
        logs,
    };

    let mut bytes = vec![];
    ciborium::into_writer(&universe, &mut bytes).unwrap();

    Ok(bytes)
}

pub type GenerateResult<T> = Result<T, GenerateError>;

#[derive(Serialize, ToSchema)]
pub struct GenerateError {
    pub message: String,
    pub logs: Vec<Record>,
}

impl GenerateError {
    pub fn new(message: String, logs: Vec<Record>) -> Self {
        Self { message, logs }
    }
}

impl IntoResponse for GenerateError {
    fn into_response(self) -> Response {
        (StatusCode::INTERNAL_SERVER_ERROR, Json(self)).into_response()
    }
}
