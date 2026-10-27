use std::{fmt::Write, time::Instant};

use wotw_seedgen::{
    data::{
        assets::{
            self, DefaultFileAccess, InlineSnippets, KnownAssetAccess, UberStateData,
            UberStateNameEntry, SEEDGEN_USER_DATA_DIR,
        },
        parse::Source,
        seed_language::{compile::Compiler, output::PlaceholderMap},
        UberIdentifier,
    },
    seed::{PlandoAttributes, Seed},
};

use crate::{cli::LaunchArgs, seed::launch_seed, Error};

pub fn generate_uber_state_log(args: LaunchArgs) -> Result<(), Error> {
    let start = Instant::now();

    let mut source = concat!(
        "fun uber_state_log_message(message: String) {\n",
        "    item_message(message)\n",
        "    debug_log(message)\n",
        "}\n",
        "\n",
    )
    .to_string();

    let pickups = DefaultFileAccess.pickups()?;
    let states = DefaultFileAccess.states()?;
    let uber_state_dump = DefaultFileAccess.uber_state_dump()?;
    let uber_state_data = DefaultFileAccess.uber_state_data(&pickups, &states)?;

    for (&group_id, group) in &uber_state_dump.groups {
        for (&member_id, state) in &group.states {
            if matches!(
                (group_id, member_id),
                (8 | 9 | 10 | 12 | 14 | 8246 | 33399, _)
                    | (5, 40 | 41)
                    | (30, 1 | 2)
                    | (3440, 37811)
            ) || state.r#type.is_readonly()
                || !state.r#type.is_serialized()
            {
                continue;
            }

            let entry = &uber_state_data.id_lookup[&UberIdentifier::new(group_id, member_id)];

            match &entry.rando_name {
                None => {
                    let (group, member) = entry.name.split_once('.').unwrap();
                    let UberStateNameEntry::Vanilla(ids) = &uber_state_data.name_lookup[group][member] else {continue};

                    if ids.len() == 1 {
                        writeln!(
                            &mut source,
                            "on change {name} uber_state_log_message(\"#{name}# ({group_id}|{member_id}) -> \" + {name})",
                            name = entry.name,
                        )?;
                    } else {
                        writeln!(
                            &mut source,
                            "on change {group_id}|{member_id} uber_state_log_message(\"#{name}# ({group_id}|{member_id}) -> \" + {group_id}|{member_id})",
                            name = entry.name,
                        )?;
                    }
                },
                Some(rando_name) => writeln!(
                    &mut source,
                    "on change {rando_name} uber_state_log_message(\"#{rando_name}# / #{name}# ({group_id}|{member_id}) -> \" + {rando_name})",
                    name = entry.name,
                )?,
            }
        }
    }

    assets::write(SEEDGEN_USER_DATA_DIR.join("uber_state_log.wotws"), &source)?;

    let snippets = InlineSnippets::from_iter([(
        String::new(),
        Source::new("uber_state_log".to_string(), source),
    )]);

    let pickups = DefaultFileAccess.pickups()?;
    let states = DefaultFileAccess.states()?;
    let uber_state_data = UberStateData::from_parts(uber_state_dump, &pickups, &states);

    let mut compiler = Compiler::new(&mut rand::rng(), &snippets, &pickups, &uber_state_data);

    compiler.compile_snippet("")?;

    let output = compiler
        .finish()
        .eprint_errors()
        .ok_or("failed to compile uber state log")?;

    let seed = Seed::new(output, PlaceholderMap::default(), false).with_plando_attributes(
        PlandoAttributes {
            name: Some("UberState Log".to_string()),
            description: Some("Prints item messages and debug logs for every UberState change. Noisy states are ignored".to_string())
        },
    );

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
