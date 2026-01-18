use std::{net::Ipv4Addr, sync::Arc, time::Duration};
use std::net::SocketAddr;
use axum::Router;
use single_instance::SingleInstance;
use socket2::{Domain, Protocol, Socket, Type};
use tokio::{
    net::TcpListener,
    sync::{RwLock, mpsc},
};
use wotw_seedgen::data::assets::DefaultFileAccess;

use crate::{
    assets::Cache,
    error::{Error, Result},
};

mod api;
mod assets;
mod error;
mod generate;
mod inactivity_timeout;
mod logic;
mod settings;

pub fn start(inactivity_timeout: Option<Duration>, address: Option<SocketAddr>) -> Result<()> {
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
        None => runtime.block_on(serve(router, address)),
        Some(duration) => {
            let (inactive_send, mut inactive_recv) = mpsc::unbounded_channel();

            router = inactivity_timeout::init(router, &mut runtime, duration, inactive_send);

            runtime.spawn(serve(router, address));

            inactive_recv.blocking_recv();

            eprintln!("inactivity deadline reached, exiting");

            Ok(())
        }
    }
}

type RouterState = Arc<RwLock<Cache>>;

async fn serve(router: Router, address: Option<SocketAddr>) -> Result<()> {
    axum::serve(listener(address).await, router)
        .await
        .map_err(Error::Serve)
}

async fn listener(address: Option<SocketAddr>) -> TcpListener {
    let address = address.unwrap_or_else(|| SocketAddr::from((Ipv4Addr::LOCALHOST, 51413)));

    let socket = Socket::new(Domain::for_address(address), Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket.set_nonblocking(true).unwrap();

    // Explicitly allow IPv6-mapped IPv4 addresses
    // e.g. access on 127.0.0.1:1234 when listening on [::]:1234
    socket.set_only_v6(false).unwrap();

    socket.bind(&address.into()).unwrap();
    socket.listen(1024).unwrap();

    let listener = TcpListener::from_std(socket.into()).unwrap();

    eprintln!("Listening on {address}");

    listener
}
