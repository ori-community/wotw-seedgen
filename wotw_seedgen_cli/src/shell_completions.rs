use std::io::Cursor;

use clap::CommandFactory;
use clap_complete::Shell;

use crate::{cli::Cli, Error};

pub fn shell_completions(shell: Option<Shell>) -> Result<(), Error> {
    let shell = shell.or_else(Shell::from_env).ok_or("Unknown shell")?;

    let mut completions = Cursor::new(Vec::new());
    clap_complete::generate(
        shell,
        &mut Cli::command(),
        // workaround for https://github.com/clap-rs/clap/issues/6421
        "_PLACEHOLDER_BIN_NAME",
        &mut completions,
    );

    let completions = String::from_utf8(completions.into_inner())
        .unwrap()
        .replace("_PLACEHOLDER_BIN_NAME", "wotw-seedgen");

    print!("{completions}");

    Ok(())
}
