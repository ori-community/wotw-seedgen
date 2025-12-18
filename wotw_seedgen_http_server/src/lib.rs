use std::{net::Ipv4Addr, sync::Arc, time::Duration};

use axum::Router;
use single_instance::SingleInstance;
use tokio::{
    net::TcpListener,
    sync::{RwLock, mpsc},
};
use wotw_seedgen::assets::DefaultFileAccess;

use crate::{
    assets::Cache,
    error::{Error, Result},
};

mod api;
mod assets;
mod error;
mod inactivity_timeout;
mod reach_check;

pub fn start(inactivity_timeout: Option<Duration>) -> Result<()> {
    let instance =
        SingleInstance::new("wotw-seedgen-http-server").map_err(Error::SingleInstance)?;

    if !instance.is_single() {
        eprintln!("server already seems to be running, exiting");

        return Ok(());
    }

    let cache = Cache::new(DefaultFileAccess)
        .map_err(|err| Error::ServerCore(wotw_seedgen_server_shared::Error::LoadAssets(err)))?;

    let (mut runtime, cache) = wotw_seedgen_server_shared::start(cache)?;

    let mut router = api::router(cache);

    match inactivity_timeout {
        None => runtime.block_on(serve(router)),
        Some(duration) => {
            let (inactive_send, mut inactive_recv) = mpsc::unbounded_channel();

            router = inactivity_timeout::init(router, &mut runtime, duration, inactive_send);

            runtime.spawn(serve(router));

            inactive_recv.blocking_recv();

            eprintln!("inactivity deadline reached, exiting");

            Ok(())
        }
    }
}

type RouterState = Arc<RwLock<Cache>>;

async fn serve(router: Router) -> Result<()> {
    axum::serve(listener().await, router)
        .await
        .map_err(Error::Serve)
}

async fn listener() -> TcpListener {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 51413))
        .await
        .unwrap()
}
