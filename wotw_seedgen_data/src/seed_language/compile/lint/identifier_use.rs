use std::{hash::BuildHasher, ops::Range};

use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashSet};
use wotw_seedgen_parse::{Error, Identifier, Spanned};

use crate::seed_language::ast::Handler;

#[derive(Default)]
pub struct Unused {
    defs: IndexMap<u64, Range<usize>, FxBuildHasher>,
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
    pub fn finish(self, errors: &mut Vec<Error>) {
        for (def, span) in self.defs {
            if !self.uses.contains(&def) {
                errors.push(Error::warning("unused value".to_string(), span));
            }
        }
    }
}
