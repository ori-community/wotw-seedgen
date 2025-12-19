mod contained_reads;
mod contained_writes;

pub use contained_reads::ContainedReads;
pub use contained_writes::{
    CommonItem, CommonUberStateWrite, CommonWriteCommand, ContainedWrites, UberStateWrite,
    UberStateWriteGeneric, WriteCommand,
};

use std::iter;

fn none<'a, T: 'a>() -> Box<dyn Iterator<Item = T> + 'a> {
    Box::new(iter::empty())
}

fn some<'a, T: 'a>(t: T) -> Box<dyn Iterator<Item = T> + 'a> {
    Box::new(iter::once(t))
}
