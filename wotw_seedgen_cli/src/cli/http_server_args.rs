use clap::Args;

#[derive(Args)]
pub struct HttpServerArgs {
    /// Stop the server after a time of inactivity
    #[arg(short = 't', long)]
    pub inactivity_timeout: Option<humantime::Duration>,
}
