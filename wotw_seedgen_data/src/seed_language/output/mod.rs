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
    CommandZone, IntoConstant,
};
pub use contained_uber_identifiers::{
    CommandVoidWrites, CommonItem, CommonUberStateWrite, CommonWriteCommand, ContainedWrites,
    ContainedWritesExt, ContainedWritesIter, UberStateWrite, UberStateWriteGeneric,
    UberStateWriteOwned, WriteCommand, WriteCommandOwned,
};
pub use event::{ClientEvent, Event, Trigger, TriggerCondition};
pub use intermediate::{Constant, Literal, Reference, VariableValue};
pub(crate) use item_metadata::ItemMetadataEntry;
pub use item_metadata::{ItemMetadata, ItemMetadataRef};
pub use operation::{
    ArithmeticOperator, Comparator, Concatenator, EqualityComparator, ExecuteOperator,
    LogicOperator, Operation,
};
pub use postprocess::{postprocess, PlaceholderMap, UniversePostprocessor};
use utoipa::ToSchema;

use crate::{Icon, Position, UberIdentifier, Zone};
use rustc_hash::{FxBuildHasher, FxHashMap, FxHashSet};
use serde::{Deserialize, Serialize};
use std::{hash::Hash, ops::Range};

// TODO check all the public derives
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct IntermediateOutput<'log> {
    pub preload: PreloadOutput,
    pub commands: CommandsOutput,
    pub modifiers: GenerationModifiers<'log>,
    pub assets: AssetsOutput,
}

impl<'log> IntermediateOutput<'log> {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn enable_debug(&mut self) {
        self.assets.enable_debug();
    }

    pub fn disable_debug(&mut self) {
        self.assets.disable_debug();
    }

    pub fn purge_only_simulation(&mut self) {
        for range in self.modifiers.only_simulation_events.drain(..).rev() {
            self.commands.events.drain(range);
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PreloadOutput {
    pub spawn: Option<Position>,
    pub tags: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct CommandsOutput {
    events: Vec<Event>,
    pub lookup: Vec<CommandVoid>,
    trigger_map: FxHashMap<UberIdentifier, Vec<usize>>,
}

impl CommandsOutput {
    pub const NONE: Self = Self {
        events: Vec::new(),
        lookup: Vec::new(),
        trigger_map: FxHashMap::with_hasher(FxBuildHasher),
    };

    pub fn events(&self) -> &Vec<Event> {
        &self.events
    }

    pub fn events_mut(&mut self) -> &mut [Event] {
        &mut self.events
    }

    pub fn push_event(&mut self, event: Event) {
        for read in event.trigger.contained_reads() {
            self.trigger_map
                .entry(read)
                .or_default()
                .push(self.events.len());
        }

        self.events.push(event);
    }

    pub fn push_event_without_registering_trigger(&mut self, event: Event) {
        self.events.push(event);
    }

    pub fn extend_events<I: IntoIterator<Item = Event>>(&mut self, iter: I) {
        self.events.extend(
            iter.into_iter()
                .zip(self.events.len()..)
                .map(|(event, index)| {
                    for read in event.trigger.contained_reads() {
                        self.trigger_map.entry(read).or_default().push(index);
                    }

                    event
                }),
        );
    }

    pub fn into_inner(self) -> (Vec<Event>, Vec<CommandVoid>) {
        (self.events, self.lookup)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct GenerationModifiers<'log> {
    pub item_pool_changes: FxHashMap<CommandVoid, i32>,
    pub spirit_light_change: i32,
    pub item_metadata: ItemMetadata<'log>,
    pub removed_locations: FxHashSet<CommandBoolean>,
    pub location_slots: FxHashMap<CommandBoolean, u32>,
    pub logical_state_sets: FxHashSet<String>,
    pub only_simulation_events: Vec<Range<usize>>,
    pub preplacements: Vec<(CommandVoid, Zone)>,
}

impl GenerationModifiers<'_> {
    pub fn total_spirit_light(&self) -> i32 {
        20000 + self.spirit_light_change
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AssetsOutput {
    pub icons: Vec<(String, Vec<u8>)>, // TODO poor memory
    pub debug: Option<DebugOutput>,
}

impl AssetsOutput {
    fn enable_debug(&mut self) {
        self.debug = Some(DebugOutput::default());
    }

    fn disable_debug(&mut self) {
        self.debug = None;
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

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub enum StringOrPlaceholder {
    Value(String),
    #[schema(no_recursion)]
    ZoneOfPlaceholder(Box<CommandVoid>),
    #[schema(no_recursion)]
    ItemOnPlaceholder(Box<Trigger>),
    CountInZonePlaceholder(#[schema(no_recursion)] Vec<CommandVoid>, Zone),
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
