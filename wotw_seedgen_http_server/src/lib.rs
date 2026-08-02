use axum::Router;
use log::LevelFilter;
use single_instance::SingleInstance;
use socket2::{Domain, Protocol, Socket, Type};
use std::net::SocketAddr;
use std::process::exit;
use std::{net::Ipv4Addr, sync::Arc, time::Duration};
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
        exit(2);
    }

    let cache = Cache::new(DefaultFileAccess)
        .map_err(|err| Error::ServerCore(wotw_seedgen_server_shared::Error::LoadAssets(err)))?;

    let (mut runtime, cache) = wotw_seedgen_server_shared::start(cache)?;

    let mut router = api::router(cache);

    // Turn the global filter off because we use local filters instead
    log::set_max_level(LevelFilter::Trace);

    match inactivity_timeout {
        None => runtime.block_on(serve(router, address)),
        Some(duration) => {
            let (inactive_send, mut inactive_recv) = mpsc::unbounded_channel();

            router = inactivity_timeout::init(router, &mut runtime, duration, inactive_send);

            runtime.spawn(serve(router, address));

            inactive_recv.blocking_recv();

            runtime.shutdown_background();

            eprintln!("inactivity deadline reached, exiting");

            Ok(())
        }
    }
}

type RouterState = Arc<RwLock<Cache>>;

async fn serve(router: Router, address: Option<SocketAddr>) -> Result<()> {
    axum::serve(listener(address), router)
        .await
        .map_err(Error::Serve)
}

fn listener(address: Option<SocketAddr>) -> TcpListener {
    let address = address.unwrap_or_else(|| SocketAddr::from((Ipv4Addr::LOCALHOST, 51413)));

    let domain = Domain::for_address(address);
    let socket = Socket::new(domain, Type::STREAM, Some(Protocol::TCP)).unwrap();
    socket.set_nonblocking(true).unwrap();

    if domain == Domain::IPV6 {
        // Explicitly allow IPv6-mapped IPv4 addresses
        // e.g. access on 127.0.0.1:1234 when listening on [::]:1234
        socket.set_only_v6(false).unwrap();
    }

    socket.bind(&address.into()).unwrap();
    socket.listen(1024).unwrap();

    let listener = TcpListener::from_std(socket.into()).unwrap();

    eprintln!("Listening on {address}");

    listener
}
