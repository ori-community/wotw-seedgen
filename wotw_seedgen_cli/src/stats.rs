use crate::{
    cli::{self, StatsArgs},
    Error,
};
use wotw_seedgen_stats::ChainedAnalyzers;

pub fn stats(_args: StatsArgs) -> Result<(), Error> {
    todo!();
}

impl Into<ChainedAnalyzers> for cli::ChainedAnalyzers {
    fn into(self) -> ChainedAnalyzers {
        todo!()
    }
}
