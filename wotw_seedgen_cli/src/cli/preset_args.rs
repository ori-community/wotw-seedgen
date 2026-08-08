use clap::{
    builder::styling::Reset, error::ErrorKind, Arg, ArgAction, ArgGroup, ArgMatches, Args,
    FromArgMatches,
};
use wotw_seedgen::data::assets::{PresetGroup, PresetInfo};

use crate::cli::interactive;

use super::{SeedSettings, SeedWorldSettings, LITERAL};

#[derive(Debug, Default)]
pub struct WorldPresetArgs {
    pub info_args: PresetInfoArgs<false>,
    pub settings: SeedWorldSettings,
}

impl Args for WorldPresetArgs {
    fn augment_args(mut cmd: clap::Command) -> clap::Command {
        cmd = PresetInfoArgs::<false>::augment_args(cmd);
        SeedWorldSettings::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for WorldPresetArgs {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let mut s = Self::default();
        s.update_from_arg_matches(matches)?;
        Ok(s)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        let Self {
            info_args,
            settings,
        } = self;

        info_args.update_from_arg_matches(matches)?;
        settings.update_from_arg_matches(matches)
    }
}

#[derive(Debug, Default)]
pub struct UniversePresetArgs {
    pub info_args: PresetInfoArgs<true>,
    pub settings: SeedSettings,
}

impl Args for UniversePresetArgs {
    fn augment_args(mut cmd: clap::Command) -> clap::Command {
        cmd = PresetInfoArgs::<true>::augment_args(cmd);
        SeedWorldSettings::augment_args(cmd)
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

impl FromArgMatches for UniversePresetArgs {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let mut s = Self::default();
        s.update_from_arg_matches(matches)?;
        Ok(s)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        let Self {
            info_args,
            settings,
        } = self;

        info_args.update_from_arg_matches(matches)?;
        settings.update_from_arg_matches(matches)
    }
}

#[derive(Debug, Default)]
pub struct PresetInfoArgs<const UNIVERSE: bool> {
    pub identifier: String,
    pub info: PresetInfo,
}

impl<const UNIVERSE: bool> Args for PresetInfoArgs<UNIVERSE> {
    fn group_id() -> Option<clap::Id> {
        Some("preset_info".into())
    }

    fn augment_args(cmd: clap::Command) -> clap::Command {
        cmd.group(ArgGroup::new("preset_info").multiple(true))
            .arg(identifier_arg::<UNIVERSE>())
            .arg(display_name_arg())
            .arg(description_arg())
            .arg(base_preset_arg())
    }

    fn augment_args_for_update(cmd: clap::Command) -> clap::Command {
        Self::augment_args(cmd)
    }
}

fn identifier_arg<const UNIVERSE: bool>() -> Arg {
    Arg::new("identifier")
        .value_name("IDENTIFIER")
        .required_unless_present("interactive")
        .help("The preset's identifier")
        .long_help(format!(
            "The preset's identifier which you can later pass like '{literal}seedgen seed -{flag} <identifier>{reset}'",
            literal = LITERAL.render(),
            flag = if UNIVERSE { 'P' } else { 'p' },
            reset = Reset.render(),
        ))
}

fn display_name_arg() -> Arg {
    Arg::new("display_name")
        .group("preset_info")
        .long("display-name")
        .short('n')
        .value_name("STRING")
        .help("The preset's display name")
}

fn description_arg() -> Arg {
    Arg::new("description")
        .group("preset_info")
        .long("description")
        .short('D')
        .value_name("STRING")
        .help("The preset's extended description")
}

fn base_preset_arg() -> Arg {
    Arg::new("base_preset")
        .group("preset_info")
        .long("base-preset")
        .short('b')
        .value_name("BOOLEAN")
        .action(ArgAction::SetTrue)
        .help("Whether the preset should be displayed as a base preset")
}

impl<const UNIVERSE: bool> FromArgMatches for PresetInfoArgs<UNIVERSE> {
    fn from_arg_matches(matches: &ArgMatches) -> Result<Self, clap::Error> {
        let mut s = Self::default();
        s.update_from_arg_matches(matches)?;
        Ok(s)
    }

    fn update_from_arg_matches(&mut self, matches: &ArgMatches) -> Result<(), clap::Error> {
        self.identifier = matches
            .get_one::<String>("identifier")
            .cloned()
            .unwrap_or_default();

        self.info = PresetInfo {
            name: matches.get_one("display_name").cloned(),
            description: matches.get_one("description").cloned(),
            group: matches.get_flag("base_preset").then_some(PresetGroup::Base),
        };

        if matches.get_flag("interactive") {
            interactive::preset_info(self)?;
        }

        if self.identifier.is_empty() {
            return Err(clap::Error::raw(
                ErrorKind::MissingRequiredArgument,
                "the following required argument was not provided: identifier",
            ));
        }

        Ok(())
    }
}
