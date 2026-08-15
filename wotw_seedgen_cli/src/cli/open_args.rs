use clap::{Args, ValueEnum};

#[derive(Args)]
pub struct OpenArgs {
    /// Which directory to open
    pub directory: OpenDirectory,
}

#[derive(Clone, ValueEnum)]
pub enum OpenDirectory {
    /// Open the install directory
    ///
    /// Next to seedgen, it contains all the installed assets. It's recommended you don't modify these by hand,
    /// instead you can override files by placing modified copies into the user-data directory.
    Install,
    /// Open the user data directory
    ///
    /// It contains generated seeds as well as any custom assets you may have.
    UserData,
    /// Open the logs directory
    ///
    /// If you run commands with --verbose, this will have a seedgen_log.txt
    Logs,
}
