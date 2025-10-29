mod command;
mod contained_uber_identifiers;
mod display;
mod event;
mod intermediate;
mod item_metadata;
mod operation;
mod postprocess;

pub use command::{
    AsConstant, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandVoid,
    CommandZone,
};
pub use contained_uber_identifiers::{
    CommonItem, CommonUberStateWrite, CommonWriteCommand, ContainedReads, ContainedWrites,
    UberStateWrite, UberStateWriteGeneric, WriteCommand,
};
pub use event::{ClientEvent, Event, Trigger};
pub use intermediate::{Constant, ConstantDiscriminants, Literal};
pub(crate) use item_metadata::ItemMetadataEntry;
pub use item_metadata::{ItemMetadata, ItemMetadataRef};
pub use operation::{
    ArithmeticOperator, Comparator, Concatenator, EqualityComparator, ExecuteOperator,
    LogicOperator, Operation,
};
pub use postprocess::StringPlaceholderMap;

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use wotw_seedgen_data::{Icon, Position};

// TODO check all the public derives
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntermediateOutput {
    pub spawn: Option<Position>,
    pub events: Vec<Event>,
    pub command_lookup: Vec<CommandVoid>,
    pub icons: Vec<(String, Vec<u8>)>, // TODO poor memory
    pub tags: Vec<String>,
    pub item_pool_changes: FxHashMap<CommandVoid, i32>,
    pub item_metadata: ItemMetadata,
    pub removed_locations: FxHashSet<CommandBoolean>,
    pub logical_state_sets: FxHashSet<String>,
    pub preplacements: Vec<(CommandVoid, wotw_seedgen_data::Zone)>,
    pub debug: Option<DebugOutput>,
}

impl IntermediateOutput {
    pub fn new(debug: bool) -> Self {
        let mut s = Self::default();
        if debug {
            s.debug = Some(Default::default());
        }
        s
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct DebugOutput {
    pub snippets: FxHashMap<String, SnippetDebugOutput>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SnippetDebugOutput {
    pub variables: FxHashMap<String, String>,
    pub function_indices: FxHashMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum StringOrPlaceholder {
    Value(String),
    ZoneOfPlaceholder(Box<CommandVoid>),
    ItemOnPlaceholder(Box<Trigger>),
    CountInZonePlaceholder(Vec<CommandVoid>, wotw_seedgen_data::Zone),
}

impl From<String> for StringOrPlaceholder {
    fn from(value: String) -> Self {
        Self::Value(value)
    }
}

impl From<&str> for StringOrPlaceholder {
    fn from(value: &str) -> Self {
        Self::Value(value.to_string())
    }
}
