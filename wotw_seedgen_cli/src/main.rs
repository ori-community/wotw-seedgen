mod cli;
mod files;
mod log_init;
mod plando;
mod seed;
mod stats;

use bugsalot::debugger;
use clap::Parser;
use cli::Cli;
use plando::plando;
use seed::seed;
use stats::stats;
use std::{
    env,
    fmt::{self, Debug},
};

fn main() -> Result<(), Error> {
    if env::var_os("ATTACH").is_some() {
        eprintln!("waiting for debugger...");
        debugger::wait_until_attached(None).unwrap();
    }

    let cli = Cli::parse();
    match cli {
        Cli::Seed { args } => seed(args),
        Cli::Plando { args } => plando(args),
        Cli::Stats { args } => stats(args),
    }
}

pub struct Error(String);
impl Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
impl<T: ToString> From<T> for Error {
    fn from(value: T) -> Self {
        Self(value.to_string())
    }
}
