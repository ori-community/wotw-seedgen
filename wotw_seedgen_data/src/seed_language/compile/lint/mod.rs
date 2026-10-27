mod builtin_icons;
mod fix;
mod format_strings;
mod id_use;
mod identifier_use;
mod preferred_names;

pub use fix::{Fix, FixKind};

use crate::{
    assets::UberStateData,
    seed_language::{
        ast::{self, Traverse},
        compile::{
            lint::{
                builtin_icons::BuiltinIcons, format_strings::FormatStrings, id_use::IdUse,
                identifier_use::Unused, preferred_names::PreferredNames,
            },
            CompileErrors, SourceErrors,
        },
    },
};

pub fn lint_ast(
    ast: &ast::Snippet,
    uber_state_data: &UberStateData,
    errors: &mut SourceErrors,
    collect_fixes: bool,
) {
    let unused = Unused::default();
    let format_strings = FormatStrings::new(&mut errors.errors);
    let builtin_icons = BuiltinIcons::new();
    let preferred_names = PreferredNames::new(uber_state_data, collect_fixes);

    let mut handler = (unused, (format_strings, (builtin_icons, preferred_names)));
    ast.traverse(&mut handler);
    let (unused, (_, (builtin_icons, preferred_names))) = handler;

    unused.finish(&mut errors.errors);
    builtin_icons.finish(&mut errors.errors);
    preferred_names.finish(errors);
}

#[derive(Debug, Default)]
pub struct LintData {
    pub id_use: IdUse,
}

impl LintData {
    pub fn finish_snippet(&mut self, identifier: &str) {
        self.id_use.finish_snippet(identifier);
    }

    pub fn finish(self, errors: &mut CompileErrors) {
        self.id_use.finish(errors);
    }
}
