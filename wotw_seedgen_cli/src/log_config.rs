use std::{fs::File, io};

use fern::{colors::ColoredLevelConfig, Dispatch};
use log::LevelFilter;

use crate::{
    cli::{VerboseArgs, VerboseTarget},
    Error,
};

pub struct LogConfig {
    trace_seedgen: bool,
    trace_placement: bool,
    trace_reached: bool,
}

impl LogConfig {
    pub fn new() -> Self {
        Self {
            trace_seedgen: false,
            trace_placement: false,
            trace_reached: false,
        }
    }

    pub fn from_args(args: VerboseArgs) -> Self {
        let mut config = Self::new();

        if let Some(targets) = args.verbose {
            config = config
                .trace_seedgen(true)
                .trace_placement(targets.is_empty() || targets.contains(&VerboseTarget::Placement))
                .trace_reached(targets.contains(&VerboseTarget::Reached))
        }

        config
    }

    pub fn trace_seedgen(mut self, trace_seedgen: bool) -> Self {
        self.trace_seedgen = trace_seedgen;
        self
    }
    pub fn trace_placement(mut self, trace_placement: bool) -> Self {
        self.trace_placement = trace_placement;
        self
    }
    pub fn trace_reached(mut self, trace_reached: bool) -> Self {
        self.trace_reached = trace_reached;
        self
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

        if self.trace_seedgen {
            dispatch = dispatch.chain(
                Dispatch::new()
                    .format(move |out, message, record| {
                        out.finish(format_args!("{:<7}{}", record.level(), message))
                    })
                    .level_for(
                        "wotw_seedgen::generator::placement",
                        level_filter(self.trace_placement),
                    )
                    .level_for(
                        "wotw_seedgen::world::reached",
                        level_filter(self.trace_reached),
                    )
                    .chain(File::create("seedgen_log.txt")?),
            )
        }

        dispatch.apply()?;

        Ok(())
    }
}

fn level_filter(trace: bool) -> LevelFilter {
    if trace {
        LevelFilter::Trace
    } else {
        LevelFilter::Off
    }
}
