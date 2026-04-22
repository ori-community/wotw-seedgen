use std::{
    fmt::{Display, Write},
    num::NonZeroUsize,
};

use clap::builder::styling::Reset;
use dialoguer::{
    console::{self, Term},
    Confirm, Input, MultiSelect, Select,
};
use itertools::Itertools;
use rustc_hash::FxHashSet;
use strum::{Display, VariantArray, VariantNames};
use wotw_seedgen::data::{
    assets::{DefaultFileAccess, PresetAccess, UniversePresetSettings, WorldPresetSettings},
    seed_language::metadata::{ConfigDefault, ConfigValue},
    Difficulty, GreaterOneU8, Spawn, Trick, UniverseSettings, WorldSettings, DEFAULT_SPAWN,
};

use crate::{
    cli::{
        AvailablePreset, AvailableSnippet, AVAILABLE_SNIPPETS, AVAILABLE_UNIVERSE_PRESETS,
        AVAILABLE_WORLD_PRESETS, LITERAL,
    },
    Error,
};

pub fn seed_settings(settings: &mut UniversePresetSettings) -> Result<(), Error> {
    select_presets(
        "",
        "universe",
        &mut settings.includes,
        &AVAILABLE_UNIVERSE_PRESETS,
    )?;

    let world_settings = settings.world_settings.as_mut().unwrap();
    choose_world_count(world_settings)?;

    let mut include_settings = UniverseSettings::new(String::new());

    for identifier in settings.includes.iter().flatten() {
        DefaultFileAccess
            .universe_preset(identifier)?
            .apply(&mut include_settings, &DefaultFileAccess)?;
    }

    let multiworld = world_settings.len() > 1;

    for (index, (settings, include_settings)) in world_settings
        .iter_mut()
        .zip(&mut include_settings.world_settings)
        .enumerate()
    {
        let prefix = if multiworld {
            format!("[World {index}] ")
        } else {
            String::new()
        };

        seed_world_settings(prefix, settings, include_settings)?;
    }

    Ok(())
}

pub fn seed_world_settings(
    prefix: String,
    settings: &mut WorldPresetSettings,
    include_settings: &mut WorldSettings,
) -> Result<(), Error> {
    select_presets(
        &prefix,
        "world",
        &mut settings.includes,
        &AVAILABLE_WORLD_PRESETS,
    )?;

    for identifier in settings.includes.iter().flatten() {
        DefaultFileAccess
            .world_preset(identifier)?
            .apply(include_settings, &DefaultFileAccess)?;
    }

    choose_spawn(&prefix, settings, include_settings)?;
    select_difficulty(&prefix, settings, include_settings)?;
    select_tricks(&prefix, settings, include_settings)?;
    select_hard(&prefix, settings, include_settings)?;
    select_randomize_doors(&prefix, settings, include_settings)?;
    select_snippets(&prefix, settings, include_settings)?;
    select_snippet_config(&prefix, settings, include_settings)?;
    Ok(())
}

fn select_presets(
    prefix: &str,
    kind: &str,
    includes: &mut Option<FxHashSet<String>>,
    available_presets: &[AvailablePreset],
) -> Result<(), Error> {
    if !available_presets.is_empty() {
        let prompt = format!("{prefix}Select any number of {kind} presets");
        let items = sanitize_items(available_presets);

        let mut query = MultiSelect::new()
            .with_prompt(&prompt)
            .items(&items)
            .report(false);

        if let Some(includes) = &includes {
            query = query.defaults(
                &available_presets
                    .iter()
                    .map(|available_preset| includes.contains(&available_preset.identifier))
                    .collect::<Vec<_>>(),
            )
        }

        let selected = query
            .interact_opt()?
            .unwrap_or_default()
            .into_iter()
            .map(|index| available_presets[index].identifier.clone())
            .collect::<Vec<_>>();

        multi_select_custom_report(&prompt, &selected);

        if !selected.is_empty() {
            *includes = Some(selected.into_iter().collect());
        }
    }

    Ok(())
}

fn choose_world_count(world_settings: &mut Vec<WorldPresetSettings>) -> Result<(), Error> {
    if world_settings.len() == 1 {
        let world_count = Input::new()
            .with_prompt("Choose the number of worlds")
            .default(NonZeroUsize::new(1).unwrap())
            .interact_text()?
            .get();

        if world_count > 1 {
            *world_settings = vec![world_settings[0].clone(); world_count];
        }
    }
    Ok(())
}

fn choose_spawn(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    #[derive(Display, VariantArray)]
    #[strum(serialize_all = "title_case")]
    enum SpawnItems {
        Skip,
        Vanilla,
        Random,
        FullyRandom,
        Custom,
    }

    if settings.spawn.is_none() {
        let (default, default_anchor) = match &include_settings.spawn {
            Spawn::Set(anchor) => match anchor.as_str() {
                DEFAULT_SPAWN => (SpawnItems::Skip as usize, None),
                other => (SpawnItems::Custom as usize, Some(other)),
            },
            Spawn::Random => (SpawnItems::Random as usize, None),
            Spawn::FullyRandom => (SpawnItems::FullyRandom as usize, None),
        };

        if let Some(index) = Select::new()
            .with_prompt(format!("{prefix}Select a spawn location"))
            .items(SpawnItems::VARIANTS)
            .default(default)
            .interact_opt()?
        {
            settings.spawn = Some(match SpawnItems::VARIANTS[index] {
                SpawnItems::Skip => return Ok(()),
                SpawnItems::Vanilla => Spawn::default(),
                SpawnItems::Random => Spawn::Random,
                SpawnItems::FullyRandom => Spawn::FullyRandom,
                SpawnItems::Custom => {
                    let mut query = Input::new()
                        .with_prompt(format!("{prefix}Choose an areas.wotw identifier as spawn"));

                    if let Some(anchor) = default_anchor {
                        query = query.default(anchor.to_owned());
                    }

                    let identifier = query.interact_text()?;
                    Spawn::Set(identifier)
                }
            });
        }
    }

    Ok(())
}

fn select_difficulty(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    if settings.difficulty.is_none() {
        if let Some(index) = Select::new()
            .with_prompt(format!("{prefix}Select a difficulty"))
            .item("Skip")
            .items(<Difficulty as VariantNames>::VARIANTS)
            .default(include_settings.difficulty as usize + 1)
            .interact_opt()?
        {
            if index > 0 {
                settings.difficulty = Some(<Difficulty as VariantArray>::VARIANTS[index - 1]);
            }
        }
    }

    Ok(())
}

fn select_tricks(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    let mut include_controlled = 0;

    let items = <Trick as VariantArray>::VARIANTS
        .iter()
        .copied()
        .filter({
            let difficulty = settings.difficulty.unwrap_or(include_settings.difficulty);
            move |trick| difficulty >= trick.min_difficulty()
        })
        .filter(|trick| {
            let controlled = include_settings.tricks.contains(trick);
            include_controlled += controlled as u8;
            !controlled
        })
        .collect::<Vec<_>>();

    if !items.is_empty() {
        let mut prompt = format!("{prefix}Select any number of tricks");
        if include_controlled > 0 {
            let _ = write!(
                &mut prompt,
                " ({include_controlled} more controlled by included presets)"
            );
        }

        let mut query = MultiSelect::new().with_prompt(prompt).items(&items);

        if let Some(tricks) = &settings.tricks {
            query = query.defaults(
                &items
                    .iter()
                    .map(|trick| tricks.contains(trick))
                    .collect::<Vec<_>>(),
            )
        }

        let indices = query.interact_opt()?.unwrap_or_default();

        if !indices.is_empty() {
            settings.tricks = Some(indices.into_iter().map(|index| items[index]).collect())
        }
    }

    Ok(())
}

fn select_hard(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    if settings.hard.is_none() {
        if let Some(true) = Confirm::new()
            .with_prompt(format!(
                "{prefix}Choose whether the seed should assume hard in-game difficulty"
            ))
            .default(include_settings.hard)
            .interact_opt()?
        {
            settings.hard = Some(true);
        }
    }

    Ok(())
}

fn select_randomize_doors(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    if settings.randomize_doors.is_none() {
        if let Some(true) = Confirm::new()
            .with_prompt(format!(
                "{prefix}Choose whether door connections should be randomized"
            ))
            .default(include_settings.randomize_doors.is_some())
            .interact_opt()?
        {
            let loop_size = Input::new()
                .with_prompt("Choose the door loop size")
                .default(
                    include_settings
                        .randomize_doors
                        .unwrap_or(GreaterOneU8::new(2).unwrap()),
                )
                .interact_text()?;

            settings.randomize_doors = Some(loop_size);
        }
    }

    Ok(())
}

fn select_snippets(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    let mut include_controlled = 0;

    let available_snippets = AVAILABLE_SNIPPETS
        .iter()
        .filter(|snippet| {
            let controlled = include_settings.snippets.contains(&snippet.identifier);
            include_controlled += controlled as u32;
            !controlled
        })
        .collect::<Vec<_>>();

    if available_snippets.is_empty() {
        return Ok(());
    }

    let mut prompt = format!("{prefix}Select any number of snippets");
    if include_controlled > 0 {
        let _ = write!(
            &mut prompt,
            " ({include_controlled} more controlled by included presets)"
        );
    }

    let items = sanitize_items(&available_snippets);

    let mut query = MultiSelect::new()
        .with_prompt(&prompt)
        .items(&items)
        .report(false);

    if let Some(snippets) = &settings.snippets {
        query = query.defaults(
            &items
                .iter()
                .map(|identifier| snippets.contains(identifier))
                .collect::<Vec<_>>(),
        )
    }

    let selected = query
        .interact_opt()?
        .unwrap_or_default()
        .into_iter()
        .map(|index| available_snippets[index].identifier.clone())
        .collect::<Vec<_>>();

    multi_select_custom_report(&prompt, &selected);

    if !selected.is_empty() {
        settings.snippets = Some(selected);
    }

    Ok(())
}

fn select_snippet_config(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    let configurable_snippets = AVAILABLE_SNIPPETS
        .iter()
        .filter(|snippet| {
            !snippet.metadata.config.is_empty()
                && (include_settings.snippets.contains(&snippet.identifier)
                    || settings
                        .snippets
                        .as_ref()
                        .is_some_and(|snippets| snippets.contains(&snippet.identifier)))
        })
        .collect::<Vec<_>>();

    if configurable_snippets.is_empty() {
        return Ok(());
    }

    let items = sanitize_items(configurable_snippets.iter().map(|snippet| {
        format!(
            "{literal}{identifier}{reset}: {values}",
            literal = LITERAL.render(),
            identifier = snippet.identifier,
            reset = Reset.render(),
            values = snippet.metadata.config.keys().format(", ")
        )
    }));

    loop {
        let prompt = format!("{prefix}Select any snippet you want to configure");

        let selected = Select::new()
            .with_prompt(&prompt)
            .item("Finish")
            .default(0)
            .items(&items)
            .report(false)
            .interact_opt()?;

        looped_select_custom_report(&prompt, selected, |index| {
            &configurable_snippets[index - 1].identifier
        });

        let index = selected.unwrap_or_default();
        if index == 0 {
            break;
        } else {
            let snippet = &configurable_snippets[index - 1];
            select_snippet_config_value(prefix, settings, snippet, include_settings)?;
        }
    }

    Ok(())
}

fn select_snippet_config_value(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    snippet: &AvailableSnippet,
    include_settings: &WorldSettings,
) -> Result<(), Error> {
    let config = snippet.metadata.config.iter().collect::<Vec<_>>();

    let mut current_values = config
        .iter()
        .map(|(identifier, value)| {
            settings
                .snippet_config
                .as_ref()
                .and_then(|config| {
                    config
                        .get(&snippet.identifier)
                        .and_then(|snippet_config| snippet_config.get(*identifier))
                })
                .or_else(|| {
                    include_settings
                        .snippet_config
                        .get(&snippet.identifier)
                        .and_then(|snippet_config| snippet_config.get(*identifier))
                })
                .cloned()
                .unwrap_or_else(|| value.default.to_string())
        })
        .collect::<Vec<_>>();

    loop {
        let items = config
            .iter()
            .zip(&current_values)
            .map(|((identifier, value), current)| {
                format!(
                    "{literal}{identifier}{reset} [{current}]: {name}{description}",
                    literal = LITERAL.render(),
                    reset = Reset.render(),
                    name = value.name,
                    description = match &value.description {
                        None => "".to_string(),
                        Some(description) => format!(" ({description})"),
                    },
                )
            });
        let items = sanitize_items(items);

        let prompt = format!("{prefix}Select any configuration value to change");

        let selected = Select::new()
            .with_prompt(format!("{prefix}Select any configuration value to change"))
            .item("Finish")
            .default(0)
            .items(&items)
            .report(false)
            .interact_opt()?;

        looped_select_custom_report(&prompt, selected, |index| &config[index - 1].0);

        let index = selected.unwrap_or_default();
        if index == 0 {
            break;
        } else {
            let value = &config[index - 1];
            choose_snippet_config_value(
                prefix,
                settings,
                &snippet.identifier,
                value,
                &mut current_values[index - 1],
            )?;
        }
    }

    Ok(())
}

fn choose_snippet_config_value(
    prefix: &str,
    settings: &mut WorldPresetSettings,
    snippet: &str,
    value: &(&String, &ConfigValue),
    current: &mut String,
) -> Result<(), Error> {
    let prompt = format!(
        "{prefix}Choose a value for {identifier} ({name}{description})",
        identifier = value.0,
        name = value.1.name,
        description = match &value.1.description {
            None => "".to_string(),
            Some(description) => format!("; {description}"),
        }
    );

    let choice = match &value.1.default {
        ConfigDefault::Boolean(_) => Select::new()
            .with_prompt(prompt)
            .items(&[true, false])
            .default((current == "false") as usize)
            .interact_opt()?
            .map(|choice| (choice == 0).to_string())
            .unwrap_or_default(),
        _ => Input::new()
            .with_prompt(prompt)
            .allow_empty(true)
            .interact_text()?,
    };

    if !choice.is_empty() {
        *current = choice.clone();

        settings
            .snippet_config
            .get_or_insert_with(Default::default)
            .entry(snippet.to_string())
            .or_default()
            .insert(value.0.to_string(), choice);
    }

    Ok(())
}

fn sanitize_items<I, T>(items: I) -> Vec<String>
where
    I: IntoIterator<Item = T>,
    T: ToString,
{
    // 6 characters are taken by the selection interface
    let width = Term::stderr().size().1.saturating_sub(6) as usize;

    items
        .into_iter()
        .map(|item| {
            let summary = item.to_string().replace('\r', "").replace('\n', "; ");
            console::truncate_str(&summary, width, "...").to_string()
        })
        .collect()
}

fn multi_select_custom_report(prompt: &str, selected: &[String]) {
    eprintln!(
        "{prompt}: {}",
        selected
            .iter()
            .format_with(", ", |identifier, f| f(&format_args!(
                "{literal}{identifier}{reset}",
                literal = LITERAL.render(),
                reset = Reset.render(),
            )))
    );
}

fn looped_select_custom_report<F, D>(prompt: &str, selected: Option<usize>, get_identifier: F)
where
    F: FnOnce(usize) -> D,
    D: Display,
{
    eprint!("{prompt}: ");

    match selected {
        None => eprintln!(),
        Some(index) => {
            if index == 0 {
                eprintln!("Finish");
            } else {
                eprintln!(
                    "{literal}{identifier}{reset}",
                    literal = LITERAL.render(),
                    identifier = get_identifier(index),
                    reset = Reset.render(),
                );
            }
        }
    }
}
