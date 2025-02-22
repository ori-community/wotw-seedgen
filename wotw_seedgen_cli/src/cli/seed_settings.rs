use super::{INVALID, LINK, LITERAL};
use crate::{cli::interactive, files, Error};
use clap::{
    builder::{styling::Reset, PossibleValue, StringValueParser, TypedValueParser},
    error::ErrorKind,
    value_parser, Arg, ArgAction, ArgGroup, ArgMatches, Args, FromArgMatches,
};
use itertools::Itertools;
use lazy_static::lazy_static;
use std::{
    convert::Infallible,
    ffi::OsStr,
    fmt::{self, Debug, Display, Write},
    marker::PhantomData,
    num::{NonZeroU8, NonZeroUsize},
    str::FromStr,
};
use strum::VariantNames;
use wotw_seedgen::settings::{Difficulty, GreaterOneU8, Spawn, Trick};
use wotw_seedgen::{
    assets::{PresetInfo, UniversePresetSettings, WorldPresetSettings},
    settings::UniverseSettings,
};
use wotw_seedgen_assets::{FileAccess, PresetAccess, SnippetAccess};
use wotw_seedgen_seed_language::metadata::Metadata;

lazy_static! {
    pub static ref PRESET_ACCESS: FileAccess =
        files::preset_access("").unwrap_or_else(|_| FileAccess::new([""]));
    pub static ref AVAILABLE_UNIVERSE_PRESETS: Vec<AvailablePreset> = PRESET_ACCESS
        .available_universe_presets()
        .into_iter()
        .map(|identifier| {
            let info = PRESET_ACCESS
                .universe_preset(&identifier)
                .map(|preset| preset.info);
            AvailablePreset { identifier, info }
        })
        .collect();
    pub static ref AVAILABLE_WORLD_PRESETS: Vec<AvailablePreset> = PRESET_ACCESS
        .available_world_presets()
        .into_iter()
        .map(|identifier| {
            let info = PRESET_ACCESS
                .world_preset(&identifier)
                .map(|preset| preset.info);
            AvailablePreset { identifier, info }
        })
        .collect();
    pub static ref AVAILABLE_SNIPPETS: Vec<AvailableSnippet> = {
        let snippet_access =
            files::snippet_access("").unwrap_or_else(|_| FileAccess::new(["", "snippets"]));
        let mut available_snippets = snippet_access
            .available_snippets()
            .into_iter()
            .map(|identifier| {
                let metadata = snippet_access
                    .read_snippet(&identifier)
                    .map(|source| Metadata::from_source(&source.content))
                    .unwrap_or_default();
                AvailableSnippet {
                    identifier,
                    metadata,
                }
            })
            .filter(|available_snippet| !available_snippet.metadata.hidden)
            .collect::<Vec<_>>();
        available_snippets.sort_unstable_by(|a, b| {
            a.metadata
                .category
                .cmp(&b.metadata.category)
                .then_with(|| a.identifier.cmp(&b.identifier))
        });
        available_snippets
    };
}

#[derive(Debug, Clone)]
pub struct AvailablePreset {
    pub identifier: String,
    pub info: Result<Option<PresetInfo>, String>,
}
impl Display for AvailablePreset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = match &self.info {
            Ok(info) => match info {
                None => "(no details provided by preset)".to_string(),
                Some(info) => info
                    .description
                    .clone()
                    .unwrap_or_else(|| "(no description provided by preset)".to_string()),
            },
            Err(err) => format!("failed to read details: {err}"),
        };
        write!(
            f,
            "{literal}{identifier}{reset}: {description}",
            literal = LITERAL.render(),
            identifier = self.identifier,
            reset = Reset.render(),
        )
    }
}

#[derive(Debug, Clone)]
pub struct AvailableSnippet {
    pub identifier: String,
    pub metadata: Metadata,
}
impl Display for AvailableSnippet {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let description = self
            .metadata
            .description
            .as_ref()
            .map(String::as_str)
            .unwrap_or_else(|| "(no description provided by snippet)");
        write!(
            f,
            "{literal}{identifier}{reset}: {description}",
            literal = LITERAL.render(),
            identifier = self.identifier,
            reset = Reset.render(),
        )
    }
}

#[derive(Debug, Default)]
pub struct SeedSettings(pub UniversePresetSettings);

impl SeedSettings {
    pub fn into_universe_settings(self) -> Result<UniverseSettings, Error> {
        let mut settings = UniverseSettings::new("".to_string());
        self.0.apply(&mut settings, &files::preset_access("")?)?;
        Ok(settings)
    }
}

impl Args for SeedSettings {
    fn group_id() -> Option<clap::Id> {
        Some("seed_settings".into())
    }

    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.group(ArgGroup::new("seed_settings").multiple(true))
            .arg(interactive_arg())
            .arg(seed_arg())
            .arg(universe_presets_arg())
            .arg(worlds_arg())
            .arg(world_presets_arg(true))
            .arg(spawn_arg(true))
            .arg(difficulty_arg(true))
            .arg(tricks_arg(true))
            .arg(hard_arg(true))
            .arg(randomize_doors_arg(true))
            .arg(snippets_arg(true))
            .arg(snippet_config_arg(true))
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

#[derive(Debug, Default)]
pub struct SeedWorldSettings(pub WorldPresetSettings);

impl Args for SeedWorldSettings {
    fn group_id() -> Option<clap::Id> {
        Some("seed_settings".into())
    }

    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.group(ArgGroup::new("seed_settings").multiple(true))
            .arg(interactive_arg())
            .arg(world_presets_arg(false))
            .arg(spawn_arg(false))
            .arg(difficulty_arg(false))
            .arg(tricks_arg(false))
            .arg(hard_arg(false))
            .arg(randomize_doors_arg(false))
            .arg(snippets_arg(false))
            .arg(snippet_config_arg(false))
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

fn interactive_arg() -> Arg {
    Arg::new("interactive")
        .group("seed_settings")
        .long("interactive")
        .short('i')
        .value_name("BOOLEAN")
        .action(ArgAction::SetTrue)
        .help("Interactively choose settings")
}

fn seed_arg() -> Arg {
    Arg::new("seed")
        .group("seed_settings")
        .long("seed")
        .value_name("STRING")
        .help("The seed that determines all randomness")
        .long_help("Generating with the same seed on the same seedgen version should always output the same result")
}

fn universe_presets_arg() -> Arg {
    Arg::new("universe_presets")
        .group("seed_settings")
        .long("universe-presets")
        .short('P')
        .value_name("NAME")
        .num_args(1..)
        .help("Universe presets to include")
        .long_help(preset_help(&*AVAILABLE_UNIVERSE_PRESETS, "Universe"))
}

fn worlds_arg() -> Arg {
    Arg::new("worlds")
        .group("seed_settings")
        .long("worlds")
        .short('w')
        .value_name("NUMBER")
        .value_parser(value_parser!(NonZeroUsize))
        .default_value("1")
        .help("Number of worlds for multiworld")
        .long_help(format!(
            "By specifying a number greater than 0, you can generate seeds for the multiworld game mode\n\
            You can define different settings for all the worlds using the scoping syntax '{literal}<INDEX>: <ARGS>...{reset}'\n\
            For example, the following options define a three world seed where all worlds include the '{literal}rspawn{reset}'\n\
            preset, the first two worlds ('{literal}0{reset}' and '{literal}1{reset}') include the '{literal}moki{reset}' preset and the last world ('{literal}2{reset}') includes\n\
            the '{literal}gorlek{reset}' preset: '{literal}--worlds 3 --world-presets rspawn 0: moki 1: moki 2: gorlek{reset}'",
            literal = LITERAL.render(),
            reset = Reset.render(),
        ))
}

macro_rules! __choose_parser {
    ($arg:expr, $world_scoped:expr, $a:expr, $b:expr) => {
        if $world_scoped {
            $arg.value_parser($a).action(ArgAction::Append)
        } else {
            $arg.value_parser($b)
        }
    };
}

macro_rules! choose_parser {
    ($arg:expr, $world_scoped:expr, $ty:ty) => {
        __choose_parser!(
            $arg,
            $world_scoped,
            value_parser!(WorldScopedArg<$ty>),
            value_parser!($ty)
        )
    };
}

macro_rules! choose_strum_enum_parser {
    ($arg:expr, $world_scoped:expr, $ty:ty) => {
        __choose_parser!(
            $arg,
            $world_scoped,
            StrumEnumValueParser::<WorldScopedArg<$ty>>::new(),
            StrumEnumValueParser::<$ty>::new()
        )
    };
}

fn world_presets_arg(world_scoped: bool) -> Arg {
    let arg = Arg::new("world_presets")
        .group("seed_settings")
        .long("world-presets")
        .short('p')
        .value_name("NAME")
        .num_args(1..)
        .help("World presets to include")
        .long_help(preset_help(&*AVAILABLE_WORLD_PRESETS, "World"));
    choose_parser!(arg, world_scoped, String)
}

fn spawn_arg(world_scoped: bool) -> Arg {
    let mut arg = Arg::new("spawn")
        .group("seed_settings")
        .long("spawn")
        .short('S')
        .value_name("IDENTIFIER")
        .help("Spawn location")
        .long_help(format!(
            "Use any anchor identifier from areas.wotw to spawn on a specific location\n\
            Use '{literal}r{reset}' / '{literal}random{reset}' for a random teleporter or '{literal}f{reset}' / '{literal}fullyrandom{reset}' for a random anchor",
            literal = LITERAL.render(),
            reset = Reset.render(),
        ));
    if world_scoped {
        arg = arg.num_args(1..);
    }
    choose_parser!(arg, world_scoped, SpawnArg)
}

fn difficulty_arg(world_scoped: bool) -> Arg {
    let mut arg = Arg::new("difficulty")
        .group("seed_settings")
        .long("difficulty")
        .short('d')
        .value_name("DIFFICULTY")
        .help("Logically expected difficulty")
        .long_help(format!(
            "The logical difficulty to expect in a seed\n\
            This represents how demanding the required core movement should be\n\
            Difficulties don't include glitches by default, these are handled separately with '{literal}--tricks{reset}'\n\
            See the paths wiki page for more information: {link}https://wiki.orirando.com/seedgen/paths{reset}",
            literal = LITERAL.render(),
            link = LINK.render(),
            reset = Reset.render(),
        ));
    if world_scoped {
        arg = arg.num_args(1..);
    }
    choose_strum_enum_parser!(arg, world_scoped, Difficulty)
}

fn tricks_arg(world_scoped: bool) -> Arg {
    let arg = Arg::new("tricks")
        .group("seed_settings")
        .long("tricks")
        .short('t')
        .value_name("TRICK")
        .num_args(1..)
        .help("Logically expected tricks")
        .long_help(format!(
            "Tricks that can be logically required\n\
            This includes mostly Glitches but also other techniques that can be toggled for logic, such as damage boosting\n\
            See the paths wiki page for more information: {link}https://wiki.orirando.com/seedgen/paths{reset}",
            link = LINK.render(),
            reset = Reset.render(),
        )); // TODO don't think damage boost toggling is actually implemented yet
    choose_strum_enum_parser!(arg, world_scoped, Trick)
}

fn hard_arg(world_scoped: bool) -> Arg {
    let arg = Arg::new("hard")
        .group("seed_settings")
        .long("hard")
        .value_name("BOOLEAN")
        .help("Logically assume hard in-game difficulty")
        .long_help(
            "Logic will account for the player using the hard in-game difficulty, for instance by scaling damage requirements"
        );
    let arg = if world_scoped {
        arg.num_args(0..)
    } else {
        arg.action(ArgAction::SetTrue)
    };
    choose_parser!(arg, world_scoped, bool)
}

fn randomize_doors_arg(world_scoped: bool) -> Arg {
    let arg = Arg::new("randomize_doors")
        .group("seed_settings")
        .long("randomize-doors")
        .value_name("NUMBER")
        .default_missing_value("2")
        .help("Randomize door connections")
        .long_help("Randomize door connections. Provide a value to set the door loop size");
    let arg = if world_scoped {
        arg.num_args(0..)
    } else {
        arg.num_args(0..=1)
    };
    choose_parser!(arg, world_scoped, GreaterOneU8)
}

fn snippets_arg(world_scoped: bool) -> Arg {
    let arg = Arg::new("snippets")
        .group("seed_settings")
        .long("snippets")
        .short('s')
        .value_name("NAME")
        .num_args(1..)
        .help("Snippets to use")
        .long_help(snippets_help(&*AVAILABLE_SNIPPETS));
    choose_parser!(arg, world_scoped, String)
}

fn snippet_config_arg(world_scoped: bool) -> Arg {
    let arg = Arg::new("snippet_config")
        .group("seed_settings")
        .long("snippet_config")
        .short('c')
        .value_name("SNIPPET.CONFIG=VALUE")
        .num_args(1..)
        .help("Configuration to pass to snippets")
        .long_help(snippet_config_help(&*AVAILABLE_SNIPPETS));
    choose_parser!(arg, world_scoped, SnippetConfigArg)
}

fn preset_help(available_presets: &[AvailablePreset], kind: &str) -> String {
    let kind_lower = kind.to_ascii_lowercase();
    let mut help = format!(
        "{kind} presets define reusable settings to be applied to a {kind_lower}.\n\
        For details on how to create them, try '{literal}seedgen help {kind_lower}-preset{reset}'\n\
        All json files in the '{kind_lower}_presets' folder in the current working directory are available\n\n\
        Currently {count} {kind_lower} preset{plural} available",
        literal = LITERAL.render(),
        reset = Reset.render(),
        count = available_presets.len(),
        plural = if available_presets.len() == 1 { " is" } else { "s are" }
    );
    if !available_presets.is_empty() {
        write!(
            help,
            ":\n{}",
            available_presets
                .iter()
                .format_with("\n", |available_preset, f| {
                    f(&format_args!(
                        "    {}",
                        available_preset.to_string().replace('\n', "\n        ")
                    ))
                })
        )
        .unwrap();
    }
    help
}

fn snippets_help(available_snippets: &[AvailableSnippet]) -> String {
    let mut help = format!(
        "Snippets can modify seed generation in many ways.\n\
        All wotws files in the 'snippets' folder inside the current directory or seedgen's directory are available\n\
        See the official documentation for information on how to write your own snippets: https://docs.wotw.orirando.com/docs/seedlang\n\n\
        Currently {} snippet{} available",
        available_snippets.len(),
        if available_snippets.len() == 1 { " is" } else { "s are" }
    );
    if !available_snippets.is_empty() {
        write!(
            help,
            ":\n{}",
            available_snippets
                .iter()
                .chunk_by(|available_snippet| &available_snippet.metadata.category)
                .into_iter()
                .format_with("\n", |(category, snippets), f| {
                    let category = category
                        .as_ref()
                        .map(String::as_str)
                        .unwrap_or("No category");
                    f(&format_args!(
                        "    {category}:\n{}",
                        snippets.format_with("\n", |available_snippet, f| {
                            f(&format_args!(
                                "        {}",
                                available_snippet
                                    .to_string()
                                    .replace('\n', "\n            ")
                            ))
                        }),
                    ))
                })
        )
        .unwrap();
    }
    help
}

fn snippet_config_help(available_snippets: &[AvailableSnippet]) -> String {
    let mut help = format!(
        "Many snippets offer additional settings to customize their behaviour.\n\
        These will only have an effect if you use the respective snippet.\n\
        For instance, you can remove black market keystones, but keep black market ore by setting {literal}black_market.keystones=false{reset}",
        literal = LITERAL.render(),
        reset = Reset.render(),
    );
    if !available_snippets.is_empty() {
        let _ = write!(help, "\nCurrently these configurations are available:");

        for available_snippet in available_snippets {
            for (config_identifier, config_value) in &available_snippet.metadata.config {
                let _ = write!(
                    help,
                    "\n    {literal}{snippet_identifier}.{config_identifier}{reset}: {description} [default: {default}]",
                    literal = LITERAL.render(),
                    snippet_identifier = available_snippet.identifier,
                    reset = Reset.render(),
                    description = config_value.description,
                    default = config_value.default,
                );
            }
        }
    }
    help
}

#[derive(Clone)]
enum WorldScopedArg<T> {
    WorldScope(usize),
    Arg(T),
}
impl<T> FromStr for WorldScopedArg<T>
where
    T: FromStr,
    T::Err: Display,
{
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.strip_suffix(':') {
            None => s.parse().map(Self::Arg).map_err(|err| err.to_string()),
            Some(world_index) => world_index
                .parse()
                .map(Self::WorldScope)
                .map_err(|err| format!("invalid world index '{world_index}': {err}")),
        }
    }
}
impl<T: VariantNames> VariantNames for WorldScopedArg<T> {
    const VARIANTS: &'static [&'static str] = T::VARIANTS;
}

#[derive(Clone)]
struct SpawnArg(Spawn);
impl FromStr for SpawnArg {
    type Err = Infallible;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let spawn = match s {
            "r" | "random" => Spawn::Random,
            "f" | "fullyrandom" => Spawn::FullyRandom,
            other => Spawn::Set(other.to_string()),
        };
        Ok(Self(spawn))
    }
}

#[derive(Clone)]
struct SnippetConfigArg {
    snippet_identifier: String,
    config_identifier: String,
    config_value: String,
}
impl FromStr for SnippetConfigArg {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some((snippet_identifier, config)) = s.split_once('.') {
            if let Some((config_identifier, config_value)) = config.split_once('=') {
                return Ok(Self {
                    snippet_identifier: snippet_identifier.to_string(),
                    config_identifier: config_identifier.to_string(),
                    config_value: config_value.to_string(),
                });
            }
        }
        Err("value format is <snippet>.<config>=<value>")
    }
}

#[derive(Clone)]
struct StrumEnumValueParser<T>(PhantomData<T>);
impl<T> StrumEnumValueParser<T> {
    fn new() -> Self {
        Self(PhantomData)
    }
}
impl<T> TypedValueParser for StrumEnumValueParser<T>
where
    T: FromStr + VariantNames + Clone + Send + Sync + 'static,
    T::Err: Display,
{
    type Value = T;

    fn parse_ref(
        &self,
        cmd: &clap::Command,
        arg: Option<&Arg>,
        value: &OsStr,
    ) -> Result<Self::Value, clap::Error> {
        StringValueParser::new()
            .try_map(|s| s.parse::<T>().map_err(|err| err.to_string()))
            .parse_ref(cmd, arg, value)
    }

    fn possible_values(&self) -> Option<Box<dyn Iterator<Item = PossibleValue> + '_>> {
        Some(Box::new(T::VARIANTS.into_iter().map(PossibleValue::new)))
    }
}

impl FromArgMatches for SeedSettings {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let mut s = Self::default();
        s.update_from_arg_matches(matches)?;
        Ok(s)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        self.0.seed = matches.get_one("seed").cloned();
        self.0.includes = matches
            .get_many("universe_presets")
            .map(|values| values.cloned().collect());

        let mut world_settings = vec![
            WorldPresetSettings::default();
            matches.get_one::<NonZeroUsize>("worlds").unwrap().get()
        ];

        fn update_from_world_scoped_args<T, F>(
            matches: &ArgMatches,
            world_settings: &mut [WorldPresetSettings],
            id: &str,
            mut f: F,
        ) -> Result<(), clap::Error>
        where
            T: Clone + Send + Sync + 'static,
            F: FnMut(&mut WorldPresetSettings, &T),
        {
            if let Some(occurences) = matches.get_occurrences::<WorldScopedArg<T>>(id) {
                for occurence in occurences {
                    let mut world_scope = None;
                    for value in occurence {
                        update_from_world_scoped_occurence(
                            world_settings,
                            &mut world_scope,
                            value,
                            &mut f,
                        )?;
                    }
                }
            }

            Ok(())
        }
        fn update_from_world_scoped_flag<F>(
            matches: &ArgMatches,
            world_settings: &mut [WorldPresetSettings],
            id: &str,
            mut f: F,
        ) -> Result<(), clap::Error>
        where
            F: FnMut(&mut WorldPresetSettings, &bool),
        {
            if let Some(occurences) = matches.get_occurrences::<WorldScopedArg<bool>>(id) {
                for occurence in occurences {
                    let mut is_empty = true;
                    let mut world_scope = None;
                    for value in occurence {
                        is_empty = false;
                        update_from_world_scoped_occurence(
                            world_settings,
                            &mut world_scope,
                            value,
                            &mut f,
                        )?;
                    }
                    if is_empty {
                        for world in &mut *world_settings {
                            f(world, &true);
                        }
                    }
                }
            }

            Ok(())
        }
        fn update_from_world_scoped_occurence<T, F>(
            world_settings: &mut [WorldPresetSettings],
            world_scope: &mut Option<usize>,
            value: &WorldScopedArg<T>,
            mut f: F,
        ) -> Result<(), clap::Error>
        where
            T: Clone + Send + Sync + 'static,
            F: FnMut(&mut WorldPresetSettings, &T),
        {
            match value {
                WorldScopedArg::WorldScope(index) => *world_scope = Some(*index),
                WorldScopedArg::Arg(t) => match world_scope {
                    None => {
                        for world in &mut *world_settings {
                            f(world, t)
                        }
                    }
                    Some(index) => {
                        let world = world_settings.get_mut(*index).ok_or_else(|| {
                                    clap::Error::raw(
                                        ErrorKind::ValueValidation,
                                        format!(
                                            "world index '{invalid}{index}{reset}' exceeds number of worlds. Try '{literal}--worlds {worlds}{reset}' to generate multiple worlds",
                                            worlds = *index + 1,
                                            invalid = INVALID.render(),
                                            literal = LITERAL.render(),
                                            reset = Reset.render()
                                        ),
                                    )
                                })?;
                        f(world, t)
                    }
                },
            }
            Ok(())
        }

        update_from_world_scoped_args(
            matches,
            &mut world_settings,
            "world_presets",
            |world_preset, preset: &String| {
                world_preset
                    .includes
                    .get_or_insert_with(Default::default)
                    .insert(preset.to_string());
            },
        )?;
        update_from_world_scoped_args(
            matches,
            &mut world_settings,
            "spawn",
            |world_preset, spawn: &SpawnArg| world_preset.spawn = Some(spawn.0.clone()),
        )?; // TODO error on multiple to the same world?
        update_from_world_scoped_args(
            matches,
            &mut world_settings,
            "difficulty",
            |world_preset, difficulty: &Difficulty| world_preset.difficulty = Some(*difficulty),
        )?;
        update_from_world_scoped_args(
            matches,
            &mut world_settings,
            "tricks",
            |world_preset, trick: &Trick| {
                world_preset
                    .tricks
                    .get_or_insert_with(Default::default)
                    .insert(*trick);
            },
        )?;
        update_from_world_scoped_flag(
            matches,
            &mut world_settings,
            "hard",
            |world_preset, hard| world_preset.hard = Some(*hard),
        )?;
        update_from_world_scoped_args(
            matches,
            &mut world_settings,
            "randomize_doors",
            |world_preset, randomize_doors: &GreaterOneU8| {
                world_preset.randomize_doors = Some(*randomize_doors)
            },
        )?;
        update_from_world_scoped_args(
            matches,
            &mut world_settings,
            "snippets",
            |world_preset, snippet: &String| {
                world_preset
                    .snippets
                    .get_or_insert_with(Default::default)
                    .push(snippet.clone());
            },
        )?;
        update_from_world_scoped_args(
            matches,
            &mut world_settings,
            "snippet_config",
            |world_preset, snippet_config: &SnippetConfigArg| {
                world_preset
                    .snippet_config
                    .get_or_insert_with(Default::default)
                    .entry(snippet_config.snippet_identifier.clone())
                    .or_default()
                    .insert(
                        snippet_config.config_identifier.clone(),
                        snippet_config.config_value.clone(),
                    );
            },
        )?; // TODO validate snippet config? inspect available snippet configuration?

        self.0.world_settings = Some(world_settings);

        if matches.get_flag("interactive") {
            interactive::seed_settings(&mut self.0)?;
        }

        Ok(())
    }
}

impl FromArgMatches for SeedWorldSettings {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let mut s = Self::default();
        s.update_from_arg_matches(matches)?;
        Ok(s)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        self.0 = WorldPresetSettings {
            includes: matches
                .get_many("world_presets")
                .map(|world_presets| world_presets.cloned().collect()),
            spawn: matches.get_one("spawn").cloned(),
            difficulty: matches.get_one("difficulty").cloned(),
            tricks: matches
                .get_many("tricks")
                .map(|trick| trick.copied().collect()),
            hard: matches.get_flag("hard").then_some(true),
            randomize_doors: matches.get_one("randomize_doors").cloned(),
            snippets: matches
                .get_many("snippets")
                .map(|snippets| snippets.cloned().collect()),
            snippet_config: matches
                .get_many("snippet_config")
                .map(|snippet_config| snippet_config.cloned().collect()),
        };

        if matches.get_flag("interactive") {
            interactive::seed_world_settings(String::new(), &mut self.0)?;
        }

        Ok(())
    }
}

impl From<Error> for clap::Error {
    fn from(value: Error) -> Self {
        clap::Error::raw(
            ErrorKind::ValueValidation,
            format!("interactive session failed: {value:?}"),
        )
    }
}
