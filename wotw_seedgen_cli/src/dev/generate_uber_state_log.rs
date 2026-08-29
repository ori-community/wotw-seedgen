use std::{fmt::Write, time::Instant};

use rand::thread_rng;
use wotw_seedgen::{
    data::{
        assets::{
            self, AssetFileAccess, DefaultFileAccess, InlineSnippets, UberStateData,
            SEEDGEN_USER_DATA_DIR,
        },
        parse::Source,
        seed_language::{compile::Compiler, output::PlaceholderMap},
    },
    seed::Seed,
};

use crate::{cli::LaunchArgs, seed::launch_seed, Error};

pub fn generate_uber_state_log(args: LaunchArgs) -> Result<(), Error> {
    let start = Instant::now();

    let mut source = String::new();

    let uber_state_dump = DefaultFileAccess.uber_state_dump().unwrap();

    for (group_id, group) in &uber_state_dump.groups {
        for (state_id, state) in &group.states {
            if matches!(
                (group_id, state_id),
                (8 | 9 | 10 | 12 | 14 | 8246 | 33399, _)
                    | (5, 40 | 41)
                    | (30, 1 | 2)
                    | (3440, 37811)
            ) || state.readonly
                || !state.uber_state_type.starts_with("Serialized")
            {
                continue;
            }

            write!(
                &mut source,
                "on change {group_id}|{state_id} item_message(\"#{group_name}.{state_name}# ({group_id}|{state_id}) -> \" + {group_id}|{state_id})",
                group_name = group.name,
                state_name = state.name
            ).unwrap();
        }
    }

    let snippets = InlineSnippets::from_iter([(
        String::new(),
        Source::new("uber_state_log".to_string(), source),
    )]);

    let loc_data = DefaultFileAccess.loc_data()?;
    let state_data = DefaultFileAccess.state_data()?;
    let uber_state_data = UberStateData::from_parts(uber_state_dump, &loc_data, &state_data);

    let mut compiler = Compiler::new(&mut thread_rng(), &snippets, &loc_data, &uber_state_data);

    compiler.compile_snippet("")?;

    let output = compiler
        .finish()
        .eprint_errors()
        .ok_or("failed to compile uber state log")?;

    let seed = Seed::new(output, PlaceholderMap::default(), false);

    let out = SEEDGEN_USER_DATA_DIR.join("uber_state_log.wotwr");
    let mut file = assets::file_create(&out)?;
    seed.package(&mut file)?;

    launch_seed(&out, args)?;

    eprintln!(
        "generated in {:.2}s to \"{}\"",
        start.elapsed().as_secs_f32(),
        out.display()
    );

    Ok(())
}
