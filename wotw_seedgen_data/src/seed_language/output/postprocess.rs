mod price_noise;

use super::{
    ArithmeticOperator, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
    CommandVoid, CommandZone, Comparator, Concatenator, EqualityComparator, Event,
    IntermediateOutput, ItemMetadata, Operation, StringOrPlaceholder, Trigger,
};

use crate::{
    assets::{LocData, LocDataEntry},
    seed_language::output::{
        item_metadata::{random_shop_description, DEFAULT_SHOP_PRICE},
        postprocess::price_noise::PriceNoise,
        IntoConstant,
    },
    ShopKind, Zone,
};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;

// TODO maybe zone_of should be a typed zone placeholder?
#[derive(Debug, Clone, Default)]
pub struct PlaceholderMap {
    pub strings: FxHashMap<StringOrPlaceholder, CommandString>,
}

impl IntermediateOutput {
    // TODO only set shop data defaults for things that aren't set by the seed?
    /// Inserts additional commands into the output to:
    ///
    /// - Set reasonable shop data defaults for items placed in shops
    /// - Populate the spoiler map
    ///
    /// Also returns a [`PlaceholderMap`] which contains resolutions for all contained placeholders.
    pub fn postprocess(&mut self, loc_data: &LocData, rng: &mut Pcg64Mcg) -> PlaceholderMap {
        let mut postprocessor = Postprocessor::new(self, loc_data);

        self.resolve(&mut postprocessor);

        let mut extra_events = vec![];
        let price_noise = PriceNoise::new();

        for (trigger, events) in postprocessor.loc_data_events {
            let shop_identifier = match trigger {
                Trigger::Condition(CommandBoolean::FetchBoolean { uber_identifier })
                    if matches!(
                        uber_identifier.shop_kind(),
                        ShopKind::Opherlike | ShopKind::Map
                    ) =>
                {
                    Some(*uber_identifier)
                }
                _ => None,
            };
            let map_position = &postprocessor.loc_data_triggers[trigger].map_position;

            if shop_identifier.is_none() && map_position.is_none() {
                continue;
            }

            let mut matches = vec![];

            let names = events.into_iter().filter_map(|event| {
                let metadata = self.item_metadata.get(&event.command);

                let name = metadata.try_force_name()?;

                matches.push(metadata);

                Some(name)
            });

            let name = multi_name(names);

            if let Trigger::Condition(CommandBoolean::FetchBoolean { uber_identifier }) = trigger {
                let uber_identifier = *uber_identifier;

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
                            name: name.clone(),
                        }));

                        let mut descriptions =
                            matches.iter().filter_map(|metadata| metadata.description());

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

            let location = postprocessor.loc_data_triggers[trigger];
            if let Some(map_position) = location.map_position {
                let icon = matches
                    .iter()
                    .find_map(|metadata| metadata.try_force_map_icon())
                    .unwrap_or_default();

                extra_events.push(Event::on_reload(CommandVoid::CreateSpoilerMapIcon {
                    icon,
                    x: map_position.x.into(),
                    y: map_position.y.into(),
                    label: name,
                }));
            }
        }

        let placeholder_map = postprocessor.placeholder_map;

        self.events.splice(0..0, extra_events);

        placeholder_map
    }
}

struct Postprocessor<'output, 'locdata> {
    events: &'output [Event],
    loc_data_triggers: FxHashMap<Trigger, &'locdata LocDataEntry>,
    loc_data_events: FxHashMap<&'output Trigger, Vec<&'output Event>>,
    item_metadata: &'output ItemMetadata,
    placeholder_map: PlaceholderMap,
}

impl<'output, 'locdata> Postprocessor<'output, 'locdata> {
    fn new(output: &'output IntermediateOutput, loc_data: &'locdata LocData) -> Self {
        let loc_data_triggers = loc_data
            .entries
            .iter()
            .map(|entry| {
                (
                    Trigger::loc_data_trigger(entry.uber_identifier, entry.value),
                    entry,
                )
            })
            .collect::<FxHashMap<_, _>>();

        let mut loc_data_events = FxHashMap::<_, Vec<_>>::default();

        for event in output
            .events
            .iter()
            .filter(|event| loc_data_triggers.contains_key(&event.trigger))
        {
            loc_data_events
                .entry(&event.trigger)
                .or_default()
                .push(event);
        }

        Self {
            events: &output.events,
            loc_data_triggers,
            loc_data_events,
            item_metadata: &output.item_metadata,
            placeholder_map: PlaceholderMap::default(),
        }
    }

    fn resolve_zone_of(&self, item: &CommandVoid) -> CommandString {
        let mut matches = self
            .loc_data_events
            .values()
            .flatten()
            .filter(|event| &event.command == item)
            .filter_map(|event| self.loc_data_triggers.get(&event.trigger));

        match matches.next() {
            None => "Unknown".into(),
            Some(first) => matches
                .fold(first.zone.to_string(), |acc, entry| {
                    format!("{acc} or {}", entry.zone)
                })
                .into(),
        }
    }

    fn resolve_item_on(&self, trigger: &Trigger) -> CommandString {
        let names = self
            .events
            .iter()
            .filter(|event| &event.trigger == trigger)
            .filter_map(|event| self.item_metadata.get(&event.command).try_force_name());

        multi_name(names)
    }

    fn resolve_count_in_zone(&self, items: &[CommandVoid], zone: Zone) -> CommandString {
        let matches = self
            .loc_data_events
            .values()
            .flatten()
            .filter_map(|event| {
                self.loc_data_triggers
                    .get(&event.trigger)
                    .map(|entry| (event, entry))
            })
            .filter(|(event, entry)| entry.zone == zone && items.contains(&event.command))
            .collect::<Vec<_>>();

        if matches.is_empty() {
            return "$0/0$".into();
        }

        CommandString::Multi {
            commands: [
                CommandVoid::SetInteger {
                    id: 2,
                    value: 0.into(),
                },
                CommandVoid::SetString {
                    id: 2,
                    value: "".into(),
                },
            ]
            .into_iter()
            .chain(matches.iter().map(|(event, entry)| CommandVoid::If {
                condition: CommandBoolean::loc_data_condition(entry.uber_identifier, entry.value),
                command: Box::new(CommandVoid::Multi {
                    commands: vec![
                        CommandVoid::SetInteger {
                            id: 2,
                            value: CommandInteger::Arithmetic {
                                operation: Box::new(Operation {
                                    left: CommandInteger::GetInteger { id: 2 },
                                    operator: ArithmeticOperator::Add,
                                    right: 1.into(),
                                }),
                            },
                        },
                        CommandVoid::If {
                            condition: CommandBoolean::CompareString {
                                operation: Box::new(Operation {
                                    left: CommandString::GetString { id: 2 },
                                    operator: EqualityComparator::Equal,
                                    right: "".into(),
                                }),
                            },
                            command: Box::new(CommandVoid::SetString {
                                id: 2,
                                value: CommandString::Concatenate {
                                    operation: Box::new(Operation {
                                        left: CommandString::GetString { id: 2 },
                                        operator: Concatenator::Concat,
                                        right: ": ".into(),
                                    }),
                                },
                            }),
                        },
                        CommandVoid::If {
                            condition: CommandBoolean::CompareString {
                                operation: Box::new(Operation {
                                    left: CommandString::GetString { id: 2 },
                                    operator: EqualityComparator::NotEqual,
                                    right: ": ".into(),
                                }),
                            },
                            command: Box::new(CommandVoid::SetString {
                                id: 2,
                                value: CommandString::Concatenate {
                                    operation: Box::new(Operation {
                                        left: CommandString::GetString { id: 2 },
                                        operator: Concatenator::Concat,
                                        right: ", ".into(),
                                    }),
                                },
                            }),
                        },
                        CommandVoid::SetString {
                            id: 2,
                            value: CommandString::Concatenate {
                                operation: Box::new(Operation {
                                    left: CommandString::GetString { id: 2 },
                                    operator: Concatenator::Concat,
                                    right: self.item_metadata.get(&event.command).force_name(), // TODO could this have placeholders again?
                                }),
                            },
                        },
                    ],
                }),
            }))
            .chain([
                CommandVoid::SetString {
                    id: 3,
                    value: "".into(),
                },
                CommandVoid::If {
                    condition: CommandBoolean::CompareInteger {
                        operation: Box::new(Operation {
                            left: CommandInteger::GetInteger { id: 2 },
                            operator: Comparator::Equal,
                            right: (matches.len() as i32).into(),
                        }),
                    },
                    command: Box::new(CommandVoid::SetString {
                        id: 3,
                        value: "$".into(),
                    }),
                },
            ])
            .collect(),
            last: Box::new(CommandString::Concatenate {
                operation: Box::new(Operation {
                    left: CommandString::GetString { id: 3 },
                    operator: Concatenator::Concat,
                    right: CommandString::Concatenate {
                        operation: Box::new(Operation {
                            left: CommandString::FromInteger {
                                integer: Box::new(CommandInteger::GetInteger { id: 2 }),
                            },
                            operator: Concatenator::Concat,
                            right: CommandString::Concatenate {
                                operation: Box::new(Operation {
                                    left: format!("/{}", matches.len()).into(),
                                    operator: Concatenator::Concat,
                                    right: CommandString::Concatenate {
                                        operation: Box::new(Operation {
                                            left: CommandString::GetString { id: 3 },
                                            operator: Concatenator::Concat,
                                            right: CommandString::GetString { id: 2 },
                                        }),
                                    },
                                }),
                            },
                        }),
                    },
                }),
            }),
        }
    }
}

// TODO maybe this adds stats tracking?
trait ResolvePlaceholders {
    fn resolve(&self, context: &mut Postprocessor);
}

impl<T: ResolvePlaceholders> ResolvePlaceholders for Vec<T> {
    fn resolve(&self, context: &mut Postprocessor) {
        for t in self {
            t.resolve(context);
        }
    }
}

impl<T: ResolvePlaceholders> ResolvePlaceholders for Option<T> {
    fn resolve(&self, context: &mut Postprocessor) {
        if let Some(t) = self {
            t.resolve(context);
        }
    }
}

impl<Item: ResolvePlaceholders, Operator> ResolvePlaceholders for Operation<Item, Operator> {
    fn resolve(&self, context: &mut Postprocessor) {
        self.left.resolve(context);
        self.right.resolve(context);
    }
}

impl ResolvePlaceholders for IntermediateOutput {
    fn resolve(&self, context: &mut Postprocessor) {
        self.events.resolve(context);
        self.command_lookup.resolve(context);
    }
}

impl ResolvePlaceholders for Event {
    fn resolve(&self, context: &mut Postprocessor) {
        self.trigger.resolve(context);
        self.command.resolve(context);
    }
}

impl ResolvePlaceholders for Trigger {
    fn resolve(&self, context: &mut Postprocessor) {
        if let Self::Condition(condition) = self {
            condition.resolve(context);
        }
    }
}

impl ResolvePlaceholders for Command {
    fn resolve(&self, context: &mut Postprocessor) {
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
    fn resolve(&self, context: &mut Postprocessor) {
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
    fn resolve(&self, context: &mut Postprocessor) {
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
    fn resolve(&self, context: &mut Postprocessor) {
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
    fn resolve(&self, context: &mut Postprocessor) {
        match self {
            Self::Constant { value } => {
                if context.placeholder_map.strings.contains_key(value) {
                    return;
                }

                let resolved = match value {
                    StringOrPlaceholder::Value(_) => return,
                    StringOrPlaceholder::ZoneOfPlaceholder(item) => context.resolve_zone_of(item),
                    StringOrPlaceholder::ItemOnPlaceholder(trigger) => {
                        context.resolve_item_on(trigger)
                    }
                    StringOrPlaceholder::CountInZonePlaceholder(items, zone) => {
                        context.resolve_count_in_zone(items, *zone)
                    }
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
    fn resolve(&self, context: &mut Postprocessor) {
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
    fn resolve(&self, context: &mut Postprocessor) {
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
            Self::SetMapMessage { value } => value.resolve(context),
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

pub fn multi_name<I>(names: I) -> CommandString
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

pub fn multi_price<I>(prices: I) -> CommandInteger
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
