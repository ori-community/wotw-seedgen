use std::{
    io,
    ops::{Index, IndexMut},
};

use fern::{colors::ColoredLevelConfig, Dispatch};
use log::{LevelFilter, Metadata};
use wotw_seedgen::data::{
    assets::{self, LOG_DATA_DIR},
    seed_language::simulate::UBER_STATES_TARGET_PREFIX,
    UberIdentifier,
};

use crate::{
    cli::{VerboseArgs, VerboseTarget},
    Error,
};

#[derive(Debug)]
pub struct LogConfig {
    trace_seedgen: bool,
    trace_placement: LevelFilter,
    trace_reached: LevelFilter,
    trace_is_met: LevelFilter,
    trace_solutions: LevelFilter,
    trace_weight: LevelFilter,
    trace_spawn: LevelFilter,
    trace_entrances: LevelFilter,
    trace_optimize_graph: LevelFilter,
    trace_uber_states: Option<Vec<UberIdentifier>>,
}

const PLACEMENT_MOD: &str = "wotw_seedgen::generator::placement";
const ITEM_POOL_MOD: &str = "wotw_seedgen::generator::item_pool";
const REACHED_MOD: &str = "wotw_seedgen::world::reached";
const IS_MET_MOD: &str = "wotw_seedgen::world::is_met";
const SOLUTIONS_MOD: &str = "wotw_seedgen::generator::solutions";
const WEIGHT_MOD: &str = "wotw_seedgen::generator::solutions::weight";
const SPAWN_MOD: &str = "wotw_seedgen::generator::spawn";
const ENTRANCES_MOD: &str = "wotw_seedgen::generator::entrances";
const OPTIMIZE_GRAPH_MOD: &str = "wotw_seedgen_data::logic_language::optimize";

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            trace_seedgen: false,
            trace_placement: LevelFilter::Info,
            trace_reached: LevelFilter::Info,
            trace_is_met: LevelFilter::Info,
            trace_solutions: LevelFilter::Info,
            trace_weight: LevelFilter::Info,
            trace_spawn: LevelFilter::Info,
            trace_entrances: LevelFilter::Info,
            trace_optimize_graph: LevelFilter::Info,
            trace_uber_states: None,
        }
    }
}

impl Index<VerboseTarget> for LogConfig {
    type Output = LevelFilter;

    fn index(&self, index: VerboseTarget) -> &Self::Output {
        match index {
            VerboseTarget::Placement => &self.trace_placement,
            VerboseTarget::Reached => &self.trace_reached,
            VerboseTarget::IsMet => &self.trace_is_met,
            VerboseTarget::Solutions => &self.trace_solutions,
            VerboseTarget::Weight => &self.trace_weight,
            VerboseTarget::Spawn => &self.trace_spawn,
            VerboseTarget::Entrances => &self.trace_entrances,
            VerboseTarget::OptimizeGraph => &self.trace_optimize_graph,
        }
    }
}

impl IndexMut<VerboseTarget> for LogConfig {
    fn index_mut(&mut self, index: VerboseTarget) -> &mut Self::Output {
        match index {
            VerboseTarget::Placement => &mut self.trace_placement,
            VerboseTarget::Reached => &mut self.trace_reached,
            VerboseTarget::IsMet => &mut self.trace_is_met,
            VerboseTarget::Solutions => &mut self.trace_solutions,
            VerboseTarget::Weight => &mut self.trace_weight,
            VerboseTarget::Spawn => &mut self.trace_spawn,
            VerboseTarget::Entrances => &mut self.trace_entrances,
            VerboseTarget::OptimizeGraph => &mut self.trace_optimize_graph,
        }
    }
}

impl LogConfig {
    pub fn from_args(args: VerboseArgs) -> Self {
        let VerboseArgs {
            verbose,
            trace_uber_states,
        } = args;

        let mut config = Self::default();

        if let Some(targets) = verbose {
            config.trace_seedgen = true;

            if targets.is_empty() {
                config.trace_placement = LevelFilter::Trace
            } else {
                for target in targets {
                    config[target] = LevelFilter::Trace;
                }
            }

            config.trace_uber_states = trace_uber_states;
        }

        config
    }

    pub fn apply(self) -> Result<(), Error> {
        let colors = ColoredLevelConfig::new();

        let mut dispatch = Dispatch::new().chain(
            Dispatch::new()
                .format(move |out, message, record| {
                    out.finish(format_args!("{} {}", colors.color(record.level()), message))
                })
                .level(LevelFilter::Info)
                .chain(io::stderr()),
        );

        let Self {
            trace_seedgen,
            trace_placement,
            trace_reached,
            trace_is_met,
            trace_solutions,
            trace_weight,
            trace_spawn,
            trace_entrances,
            trace_optimize_graph,
            trace_uber_states,
        } = self;

        if trace_seedgen {
            assets::create_dir_all(&*LOG_DATA_DIR)?;

            let mut file_dispatch = Dispatch::new()
                .format(|out, message, record| {
                    out.finish(format_args!("{:<7}{}", record.level(), message))
                })
                .level_for(PLACEMENT_MOD, trace_placement)
                .level_for(ITEM_POOL_MOD, trace_placement)
                .level_for(REACHED_MOD, trace_reached)
                .level_for(IS_MET_MOD, trace_is_met)
                .level_for(SOLUTIONS_MOD, trace_solutions)
                .level_for(WEIGHT_MOD, trace_weight)
                .level_for(SPAWN_MOD, trace_spawn)
                .level_for(ENTRANCES_MOD, trace_entrances)
                .level_for(OPTIMIZE_GRAPH_MOD, trace_optimize_graph)
                .level_for("perf_counters", LevelFilter::Off)
                .chain(assets::file_create(LOG_DATA_DIR.join("seedgen_log.txt"))?);

            match trace_uber_states {
                None => {
                    file_dispatch =
                        file_dispatch.filter(|metadata| !uber_states_level_filter(metadata))
                }
                Some(uber_states) if uber_states.is_empty() => {
                    file_dispatch = file_dispatch.filter(uber_states_level_filter)
                }
                Some(uber_states) => {
                    let filter = FilterUberStates { uber_states };
                    file_dispatch = file_dispatch.filter(move |metadata| filter.filter(metadata))
                }
            }

            dispatch = dispatch.chain(file_dispatch);
        }

        dispatch.apply()?;

        Ok(())
    }
}

struct FilterUberStates {
    uber_states: Vec<UberIdentifier>,
}

impl FilterUberStates {
    fn filter(&self, metadata: &Metadata) -> bool {
        let Some(uber_identifier) = metadata.target().strip_prefix(UBER_STATES_TARGET_PREFIX)
        else {
            return true;
        };

        let uber_identifier = uber_identifier.parse().unwrap();
        self.uber_states.contains(&uber_identifier)
    }
}

fn uber_states_level_filter(metadata: &Metadata) -> bool {
    metadata.target().starts_with(UBER_STATES_TARGET_PREFIX)
}
