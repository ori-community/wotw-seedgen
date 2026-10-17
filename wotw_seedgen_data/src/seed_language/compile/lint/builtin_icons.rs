use std::path::PathBuf;

use wotw_seedgen_parse::Error;

use crate::{
    assets::{self, EXECUTABLE_DIR},
    seed_language::ast::{get_command_arg_ref, get_command_args_ref, Command, Handler},
};

pub struct BuiltinIcons {
    assets_dir: Option<PathBuf>,
    errors: Vec<Error>,
}

impl BuiltinIcons {
    pub fn new() -> Self {
        Self {
            assets_dir: assets_dir(),
            errors: Vec::new(),
        }
    }

    pub fn finish(mut self, errors: &mut Vec<Error>) {
        errors.append(&mut self.errors);
    }
}

impl Handler for BuiltinIcons {
    fn command(&mut self, command: &Command) {
        let Some(assets_dir) = &self.assets_dir else {
            return;
        };

        let Command::BuiltinIcon(_, args) = command else {
            return;
        };

        let Some(args) = get_command_args_ref(args) else {
            return;
        };

        let Some(path) = get_command_arg_ref(&args.path) else {
            return;
        };

        if let Err(err) = assets::metadata(assets_dir.join(path.data)) {
            self.errors.push(Error::warning(err, path.span.clone()));
        }
    }
}

fn assets_dir() -> Option<PathBuf> {
    let assets_dir = EXECUTABLE_DIR.parent()?.join("client/assets");
    assets::metadata(&assets_dir).is_ok().then_some(assets_dir)
}
