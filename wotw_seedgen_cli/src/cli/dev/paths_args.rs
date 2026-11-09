use clap::{Args, ValueEnum};

#[derive(Args)]
pub struct PathsArgs {
    /// Which path should be displayed
    pub path: Option<SeedgenPath>,
}

#[derive(ValueEnum, Debug, Clone, PartialEq)]
pub enum SeedgenPath {
    Install,
    User,
    Logs,
}
