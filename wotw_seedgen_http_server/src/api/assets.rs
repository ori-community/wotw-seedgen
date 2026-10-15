use axum::{
    Router,
    extract::{State, WebSocketUpgrade},
    response::Response,
    routing::any,
};
use constcat::concat;
use futures_util::StreamExt;
use serde::Serialize;
use utoipa::{OpenApi, ToSchema};

use crate::RouterState;

pub const TAG: &str = "assets";
pub const ASSETS: &str = concat!("/", TAG);

const WATCH: &str = "/watch";

pub fn router() -> Router<RouterState> {
    Router::new().route(WATCH, any(watch))
}

#[derive(OpenApi)]
#[openapi(paths(watch))]
pub struct Docs;

/// Receive Websocket messages when assets used by seedgen change
#[utoipa::path(
    get,
    path = WATCH,
    responses(
        (status = SWITCHING_PROTOCOLS),
        (status = BAD_REQUEST, body = String),
    ),
)]
async fn watch(ws: WebSocketUpgrade, State(cache): State<RouterState>) -> Response {
    ws.on_upgrade(async move |socket| {
        let (sender, mut receiver) = socket.split();

        {
            let watchers = &cache.read().await.values.watchers;
            let mut sockets = watchers.as_ref().unwrap().sockets.lock().await;
            sockets.push(sender);
        }

        while receiver.next().await.is_some() {}

        let watchers = &cache.read().await.values.watchers;
        let mut sockets = watchers.as_ref().unwrap().sockets.lock().await;
        if let Some(index) = sockets
            .iter()
            .position(|sender| sender.is_pair_of(&receiver))
        {
            drop(sockets.swap_remove(index));
        }
    })
}

/// Origin of an asset
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(tag = "kind", rename_all_fields = "camelCase")]
pub enum AssetOrigin {
    /// This asset was found within the seedgen executable's parent directory
    ExecutableDir,
    /// This asset was found within the user data directory
    UserDataDir { path: String },
}
