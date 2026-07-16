use std::{
    fmt::{self, Display},
    sync::Mutex,
    time::{Duration, Instant},
};

use indexmap::IndexMap;
use itertools::Itertools;
use rustc_hash::FxBuildHasher;
use wotw_seedgen_data::logic_language::output::Graph;

use crate::world::ConnectionIndex;

#[derive(Debug, Default)]
pub struct PerfData<'graph> {
    reached: Option<Mutex<IndexMap<ConnectionIndex<'graph>, Duration, FxBuildHasher>>>,
}

impl<'graph> PerfData<'graph> {
    pub const fn new() -> Self {
        Self { reached: None }
    }

    pub fn record_reached(&mut self) {
        self.reached = Some(Mutex::new(IndexMap::default()));
    }

    pub fn display<'data>(&'data self, graph: &'graph Graph) -> PerfDataDisplay<'data, 'graph> {
        PerfDataDisplay::new(self, graph)
    }

    pub(crate) fn reached_start(&self) -> Option<ReachedRecord> {
        self.reached
            .is_some()
            .then(|| ReachedRecord(Instant::now()))
    }

    pub(crate) fn reached_finish(
        &self,
        record: ReachedRecord,
        connection: ConnectionIndex<'graph>,
    ) {
        let Some(reached) = &self.reached else {
            return;
        };

        *reached.lock().unwrap().entry(connection).or_default() += record.0.elapsed();
    }
}

pub struct PerfDataDisplay<'data, 'graph> {
    data: &'data PerfData<'graph>,
    graph: &'graph Graph,
}

impl<'data, 'graph> PerfDataDisplay<'data, 'graph> {
    fn new(data: &'data PerfData<'graph>, graph: &'graph Graph) -> Self {
        Self { data, graph }
    }
}

impl Display for PerfDataDisplay<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(reached) = &self.data.reached {
            let mut reached = reached.lock().unwrap();

            let total = reached.values().sum::<Duration>();
            writeln!(f, "Total: {}s", total.as_secs_f64())?;

            reached.sort_unstable_by(|_, a, _, b| a.cmp(b).reverse());

            reached
                .iter()
                .format_with("\n", |(connection, duration), f| {
                    f(&format_args!(
                        "{}: {}s",
                        connection.display(self.graph),
                        duration.as_secs_f64()
                    ))
                })
                .fmt(f)?;
        }

        Ok(())
    }
}

pub(crate) struct ReachedRecord(Instant);
