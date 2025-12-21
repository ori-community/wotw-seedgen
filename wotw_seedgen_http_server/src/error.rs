use std::io;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use single_instance::error::SingleInstanceError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ServerCore(#[from] wotw_seedgen_server_shared::Error),
    #[error("failed to enforce single instance: {0}")]
    SingleInstance(SingleInstanceError),
    #[error("{0}")]
    Custom(String),
    #[error("failed to parse seedgen_info: {0}")]
    SeedgenInfo(serde_json::Error),
    #[error("failed to start server: {0}")]
    Serve(io::Error),
    #[error("failed to apply preset: {0}")]
    ApplyPreset(String),
    #[error("failed to generate seed: {0}")]
    Generate(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match &self {
            Error::Custom(_) | Error::Generate(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SeedgenInfo(error) => {
                if error.is_data() {
                    StatusCode::UNPROCESSABLE_ENTITY
                } else {
                    StatusCode::BAD_REQUEST
                }
            }
            Error::ApplyPreset(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Error::ServerCore(_) | Error::SingleInstance(_) | Error::Serve(_) => unreachable!(),
        };

        (status, self.to_string()).into_response()
    }
}
