mod cli;
mod files;
mod plando;
mod seed;
mod stats;

use clap::Parser;
use cli::Cli;
use plando::plando;
use seed::seed;
use stats::stats;
use std::fmt::{self, Debug};

fn main() -> Result<(), Error> {
    let cli = Cli::parse();
    match cli {
        Cli::Seed { settings } => seed(settings.0),
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
