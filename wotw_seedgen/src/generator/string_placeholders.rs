use std::iter;

use crate::world::{node_condition, node_trigger};

use super::placement::{command_name, Context};
use rustc_hash::FxHashMap;
use wotw_seedgen_data::Zone;
use wotw_seedgen_logic_language::output::Node;
use wotw_seedgen_seed_language::output::{
    ArithmeticOperator, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
    CommandVoid, CommandZone, Comparator, EqualityComparator, Event, ItemMetadata, Operation,
    StringOrPlaceholder, Trigger,
};

impl Context<'_, '_> {
    pub fn resolve_placeholders(&mut self) {
        let node_trigger_map = self.worlds[0]
            .world
            .graph
            .nodes
            .iter()
            .flat_map(|node| {
                iter::zip(node_trigger(node), node.zone())
                    .map(move |(trigger, zone)| (trigger, (node, zone)))
            })
            .collect::<FxHashMap<_, _>>();

        // TODO prefilter output events to only include things that can succeed a node_trigger_map lookup

        for world in &mut self.worlds {
            let events = world
                .output
                .events
                .iter()
                .filter(|event| node_trigger_map.contains_key(&event.trigger))
                .cloned()
                .collect::<Vec<_>>();
            let context = ResolveContext {
                events,
                node_trigger_map: &node_trigger_map,
                item_metadata: &world.output.item_metadata,
            };

            world.output.events.resolve(&context);
            world.output.command_lookup.resolve(&context);
        }
    }
}

struct ResolveContext<'a> {
    events: Vec<Event>,
    node_trigger_map: &'a FxHashMap<Trigger, (&'a Node, Zone)>,
    item_metadata: &'a ItemMetadata,
}

fn resolve_zone_of(item: &CommandVoid, context: &ResolveContext) -> CommandString {
    context
        .events
        .iter()
        .find(|event| &event.command == item) // TODO there could be multiple
        .and_then(|event| context.node_trigger_map.get(&event.trigger))
        .map_or_else(|| "Unknown".to_string(), |(_, zone)| zone.to_string())
        .into()
}
fn resolve_item_on(trigger: &Trigger, context: &ResolveContext) -> CommandString {
    context
        .events
        .iter()
        .find(|event| &event.trigger == trigger)
        .map_or_else(
            || "Nothing".into(),
            |event| command_name(&event.command, context.item_metadata),
        )
}
fn resolve_count_in_zone(
    items: &[CommandVoid],
    zone: Zone,
    context: &ResolveContext,
) -> CommandString {
    let matches = context
        .events
        .iter()
        .filter_map(|event| {
            context
                .node_trigger_map
                .get(&event.trigger)
                .map(|entry| (event, entry))
        })
        .filter(|(event, (_, z))| *z == zone && items.contains(&event.command))
        .collect::<Vec<_>>();

    if matches.is_empty() {
        "$0/0$".into()
    } else {
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
            .chain(matches.iter().map(|(event, (node, _))| CommandVoid::If {
                condition: node_condition(node).unwrap(),
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
                                    left: Box::new(CommandString::GetString { id: 2 }),
                                    right: Box::new(": ".into()),
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
                                    left: Box::new(CommandString::GetString { id: 2 }),
                                    right: Box::new(", ".into()),
                                },
                            }),
                        },
                        CommandVoid::SetString {
                            id: 2,
                            value: CommandString::Concatenate {
                                left: Box::new(CommandString::GetString { id: 2 }),
                                right: Box::new(command_name(
                                    &event.command,
                                    context.item_metadata,
                                )), // TODO could this have placeholders again?
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
                left: Box::new(CommandString::GetString { id: 3 }),
                right: Box::new(CommandString::Concatenate {
                    left: Box::new(CommandString::FromInteger {
                        integer: Box::new(CommandInteger::GetInteger { id: 2 }),
                    }),
                    right: Box::new(CommandString::Concatenate {
                        left: Box::new(format!("/{}", matches.len()).into()),
                        right: Box::new(CommandString::Concatenate {
                            left: Box::new(CommandString::GetString { id: 3 }),
                            right: Box::new(CommandString::GetString { id: 2 }),
                        }),
                    }),
                }),
            }),
        }
    }
}

trait ResolvePlaceholders {
    fn resolve(&mut self, context: &ResolveContext);
}
impl<T: ResolvePlaceholders> ResolvePlaceholders for Vec<T> {
    fn resolve(&mut self, context: &ResolveContext) {
        for t in self {
            t.resolve(context);
        }
    }
}
impl<T: ResolvePlaceholders> ResolvePlaceholders for Option<T> {
    fn resolve(&mut self, context: &ResolveContext) {
        if let Some(t) = self {
            t.resolve(context);
        }
    }
}
impl<Item: ResolvePlaceholders, Operator> ResolvePlaceholders for Operation<Item, Operator> {
    fn resolve(&mut self, context: &ResolveContext) {
        self.left.resolve(context);
        self.right.resolve(context);
    }
}
impl ResolvePlaceholders for Event {
    fn resolve(&mut self, context: &ResolveContext) {
        if let Trigger::Condition(condition) = &mut self.trigger {
            condition.resolve(context);
        }
        self.command.resolve(context);
    }
}
impl ResolvePlaceholders for Command {
    fn resolve(&mut self, context: &ResolveContext) {
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
    fn resolve(&mut self, context: &ResolveContext) {
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
            Self::IsInHitbox { x1, y1, x2, y2 } => {
                x1.resolve(context);
                y1.resolve(context);
                x2.resolve(context);
                y2.resolve(context);
            }
            Self::Constant { .. } | Self::FetchBoolean { .. } | Self::GetBoolean { .. } => {}
        }
    }
}
impl ResolvePlaceholders for CommandInteger {
    fn resolve(&mut self, context: &ResolveContext) {
        match self {
            Self::Multi { commands, last } => {
                commands.resolve(context);
                last.resolve(context);
            }
            Self::Arithmetic { operation } => operation.resolve(context),
            Self::FromFloat { float } => float.resolve(context),
            Self::Constant { .. } | Self::FetchInteger { .. } | Self::GetInteger { .. } => {}
        }
    }
}
impl ResolvePlaceholders for CommandFloat {
    fn resolve(&mut self, context: &ResolveContext) {
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
    fn resolve(&mut self, context: &ResolveContext) {
        match self {
            Self::Constant { value } => {
                *self = match value {
                    StringOrPlaceholder::Value(_) => return,
                    StringOrPlaceholder::ZoneOfPlaceholder(item) => resolve_zone_of(item, context),
                    StringOrPlaceholder::ItemOnPlaceholder(trigger) => {
                        resolve_item_on(trigger, context)
                    }
                    StringOrPlaceholder::CountInZonePlaceholder(items, zone) => {
                        resolve_count_in_zone(items, *zone, context)
                    }
                }
            }
            Self::Multi { commands, last } => {
                commands.resolve(context);
                last.resolve(context);
            }
            Self::Concatenate { left, right } => {
                left.resolve(context);
                right.resolve(context);
            }
            Self::FromBoolean { boolean } => boolean.resolve(context),
            Self::FromInteger { integer } => integer.resolve(context),
            Self::FromFloat { float } => float.resolve(context),
            Self::GetString { .. } | Self::WorldName { .. } => {}
        }
    }
}
impl ResolvePlaceholders for CommandZone {
    fn resolve(&mut self, context: &ResolveContext) {
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
    fn resolve(&mut self, context: &ResolveContext) {
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
            Self::Warp { x, y } => {
                x.resolve(context);
                y.resolve(context);
            }
            Self::SetSpoilerMapIcon { label, .. } => label.resolve(context),
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
            Self::Lookup { .. }
            | Self::DefineTimer { .. }
            | Self::MessageDestroy { .. }
            | Self::FreeMessageAlignment { .. }
            | Self::FreeMessageScreenPosition { .. }
            | Self::Save {}
            | Self::SaveToMemory {}
            | Self::Equip { .. }
            | Self::Unequip { .. }
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
            | Self::DebugLog { .. } => {}
        }
    }
}
