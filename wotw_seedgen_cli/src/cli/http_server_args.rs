use clap::{Args, ValueHint};
use std::net::SocketAddr;

#[derive(Args)]
pub struct HttpServerArgs {
    /// Stop the server after a time of inactivity
    #[arg(short = 't', long, value_name = "DURATION", value_hint = ValueHint::Other)]
    pub inactivity_timeout: Option<humantime::Duration>,
    /// Socket address to listen on, e.g. 0.0.0.0:1234 or \[::\]:1234
    #[arg(short, long, value_hint = ValueHint::Other)]
    pub address: Option<SocketAddr>,
}
