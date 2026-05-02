mod price_noise;

use std::{
    collections::hash_map::Entry,
    fmt::{self, Display},
    mem,
    ops::{Deref, Index},
};

use super::{
    ArithmeticOperator, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
    CommandVoid, CommandZone, Comparator, Concatenator, EqualityComparator, Event,
    IntermediateOutput, ItemMetadata, Operation, StringOrPlaceholder, Trigger,
};

use crate::{
    assets::{LocData, LocDataEntry},
    seed_language::output::{
        display::strip_invisible_characters,
        item_metadata::{random_shop_description, DEFAULT_SHOP_PRICE},
        postprocess::price_noise::PriceNoise,
        ClientEvent, ContainedWrites, IntoConstant, ItemMetadataRef, TriggerCondition,
    },
    Position, ShopKind, UberIdentifier, Zone,
};
use itertools::Itertools;
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;

pub fn postprocess(
    worlds: &mut [&mut IntermediateOutput],
    loc_data: &LocData,
    rng: &mut Pcg64Mcg,
) -> Vec<PlaceholderMap> {
    let postprocessor = UniversePostprocessor::new(worlds, loc_data);

    let placeholder_maps = postprocessor.resolve_placeholders().collect::<Vec<_>>();
    let extra_events = postprocessor.generate_defaults(rng).collect::<Vec<_>>();

    let UniversePostprocessor {
        loc_data_triggers, ..
    } = postprocessor;

    for (output, extra_events) in worlds.iter_mut().zip(extra_events) {
        // TODO merge events with identical triggers?

        loc_data_triggers.generate_message_origins(&mut output.events);

        output.events.splice(0..0, extra_events);
    }

    placeholder_maps
}

// TODO maybe zone_of should be a typed zone placeholder?
#[derive(Debug, Clone, Default)]
pub struct PlaceholderMap {
    pub strings: FxHashMap<StringOrPlaceholder, CommandString>,
}

pub struct UniversePostprocessor<'output, 'locdata> {
    worlds: Vec<WorldPostprocessor<'output>>,
    loc_data_triggers: LocDataTriggers<'locdata>,
    multiworld_lookup: MultiworldLookup<'output>,
}

struct WorldPostprocessor<'output> {
    output: &'output IntermediateOutput,
    loc_data_events: FxHashMap<&'output Trigger, Vec<&'output Event>>,
}

struct LocDataTriggers<'locdata> {
    inner: FxHashMap<CommandBoolean, &'locdata LocDataEntry>,
}

#[derive(Default)]
struct MultiworldLookup<'output> {
    inner: FxHashMap<i32, MultiworldEvent<'output>>,
}

struct MultiworldEvent<'output> {
    origin_world_index: usize,
    origin_trigger: &'output Trigger,
    target_world_index: usize,
    target_command: &'output CommandVoid,
}

impl<'output, 'locdata> UniversePostprocessor<'output, 'locdata> {
    pub fn new(worlds: &'output [&mut IntermediateOutput], loc_data: &'locdata LocData) -> Self {
        let loc_data_triggers = LocDataTriggers::new(loc_data);
        let multiworld_lookup = MultiworldLookup::new(worlds);
        let worlds = worlds
            .iter()
            .map(|output| WorldPostprocessor::new(output, &loc_data_triggers))
            .collect();

        Self {
            worlds,
            loc_data_triggers,
            multiworld_lookup,
        }
    }

    pub fn resolve_placeholders(&self) -> impl Iterator<Item = PlaceholderMap> + use<'_> {
        (0..self.worlds.len()).map(|world_index| {
            let mut context = ResolveContext::new(self, world_index);

            self.worlds[world_index].output.resolve(&mut context);

            context.placeholder_map
        })
    }

    pub fn generate_defaults<'s, 'r>(
        &'s self,
        rng: &'r mut Pcg64Mcg,
    ) -> impl Iterator<Item = Vec<Event>> + use<'s, 'r> {
        let price_noise = PriceNoise::new();

        self.worlds
            .iter()
            .map(move |world| self.generate_defaults_for(world, &price_noise, rng))
    }

    fn resolve_zone_of(
        &self,
        uber_identifiers: &[UberIdentifier],
        target_world_index: usize,
    ) -> CommandString {
        let target_world = &self.worlds[target_world_index];

        let matches = target_world
            .output
            .events
            .iter()
            .filter(|event| self.command_writes_any(&event.command, uber_identifiers))
            .filter_map(|event| self.zone_of_trigger(&event.trigger, target_world_index))
            .map(|(origin_world_index, zone)| ZoneOfMatch {
                origin_world_index,
                target_world_index,
                zone,
            });

        let message = matches.format(" or ").to_string();

        if message.is_empty() {
            "Unknown".into()
        } else {
            message.into()
        }
    }

    fn zone_of_trigger(
        &self,
        trigger: &Trigger,
        origin_world_index: usize,
    ) -> Option<(usize, Zone)> {
        if let Trigger::ClientEvent(ClientEvent::Spawn) = trigger {
            return Some((origin_world_index, Zone::Spawn));
        }

        if let Some(id) = trigger.as_multiworld() {
            return self
                .multiworld_lookup
                .get(&id)
                .and_then(|multiworld_event| {
                    self.zone_of_trigger(
                        &multiworld_event.origin_trigger,
                        multiworld_event.origin_world_index,
                    )
                });
        }

        self.loc_data_triggers
            .get(trigger)
            .map(|entry| (origin_world_index, entry.zone))
    }

    fn resolve_item_on(&self, trigger: &Trigger, origin_world_index: usize) -> CommandString {
        self.worlds[origin_world_index].resolve_item_on(trigger)
    }

    fn resolve_count_in_zone(
        &self,
        uber_identifiers: &[UberIdentifier],
        zone: Zone,
        origin_world_index: usize,
    ) -> CommandString {
        let origin_world = &self.worlds[origin_world_index];

        let matches = origin_world
            .loc_data_events
            .iter()
            .map(|(trigger, events)| (&self.loc_data_triggers[trigger], events))
            .filter(|(entry, _)| entry.zone == zone)
            .flat_map(|(entry, events)| {
                events
                    .iter()
                    .filter(|event| self.command_writes_any(&event.command, uber_identifiers))
                    .map(move |event| (*event, entry))
            })
            .collect::<Vec<_>>();

        count_in_zone_message(matches, &origin_world.output.item_metadata)
    }

    fn command_writes_any(
        &self,
        command: &CommandVoid,
        uber_identifiers: &[UberIdentifier],
    ) -> bool {
        command
            .contained_write_identifiers()
            .any(|uber_identifier| match uber_identifier.as_multiworld() {
                None => uber_identifiers.contains(&uber_identifier),
                Some(id) => self
                    .multiworld_lookup
                    .get(&id)
                    .is_some_and(|multiworld_event| {
                        self.command_writes_any(multiworld_event.target_command, uber_identifiers)
                    }),
            })
    }

    fn generate_defaults_for(
        &self,
        world: &WorldPostprocessor,
        price_noise: &PriceNoise,
        rng: &mut Pcg64Mcg,
    ) -> Vec<Event> {
        let mut extra_events = vec![];

        for (trigger, events) in &world.loc_data_events {
            let shop_identifier = trigger
                .as_condition()
                .and_then(|condition| match condition {
                    CommandBoolean::FetchBoolean { uber_identifier }
                        if matches!(
                            uber_identifier.shop_kind(),
                            ShopKind::Opherlike | ShopKind::Map
                        ) =>
                    {
                        Some(*uber_identifier)
                    }
                    _ => None,
                });

            let map_position = self.loc_data_triggers[trigger].map_position;

            if shop_identifier.is_none() && map_position.is_none() {
                continue;
            }

            let (name, matches) = self.find_metadata(events, world);

            if let Some(uber_identifier) = shop_identifier {
                self.generate_shop_defaults(
                    uber_identifier,
                    name.clone(),
                    &matches,
                    price_noise,
                    rng,
                    &mut extra_events,
                );
            }

            if let Some(map_position) = map_position {
                self.generate_spoiler_defaults(map_position, name, &matches, &mut extra_events);
            }
        }

        extra_events
    }

    fn find_metadata(
        &self,
        events: &[&'output Event],
        origin_world: &WorldPostprocessor<'output>,
    ) -> (CommandString, Vec<ItemMetadataRef<'output, 'output>>) {
        let mut matches = vec![];

        let names = events.iter().filter_map(|event| {
            let metadata = origin_world.output.item_metadata.get(&event.command);

            let name = metadata.try_force_name()?;

            matches.push(metadata);

            matches.extend(
                event
                    .command
                    .contained_write_identifiers()
                    .filter_map(UberIdentifier::as_multiworld)
                    .filter_map(|id| self.multiworld_lookup.get(&id))
                    .map(|multiworld_event| {
                        self.worlds[multiworld_event.target_world_index]
                            .output
                            .item_metadata
                            .get(multiworld_event.target_command)
                    }),
            );

            Some(name)
        });

        let name = multi_name(names);

        (name, matches)
    }

    fn generate_shop_defaults(
        &self,
        uber_identifier: UberIdentifier,
        name: CommandString,
        matches: &[ItemMetadataRef<'output, 'output>],
        price_noise: &PriceNoise,
        rng: &mut Pcg64Mcg,
        extra_events: &mut Vec<Event>,
    ) {
        if matches!(
            uber_identifier.shop_kind(),
            ShopKind::Opherlike | ShopKind::Map
        ) {
            let prices = matches
                .iter()
                .filter_map(|metadata| metadata.try_force_shop_price());

            let mut price = multi_price(prices);
            price_noise.add_noise(&mut price, rng);

            extra_events.push(Event::on_reload(CommandVoid::SetShopItemPrice {
                uber_identifier,
                price,
            }));

            if uber_identifier.shop_kind() == ShopKind::Opherlike {
                extra_events.push(Event::on_reload(CommandVoid::SetShopItemName {
                    uber_identifier,
                    name,
                }));

                let mut descriptions = matches.iter().filter_map(|metadata| metadata.description());

                let description = match (descriptions.next(), descriptions.next()) {
                    (Some(description), None) => description,
                    _ => random_shop_description(rng).into(),
                };

                extra_events.push(Event::on_reload(CommandVoid::SetShopItemDescription {
                    uber_identifier,
                    description,
                }));

                let icon = matches
                    .iter()
                    .find_map(|metadata| metadata.try_force_icon());

                if let Some(icon) = icon {
                    extra_events.push(Event::on_reload(CommandVoid::SetShopItemIcon {
                        uber_identifier,
                        icon,
                    }));
                }
            }
        }
    }

    fn generate_spoiler_defaults(
        &self,
        map_position: Position,
        name: CommandString,
        matches: &[ItemMetadataRef<'output, 'output>],
        extra_events: &mut Vec<Event>,
    ) {
        let icon = matches
            .iter()
            .find_map(|metadata| metadata.try_force_map_icon())
            .unwrap_or_default();

        let label = match name.into_constant() {
            Ok(name) => strip_invisible_characters(&name).into(),
            Err(name) => name,
        };

        extra_events.push(Event::on_reload(CommandVoid::CreateSpoilerMapIcon {
            icon,
            x: map_position.x.into(),
            y: map_position.y.into(),
            label,
        }));
    }
}

impl<'output> WorldPostprocessor<'output> {
    fn new(output: &'output IntermediateOutput, loc_data_triggers: &LocDataTriggers) -> Self {
        let mut loc_data_events = FxHashMap::<_, Vec<_>>::default();

        for event in output
            .events
            .iter()
            .filter(|event| loc_data_triggers.contains(&event.trigger))
        {
            loc_data_events
                .entry(&event.trigger)
                .or_default()
                .push(event);
        }

        Self {
            output,
            loc_data_events,
        }
    }

    fn resolve_item_on(&self, trigger: &Trigger) -> CommandString {
        let names = self
            .output
            .events
            .iter()
            .filter(|event| &event.trigger == trigger)
            .filter_map(|event| {
                self.output
                    .item_metadata
                    .get(&event.command)
                    .try_force_name()
            });

        multi_name(names)
    }
}

impl<'locdata> LocDataTriggers<'locdata> {
    fn new(loc_data: &'locdata LocData) -> Self {
        Self {
            inner: loc_data
                .entries
                .iter()
                .map(|entry| {
                    (
                        CommandBoolean::loc_data_condition(entry.uber_identifier, entry.value),
                        entry,
                    )
                })
                .collect(),
        }
    }

    fn get(&self, trigger: &Trigger) -> Option<&LocDataEntry> {
        self.inner.get(trigger.as_condition()?).copied()
    }

    fn contains(&self, trigger: &Trigger) -> bool {
        trigger
            .as_condition()
            .is_some_and(|condition| self.inner.contains_key(condition))
    }

    fn generate_message_origins(&self, events: &mut [Event]) {
        for event in events {
            if let Some(map_position) = self
                .get(&event.trigger)
                .and_then(|entry| entry.map_position)
            {
                let set_position = CommandVoid::QueuedMessageScopedPickupPosition {
                    x: map_position.x.into(),
                    y: map_position.y.into(),
                };

                match &mut event.command {
                    CommandVoid::Multi { commands } => commands.insert(0, set_position),
                    other => {
                        let previous = mem::replace(other, CommandVoid::Multi { commands: vec![] });
                        *other = CommandVoid::Multi {
                            commands: vec![set_position, previous],
                        }
                    }
                }
            }
        }
    }
}

impl Index<&Trigger> for LocDataTriggers<'_> {
    type Output = LocDataEntry;

    fn index(&self, index: &Trigger) -> &Self::Output {
        self.get(index).unwrap()
    }
}

impl<'output> MultiworldLookup<'output> {
    fn new(worlds: &'output [&mut IntermediateOutput]) -> Self {
        let mut multiworld_lookup = Self::default();
        let mut unmatched_triggers = FxHashMap::default();
        let mut unmatched_commands = FxHashMap::default();

        for (world_index, output) in worlds.iter().enumerate() {
            for event in &output.events {
                if let Some(id) = event.trigger.as_multiworld() {
                    match unmatched_triggers.entry(id) {
                        Entry::Occupied(occupied) => {
                            let (origin_world_index, origin_trigger) = occupied.remove();
                            multiworld_lookup.inner.insert(
                                id,
                                MultiworldEvent {
                                    origin_world_index,
                                    origin_trigger,
                                    target_world_index: world_index,
                                    target_command: &event.command,
                                },
                            );
                        }
                        Entry::Vacant(_) => {
                            unmatched_commands.insert(id, (world_index, &event.command));
                        }
                    }
                } else {
                    for id in event
                        .command
                        .contained_write_identifiers()
                        .filter_map(UberIdentifier::as_multiworld)
                    {
                        match unmatched_commands.entry(id) {
                            Entry::Occupied(occupied) => {
                                let (target_world_index, target_command) = occupied.remove();
                                multiworld_lookup.inner.insert(
                                    id,
                                    MultiworldEvent {
                                        origin_world_index: world_index,
                                        origin_trigger: &event.trigger,
                                        target_world_index,
                                        target_command,
                                    },
                                );
                            }
                            Entry::Vacant(_) => {
                                unmatched_triggers.insert(id, (world_index, &event.trigger));
                            }
                        }
                    }
                }
            }
        }

        debug_assert!(
            unmatched_triggers.is_empty(),
            "unmatched multiworld triggers: {}",
            unmatched_triggers
                .iter()
                .format_with(", ", |(id, (world_index, trigger)), f| f(&format_args!(
                    "12|{id}: [{world_index}] {trigger}"
                )))
        );
        debug_assert!(
            unmatched_commands.is_empty(),
            "unmatched multiworld commands: {}",
            unmatched_commands
                .iter()
                .format_with(", ", |(id, (world_index, command)), f| f(&format_args!(
                    "12|{id}: [{world_index}] {command}"
                )))
        );

        multiworld_lookup
    }
}

impl<'output> Deref for MultiworldLookup<'output> {
    type Target = FxHashMap<i32, MultiworldEvent<'output>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

// TODO maybe this adds stats tracking?
trait ResolvePlaceholders {
    fn resolve(&self, context: &mut ResolveContext);
}

struct ResolveContext<'postprocessor, 'output, 'locdata> {
    postprocessor: &'postprocessor UniversePostprocessor<'output, 'locdata>,
    world_index: usize,
    placeholder_map: PlaceholderMap,
}

impl<'postprocessor, 'output, 'locdata> ResolveContext<'postprocessor, 'output, 'locdata> {
    fn new(
        postprocessor: &'postprocessor UniversePostprocessor<'output, 'locdata>,
        world_index: usize,
    ) -> Self {
        Self {
            postprocessor,
            world_index,
            placeholder_map: PlaceholderMap::default(),
        }
    }
}

impl<T: ResolvePlaceholders> ResolvePlaceholders for Vec<T> {
    fn resolve(&self, context: &mut ResolveContext) {
        for t in self {
            t.resolve(context);
        }
    }
}

impl<T: ResolvePlaceholders> ResolvePlaceholders for Option<T> {
    fn resolve(&self, context: &mut ResolveContext) {
        if let Some(t) = self {
            t.resolve(context);
        }
    }
}

impl<Item: ResolvePlaceholders, Operator> ResolvePlaceholders for Operation<Item, Operator> {
    fn resolve(&self, context: &mut ResolveContext) {
        self.left.resolve(context);
        self.right.resolve(context);
    }
}

impl ResolvePlaceholders for IntermediateOutput {
    fn resolve(&self, context: &mut ResolveContext) {
        self.events.resolve(context);
        self.command_lookup.resolve(context);
    }
}

impl ResolvePlaceholders for Event {
    fn resolve(&self, context: &mut ResolveContext) {
        self.trigger.resolve(context);
        self.command.resolve(context);
    }
}

impl ResolvePlaceholders for Trigger {
    fn resolve(&self, context: &mut ResolveContext) {
        if let Self::Condition(condition) = self {
            condition.resolve(context);
        }
    }
}

impl ResolvePlaceholders for TriggerCondition {
    fn resolve(&self, context: &mut ResolveContext) {
        self.condition.resolve(context);
    }
}

impl ResolvePlaceholders for Command {
    fn resolve(&self, context: &mut ResolveContext) {
        match self {
            Self::Boolean(command) => command.resolve(context),
            Self::Integer(command) => command.resolve(context),
            Self::Float(command) => command.resolve(context),
            Self::String(command) => command.resolve(context),
            Self::Zone(command) => command.resolve(context),
            Self::Void(command) => command.resolve(context),
        }
    }
}

impl ResolvePlaceholders for CommandBoolean {
    fn resolve(&self, context: &mut ResolveContext) {
        match self {
            Self::Multi { commands, last } => {
                commands.resolve(context);
                last.resolve(context);
            }
            Self::CompareBoolean { operation } => operation.resolve(context),
            Self::CompareInteger { operation } => operation.resolve(context),
            Self::CompareFloat { operation } => operation.resolve(context),
            Self::CompareString { operation } => operation.resolve(context),
            Self::CompareZone { operation } => operation.resolve(context),
            Self::LogicOperation { operation } => operation.resolve(context),
            Self::Constant { .. }
            | Self::FetchBoolean { .. }
            | Self::GetBoolean { .. }
            | Self::IsInBox { .. } => {}
        }
    }
}

impl ResolvePlaceholders for CommandInteger {
    fn resolve(&self, context: &mut ResolveContext) {
        match self {
            Self::Multi { commands, last } => {
                commands.resolve(context);
                last.resolve(context);
            }
            Self::Arithmetic { operation } => operation.resolve(context),
            Self::FromFloat { float } => float.resolve(context),
            Self::StringLength { string } => string.resolve(context),
            Self::Constant { .. } | Self::FetchInteger { .. } | Self::GetInteger { .. } => {}
        }
    }
}

impl ResolvePlaceholders for CommandFloat {
    fn resolve(&self, context: &mut ResolveContext) {
        match self {
            Self::Multi { commands, last } => {
                commands.resolve(context);
                last.resolve(context);
            }
            Self::Arithmetic { operation } => operation.resolve(context),
            Self::FromInteger { integer } => integer.resolve(context),
            Self::Constant { .. } | Self::FetchFloat { .. } | Self::GetFloat { .. } => {}
        }
    }
}

impl ResolvePlaceholders for CommandString {
    fn resolve(&self, context: &mut ResolveContext) {
        match self {
            Self::Constant { value } => {
                if context.placeholder_map.strings.contains_key(value) {
                    return;
                }

                let resolved = match value {
                    StringOrPlaceholder::Value(_) => return,
                    StringOrPlaceholder::ZoneOfPlaceholder(uber_identifiers) => context
                        .postprocessor
                        .resolve_zone_of(uber_identifiers, context.world_index),
                    StringOrPlaceholder::ItemOnPlaceholder(trigger) => context
                        .postprocessor
                        .resolve_item_on(trigger, context.world_index),
                    StringOrPlaceholder::CountInZonePlaceholder(uber_identifiers, zone) => context
                        .postprocessor
                        .resolve_count_in_zone(uber_identifiers, *zone, context.world_index),
                };

                context
                    .placeholder_map
                    .strings
                    .insert(value.clone(), resolved);
            }
            Self::Multi { commands, last } => {
                commands.resolve(context);
                last.resolve(context);
            }
            Self::Concatenate { operation } => {
                operation.resolve(context);
            }
            Self::FromBoolean { boolean } => boolean.resolve(context),
            Self::FromInteger { integer } => integer.resolve(context),
            Self::FromFloat { float } => float.resolve(context),
            Self::GetString { .. } | Self::WorldName { .. } => {}
        }
    }
}

impl ResolvePlaceholders for CommandZone {
    fn resolve(&self, context: &mut ResolveContext) {
        match self {
            Self::Multi { commands, last } => {
                commands.resolve(context);
                last.resolve(context);
            }
            Self::Constant { .. } | Self::CurrentZone {} | Self::CurrentMapZone {} => {}
        }
    }
}

impl ResolvePlaceholders for CommandVoid {
    fn resolve(&self, context: &mut ResolveContext) {
        match self {
            Self::Multi { commands } => commands.resolve(context),
            Self::If { condition, command } => {
                condition.resolve(context);
                command.resolve(context);
            }
            Self::QueuedMessage {
                message, timeout, ..
            } => {
                message.resolve(context);
                timeout.resolve(context);
            }
            Self::QueuedMessageScopedPickupPosition { x, y } => {
                x.resolve(context);
                y.resolve(context);
            }
            Self::FreeMessage { message, .. } => message.resolve(context),
            Self::MessageText { message, .. } => message.resolve(context),
            Self::MessageTimeout { timeout, .. } => timeout.resolve(context),
            Self::MessageBackground { background, .. } => background.resolve(context),
            Self::FreeMessagePosition { x, y, .. } => {
                x.resolve(context);
                y.resolve(context);
            }
            Self::FreeMessageShow { fade, sound, .. } => {
                fade.resolve(context);
                sound.resolve(context);
            }
            Self::FreeMessageHide { fade, .. } => fade.resolve(context),
            Self::StoreBoolean { value, .. } => value.resolve(context),
            Self::StoreInteger { value, .. } => value.resolve(context),
            Self::StoreFloat { value, .. } => value.resolve(context),
            Self::SetBoolean { value, .. } => value.resolve(context),
            Self::SetInteger { value, .. } => value.resolve(context),
            Self::SetFloat { value, .. } => value.resolve(context),
            Self::SetString { value, .. } => value.resolve(context),
            Self::BoxTrigger { x1, y1, x2, y2, .. } => {
                x1.resolve(context);
                y1.resolve(context);
                x2.resolve(context);
                y2.resolve(context);
            }
            Self::SaveAt { x, y, .. } => {
                x.resolve(context);
                y.resolve(context);
            }
            Self::Warp { x, y } => {
                x.resolve(context);
                y.resolve(context);
            }
            Self::InstantWarp { x, y } => {
                x.resolve(context);
                y.resolve(context);
            }
            Self::CreateSpoilerMapIcon { x, y, label, .. } => {
                x.resolve(context);
                y.resolve(context);
                label.resolve(context);
            }
            Self::CreateWarpIcon { x, y, .. } => {
                x.resolve(context);
                y.resolve(context);
            }
            Self::SetWarpIconLabel { label, .. } => label.resolve(context),
            Self::SetShopItemPrice { price, .. } => price.resolve(context),
            Self::SetShopItemName { name, .. } => name.resolve(context),
            Self::SetShopItemDescription { description, .. } => description.resolve(context),
            Self::SetShopItemHidden { hidden, .. } => hidden.resolve(context),
            Self::SetShopItemLocked { locked, .. } => locked.resolve(context),
            Self::SetWheelItemName { name, .. } => name.resolve(context),
            Self::SetWheelItemDescription { description, .. } => description.resolve(context),
            Self::SetWheelItemColor {
                red,
                green,
                blue,
                alpha,
                ..
            } => {
                red.resolve(context);
                green.resolve(context);
                blue.resolve(context);
                alpha.resolve(context);
            }
            Self::SetWheelPinned { pinned, .. } => pinned.resolve(context),
            Self::DebugLog { message } => message.resolve(context),
            Self::Lookup { .. }
            | Self::DefineTimer { .. }
            | Self::FreeMessageUninitialized { .. }
            | Self::MessageDestroy { .. }
            | Self::FreeMessageAlignment { .. }
            | Self::FreeMessageHorizontalAnchor { .. }
            | Self::FreeMessageVerticalAnchor { .. }
            | Self::FreeMessageBoxWidth { .. }
            | Self::FreeMessageCoordinateSystem { .. }
            | Self::BoxTriggerDestroy { .. }
            | Self::BoxTriggerEnterCallback { .. }
            | Self::BoxTriggerLeaveCallback { .. }
            | Self::Save { .. }
            | Self::Equip { .. }
            | Self::Unequip { .. }
            | Self::TriggerClientEvent { .. }
            | Self::TriggerKeybind { .. }
            | Self::EnableServerSync { .. }
            | Self::DisableServerSync { .. }
            | Self::DestroyWarpIcon { .. }
            | Self::SetShopItemIcon { .. }
            | Self::SetWheelItemIcon { .. }
            | Self::SetWheelItemAction { .. }
            | Self::DestroyWheelItem { .. }
            | Self::SwitchWheel { .. }
            | Self::ResetAllWheels {}
            | Self::CloseMenu {} => {}
            Self::CloseWeaponWheel {} => {}
        }
    }
}

struct ZoneOfMatch {
    origin_world_index: usize,
    target_world_index: usize,
    zone: Zone,
}

impl Display for ZoneOfMatch {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.origin_world_index != self.target_world_index {
            write!(f, "<world>{}</>'s ", self.origin_world_index)?;
        }

        write!(f, "{}", self.zone)
    }
}

fn multi_name<I>(names: I) -> CommandString
where
    I: IntoIterator<Item = CommandString>,
{
    let names = names.into_iter();

    let (acc, const_acc) =
        names
            .into_iter()
            .fold((None, String::new()), |(acc, const_acc), name| {
                match name.into_constant() {
                    Ok(name) => {
                        let const_acc = if const_acc.is_empty() {
                            name
                        } else {
                            format!("{const_acc} and {name}")
                        };

                        (acc, const_acc)
                    }
                    Err(name) => {
                        let acc = match acc {
                            None => name,
                            Some(acc) => CommandString::from(Operation {
                                left: CommandString::from(Operation {
                                    left: acc,
                                    operator: Concatenator::Concat,
                                    right: " and ".into(),
                                }),
                                operator: Concatenator::Concat,
                                right: name,
                            }),
                        };

                        (Some(acc), const_acc)
                    }
                }
            });

    match (acc, const_acc.is_empty()) {
        (None, false) => const_acc.into(),
        (None, true) => "Unknown".into(),
        (Some(acc), false) => CommandString::from(Operation {
            left: acc,
            operator: Concatenator::Concat,
            right: format!(" and {const_acc}").into(),
        }),
        (Some(acc), true) => acc,
    }
}

fn multi_price<I>(prices: I) -> CommandInteger
where
    I: IntoIterator<Item = CommandInteger>,
{
    let (acc, const_acc) =
        prices
            .into_iter()
            .fold((None, 0), |(acc, const_acc), price| {
                match price.into_constant() {
                    Ok(price) => (acc, const_acc + price),
                    Err(price) => match acc {
                        None => (Some(price), const_acc),
                        Some(acc) => (
                            Some(CommandInteger::from(Operation {
                                left: acc,
                                operator: ArithmeticOperator::Add,
                                right: price,
                            })),
                            const_acc,
                        ),
                    },
                }
            });

    match (acc, const_acc) {
        (None, 0) => DEFAULT_SHOP_PRICE.into(),
        (None, const_acc) => const_acc.into(),
        (Some(acc), 0) => acc,
        (Some(acc), const_acc) => CommandInteger::from(Operation {
            left: acc,
            operator: ArithmeticOperator::Add,
            right: const_acc.into(),
        }),
    }
}

fn count_in_zone_message(
    matches: Vec<(&Event, &LocDataEntry)>,
    item_metadata: &ItemMetadata,
) -> CommandString {
    if matches.is_empty() {
        return "$0/0$".into();
    }

    const MESSAGE: usize = 2;
    const COUNT: usize = 2;
    const COLOR: usize = 3;

    let len = matches.len();

    CommandString::Multi {
        commands: [
            CommandVoid::SetInteger {
                id: COUNT,
                value: 0.into(),
            },
            CommandVoid::SetString {
                id: MESSAGE,
                value: "".into(),
            },
        ]
        .into_iter()
        .chain(matches.into_iter().map(|(event, entry)| CommandVoid::If {
            condition: CommandBoolean::loc_data_condition(entry.uber_identifier, entry.value),
            command: Box::new(CommandVoid::Multi {
                commands: vec![
                    CommandVoid::SetInteger {
                        id: COUNT,
                        value: CommandInteger::from(Operation {
                            left: CommandInteger::GetInteger { id: COUNT },
                            operator: ArithmeticOperator::Add,
                            right: 1.into(),
                        }),
                    },
                    CommandVoid::If {
                        condition: CommandBoolean::from(Operation {
                            left: CommandString::GetString { id: MESSAGE },
                            operator: EqualityComparator::Equal,
                            right: "".into(),
                        }),
                        command: Box::new(CommandVoid::SetString {
                            id: MESSAGE,
                            value: ": ".into(),
                        }),
                    },
                    CommandVoid::If {
                        condition: CommandBoolean::from(Operation {
                            left: CommandString::GetString { id: MESSAGE },
                            operator: EqualityComparator::NotEqual,
                            right: ": ".into(),
                        }),
                        command: Box::new(CommandVoid::SetString {
                            id: MESSAGE,
                            value: CommandString::from(Operation {
                                left: CommandString::GetString { id: MESSAGE },
                                operator: Concatenator::Concat,
                                right: ", ".into(),
                            }),
                        }),
                    },
                    CommandVoid::SetString {
                        id: MESSAGE,
                        value: CommandString::from(Operation {
                            left: CommandString::GetString { id: MESSAGE },
                            operator: Concatenator::Concat,
                            right: item_metadata.get(&event.command).force_name(), // TODO could this have placeholders again?
                        }),
                    },
                ],
            }),
        }))
        .chain([
            CommandVoid::SetString {
                id: COLOR,
                value: "".into(),
            },
            CommandVoid::If {
                condition: CommandBoolean::from(Operation {
                    left: CommandInteger::GetInteger { id: COUNT },
                    operator: Comparator::Equal,
                    right: (len as i32).into(),
                }),
                command: Box::new(CommandVoid::SetString {
                    id: COLOR,
                    value: "$".into(),
                }),
            },
        ])
        .collect(),
        last: Box::new(CommandString::from(Operation {
            left: CommandString::GetString { id: COLOR },
            operator: Concatenator::Concat,
            right: CommandString::from(Operation {
                left: CommandString::FromInteger {
                    integer: Box::new(CommandInteger::GetInteger { id: COUNT }),
                },
                operator: Concatenator::Concat,
                right: CommandString::from(Operation {
                    left: format!("/{len}").into(),
                    operator: Concatenator::Concat,
                    right: CommandString::from(Operation {
                        left: CommandString::GetString { id: COLOR },
                        operator: Concatenator::Concat,
                        right: CommandString::GetString { id: MESSAGE },
                    }),
                }),
            }),
        })),
    }
}
