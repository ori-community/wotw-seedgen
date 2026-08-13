use std::io::Cursor;

use crate::{cli::Cli, Error};
use clap::builder::PossibleValue;
use clap::{CommandFactory, ValueEnum};
use clap_complete::{Generator, Shell};
use clap_complete_nushell::Nushell;

#[derive(Clone)]
pub enum CompletionGenerator {
    Shell(Shell),
    Nushell,
}

impl ValueEnum for CompletionGenerator {
    fn value_variants<'a>() -> &'a [Self] {
        &[
            CompletionGenerator::Shell(Shell::Bash),
            CompletionGenerator::Shell(Shell::Elvish),
            CompletionGenerator::Shell(Shell::Fish),
            CompletionGenerator::Shell(Shell::PowerShell),
            CompletionGenerator::Shell(Shell::Zsh),
            CompletionGenerator::Nushell,
        ]
    }

    fn to_possible_value(&self) -> Option<PossibleValue> {
        match self {
            CompletionGenerator::Shell(shell) => shell.to_possible_value(),
            CompletionGenerator::Nushell => Some(PossibleValue::new("nushell")),
        }
    }
}

fn shell_completions_with<G: Generator>(generator: G) -> Result<(), Error> {
    let mut completions = Cursor::new(Vec::new());

    clap_complete::generate(
        generator,
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

pub fn shell_completions(shell: Option<CompletionGenerator>) -> Result<(), Error> {
    match shell {
        Some(CompletionGenerator::Shell(shell)) => shell_completions_with(shell),
        Some(CompletionGenerator::Nushell) => shell_completions_with(Nushell),
        None => {
            if std::env::var_os("NU_VERSION").is_some() {
                shell_completions_with(Nushell)
            } else {
                shell_completions_with(Shell::from_env().ok_or("Unknown shell")?)
            }
        }
    }
}
