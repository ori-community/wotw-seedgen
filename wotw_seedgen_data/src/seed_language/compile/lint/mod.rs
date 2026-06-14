mod id_use;
mod identifier_use;

use rustc_hash::FxHashMap;
use wotw_seedgen_parse::{Error, Source};

use crate::seed_language::{
    ast::{self, Traverse},
    compile::lint::{id_use::IdUse, identifier_use::Unused},
};

pub fn lint_ast(ast: &ast::Snippet, errors: &mut Vec<Error>) {
    let mut unused = Unused::default();
    ast.traverse(&mut unused);
    unused.finish(errors);
}

#[derive(Debug, Default)]
pub struct LintData {
    pub id_use: IdUse,
}

impl LintData {
    pub fn finish_snippet(&mut self, identifier: &str) {
        self.id_use.finish_snippet(identifier);
    }

    pub fn finish(self, errors: &mut FxHashMap<String, (Source, Vec<Error>)>) {
        self.id_use.finish(errors);
    }
}
