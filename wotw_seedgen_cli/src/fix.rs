use std::{
    io,
    path::{Path, PathBuf},
};

use rustc_hash::FxHashMap;
use wotw_seedgen::data::{
    assets::{
        self, ChainedSnippetAccess, DefaultFileAccess, InlineSnippets, KnownAssetAccess, Pickups,
        SnippetAccess, UberStateData,
    },
    parse::Source,
    seed_language::compile::Compiler,
};

use crate::{cli::FixArgs, Error};

pub fn fix(args: FixArgs) -> Result<(), Error> {
    let FixArgs { paths } = args;

    let mut fixer = Fixer::new()?;
    fixer.fix_paths(paths)?;

    fixer.finish();

    Ok(())
}

struct Fixer {
    pickups: Pickups,
    uber_state_data: UberStateData,
    fix_count: u32,
}

impl Fixer {
    fn new() -> Result<Self, String> {
        let pickups = DefaultFileAccess.pickups()?;
        let states = DefaultFileAccess.states()?;
        let uber_state_data = DefaultFileAccess.uber_state_data(&pickups, &states)?;

        Ok(Self {
            pickups,
            uber_state_data,
            fix_count: 0,
        })
    }

    fn fix_paths(&mut self, paths: Vec<PathBuf>) -> Result<(), Error> {
        for path in paths {
            let content = assets::read_to_string(&path)?;
            let source = Source::new(path.to_string_lossy().to_string(), content);

            let path_access =
                InlineSnippets::new(FxHashMap::from_iter([("/".to_string(), source)]));
            let snippet_access = ChainedSnippetAccess::new(&path_access, &DefaultFileAccess);

            self.fix("/", &path, &snippet_access)?;
        }

        Ok(())
    }

    fn fix<S>(&mut self, identifier: &str, path: &Path, snippet_access: &S) -> Result<(), Error>
    where
        S: SnippetAccess,
    {
        let mut compiler = Compiler::new(
            &mut rand::rng(),
            snippet_access,
            &self.pickups,
            &self.uber_state_data,
        )
        .with_fixes(true);

        compiler.compile_snippet(identifier)?;

        let mut result = compiler.finish();

        let (mut source, mut errors) = result.errors.inner.remove(identifier).unwrap();

        {
            let mut stderr = io::stderr().lock();
            for error in errors.errors {
                let _ = error.write_pretty(&source, &mut stderr);
            }
        }

        errors.fixes.sort_unstable_by_key(|fix| fix.span.start);

        let mut last_end = 0;
        let mut offset = 0;

        for fix in errors.fixes {
            let overlap = fix.span.start < last_end;
            last_end = fix.span.end;

            if overlap {
                eprintln!("Cannot apply overlapping fix");
                continue;
            }

            let resulting_offset = fix.resulting_offset();

            let offset_span = (fix.span.start as isize + offset) as usize
                ..(fix.span.end as isize + offset) as usize;
            source.content.replace_range(offset_span, &fix.content);
            offset += resulting_offset;

            self.fix_count += 1;
        }

        assets::write(path, &source.content)?;

        Ok(())
    }

    fn finish(self) {
        if self.fix_count == 0 {
            eprintln!("No automatic fixes discovered");
        } else {
            eprintln!(
                "Applied {fix_count} fix{es}",
                fix_count = self.fix_count,
                es = if self.fix_count == 1 { "" } else { "es" },
            );
        }
    }
}
