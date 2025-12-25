use std::{hash::BuildHasher, ops::Range};

use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use wotw_seedgen_parse::{Error, Identifier, Spanned};

use crate::seed_language::ast::{self, Handler, Traverse};

pub fn lint(ast: &ast::Snippet, errors: &mut Vec<Error>) {
    let mut unused = Unused::default();
    ast.traverse(&mut unused);
    unused.finish(errors);
}

#[derive(Default)]
struct Unused {
    defs: FxHashMap<u64, Range<usize>>,
    uses: FxHashSet<u64>,
}

impl Handler for Unused {
    fn identifier_def(&mut self, identifier: &Spanned<Identifier>) {
        self.defs.insert(
            FxBuildHasher.hash_one(identifier.data),
            identifier.span.clone(),
        );
    }

    fn identifier_use(&mut self, identifier: &Spanned<Identifier>) {
        self.uses.insert(FxBuildHasher.hash_one(identifier.data));
    }

    fn function_def(&mut self, identifier: &Spanned<Identifier>) {
        self.defs.insert(
            FxBuildHasher.hash_one(identifier.data),
            identifier.span.clone(),
        );
    }

    fn function_use(&mut self, identifier: &Spanned<Identifier>) {
        self.uses.insert(FxBuildHasher.hash_one(identifier.data));
    }
}

impl Unused {
    fn finish(self, errors: &mut Vec<Error>) {
        for (def, span) in self.defs {
            if !self.uses.contains(&def) {
                errors.push(Error::warning("unused value".to_string(), span));
            }
        }
    }
}
