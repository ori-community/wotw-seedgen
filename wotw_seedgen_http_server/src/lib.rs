use std::{net::Ipv4Addr, sync::Arc};

use single_instance::SingleInstance;
use tokio::{net::TcpListener, sync::RwLock};
use wotw_seedgen::assets::DefaultFileAccess;

use crate::{
    assets::Cache,
    error::{Error, Result},
};

mod api;
mod assets;
mod error;
mod reach_check;

pub fn start() -> Result<()> {
    let instance =
        SingleInstance::new("wotw-seedgen-http-server").map_err(Error::SingleInstance)?;

    if !instance.is_single() {
        eprintln!("server already seems to be running, exiting");

        return Ok(());
    }

    let cache = Cache::new(DefaultFileAccess)
        .map_err(|err| Error::ServerCore(wotw_seedgen_server_shared::Error::LoadAssets(err)))?;

    let (runtime, state) = wotw_seedgen_server_shared::start(cache)?;

    runtime
        .block_on(async { axum::serve(listener().await, api::router(state)).await })
        .map_err(Error::Serve)?;

    Ok(())
}

type RouterState = Arc<RwLock<Cache>>;

async fn listener() -> TcpListener {
    TcpListener::bind((Ipv4Addr::LOCALHOST, 51413))
        .await
        .unwrap()
}
