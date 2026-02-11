use std::{
    io,
    ops::{Index, IndexMut},
};

use fern::{colors::ColoredLevelConfig, Dispatch};
use log::LevelFilter;
use wotw_seedgen::data::assets::{self, LOG_DATA_DIR};

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
    trace_doors: LevelFilter,
    trace_optimize_graph: LevelFilter,
}

const PLACEMENT_MOD: &str = "wotw_seedgen::generator::placement";
const ITEM_POOL_MOD: &str = "wotw_seedgen::generator::item_pool";
const REACHED_MOD: &str = "wotw_seedgen::world::reached";
const IS_MET_MOD: &str = "wotw_seedgen::world::is_met";
const SOLUTIONS_MOD: &str = "wotw_seedgen::generator::solutions";
const WEIGHT_MOD: &str = "wotw_seedgen::generator::solutions::weight";
const DOORS_MOD: &str = "wotw_seedgen::generator::doors";
const OPTIMIZE_GRAPH_MOD: &str = "wotw_seedgen_data::logic_language::optimize";

impl Default for LogConfig {
    fn default() -> Self {
        Self {
            trace_seedgen: false,
            trace_placement: LevelFilter::Off,
            trace_reached: LevelFilter::Off,
            trace_is_met: LevelFilter::Off,
            trace_solutions: LevelFilter::Off,
            trace_weight: LevelFilter::Off,
            trace_doors: LevelFilter::Off,
            trace_optimize_graph: LevelFilter::Off,
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
            VerboseTarget::Doors => &self.trace_doors,
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
            VerboseTarget::Doors => &mut self.trace_doors,
            VerboseTarget::OptimizeGraph => &mut self.trace_optimize_graph,
        }
    }
}

impl LogConfig {
    pub fn from_args(args: VerboseArgs) -> Self {
        let mut config = Self::default();

        if let Some(targets) = args.verbose {
            config.trace_seedgen = true;

            if targets.is_empty() {
                config.trace_placement = LevelFilter::Trace
            } else {
                for target in targets {
                    config[target] = LevelFilter::Trace;
                }
            }
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
            trace_doors,
            trace_optimize_graph,
        } = self;

        if trace_seedgen {
            assets::create_dir_all(&*LOG_DATA_DIR)?;

            dispatch = dispatch.chain(
                Dispatch::new()
                    .format(move |out, message, record| {
                        out.finish(format_args!("{:<7}{}", record.level(), message))
                    })
                    .level_for(PLACEMENT_MOD, trace_placement)
                    .level_for(ITEM_POOL_MOD, trace_placement)
                    .level_for(REACHED_MOD, trace_reached)
                    .level_for(IS_MET_MOD, trace_is_met)
                    .level_for(SOLUTIONS_MOD, trace_solutions)
                    .level_for(WEIGHT_MOD, trace_weight)
                    .level_for(DOORS_MOD, trace_doors)
                    .level_for(OPTIMIZE_GRAPH_MOD, trace_optimize_graph)
                    .chain(assets::file_create(LOG_DATA_DIR.join("seedgen_log.txt"))?),
            )
        }

        dispatch.apply()?;

        Ok(())
    }
}
