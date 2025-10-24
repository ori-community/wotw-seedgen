mod command;
mod display;
mod event;
mod intermediate;
mod operation;
mod postprocess;

pub use command::{
    Command, CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandVoid, CommandZone,
};
pub use event::{ClientEvent, Event, Trigger};
pub use intermediate::{Constant, ConstantDiscriminants, Literal};
pub use operation::{ArithmeticOperator, Comparator, EqualityComparator, LogicOperator, Operation};
pub use postprocess::StringPlaceholderMap;

use rustc_hash::{FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::hash::Hash;
use wotw_seedgen_data::{Icon, MapIcon, Position};

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
    pub events: FxHashMap<String, FxHashMap<String, usize>>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct SnippetDebugOutput {
    pub variables: FxHashMap<String, String>,
    pub function_indices: FxHashMap<String, usize>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemMetadata(pub(crate) FxHashMap<CommandVoid, ItemMetadataEntry>);
impl ItemMetadata {
    pub fn name(&self, command: &CommandVoid) -> Option<StringOrPlaceholder> {
        self.0.get(command).and_then(|entry| entry.name.clone())
    }

    /// Force some kind of name out of `command`.
    ///
    /// If nothing is given by [`Self::name`], tries to scan `command` for messages
    /// or even uses an approximate seedlang representation of it.
    pub fn force_name(&self, command: &CommandVoid) -> CommandString {
        self.name(command)
            .map(CommandString::from)
            .or_else(|| command.find_message().cloned())
            .unwrap_or_else(|| command.to_string().into())
    }

    pub fn shop_data(
        &self,
        command: &CommandVoid,
    ) -> (Option<CommandInteger>, Option<CommandString>, Option<Icon>) {
        self.0.get(command).map_or((None, None, None), |entry| {
            (
                entry.price.clone(),
                entry.description.clone(),
                entry.icon.clone(),
            )
        })
    }
    pub fn map_icon(&self, command: &CommandVoid) -> Option<MapIcon> {
        self.0.get(command).and_then(|entry| entry.map_icon)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ItemMetadataEntry {
    /// Generic name used when sending the item to another world and in the spoiler
    pub name: Option<StringOrPlaceholder>, // TODO why not commandstring
    /// Base price used when placed in a shop
    pub price: Option<CommandInteger>,
    /// Description used when placed in a shop
    pub description: Option<CommandString>,
    /// Icon used when placed in a shop
    pub icon: Option<Icon>,
    /// Map Icon used in the spoiler map
    pub map_icon: Option<MapIcon>,
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
