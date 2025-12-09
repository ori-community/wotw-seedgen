use std::io;

use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
};
use wotw_seedgen::assets::WatcherError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    Custom(String),
    #[error("failed to parse seedgen_info: {0}")]
    SeedgenInfo(serde_json::Error),
    #[error("failed to configure runtime: {0}")]
    BuildRuntime(io::Error),
    #[error("failed to start server: {0}")]
    Serve(io::Error),
    #[error(transparent)]
    Watcher(#[from] WatcherError),
    #[error("failed to reload assets: {0}")]
    ReloadAssets(String),
}

impl IntoResponse for Error {
    fn into_response(self) -> Response {
        let status = match self {
            Error::Custom(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Error::SeedgenInfo(_) => StatusCode::BAD_REQUEST,
            Error::BuildRuntime(_)
            | Error::Serve(_)
            | Error::Watcher(_)
            | Error::ReloadAssets(_) => unreachable!(),
        };

        (status, self.to_string()).into_response()
    }
}
