use std::io;

use wotw_seedgen_data::assets::WatcherError;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("failed to configure runtime: {0}")]
    BuildRuntime(io::Error),
    #[error(transparent)]
    Watcher(#[from] WatcherError),
    #[error("failed to load assets: {0}")]
    LoadAssets(String),
}
