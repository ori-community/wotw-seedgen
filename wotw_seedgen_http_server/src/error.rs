use std::io;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error(transparent)]
    ServerCore(#[from] wotw_seedgen_server_shared::Error),
    #[error("{0}")]
    Custom(String),
    #[error("failed to parse seedgen_info: {0}")]
    SeedgenInfo(serde_json::Error),
    #[error("failed to start server: {0}")]
    Serve(io::Error),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SeedgenInfo(_) => StatusCode::BAD_REQUEST,
            Error::ServerCore(_) | Error::Serve(_) => unreachable!(),
        };

        (status, self.to_string()).into_response()
    }
}
