use std::net::SocketAddr;
use clap::Args;

#[derive(Args)]
pub struct HttpServerArgs {
    /// Stop the server after a time of inactivity
    #[arg(short = 't', long)]
    pub inactivity_timeout: Option<humantime::Duration>,
    /// Socket address to listen on, e.g. 0.0.0.0:1234 or \[::\]:1234
    #[arg(short, long)]
    pub address: Option<SocketAddr>,
}
