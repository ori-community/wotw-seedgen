use std::{
    net::Ipv4Addr,
    sync::Arc,
    time::{Duration, Instant},
};

use tokio::{net::TcpListener, sync::RwLock};
use wotw_seedgen::assets::{DefaultFileAccess, Watcher};

use crate::{
    assets::{Cache, watch_assets},
    error::{Error, Result},
};

mod api;
mod assets;
mod error;
mod reach_check;

pub fn start() -> Result<()> {
    let start = Instant::now();

    let runtime = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .map_err(Error::BuildRuntime)?;

    eprintln!("Started runtime");

    let cache = Cache::new(DefaultFileAccess).map_err(Error::Custom)?; // TODO better error

    let mut watcher = Watcher::new(Duration::from_secs(1))?;

    cache.watch(&mut watcher)?;

    let state = Arc::new(RwLock::new(cache));
    runtime.spawn(watch_assets(state.clone(), watcher));

    eprintln!("Loaded assets");

    eprintln!("Server ready in {:.2}s", start.elapsed().as_secs_f32());

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
