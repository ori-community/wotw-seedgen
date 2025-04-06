use super::{
    intermediate::Literal, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
    CommandVoid, CommandZone, Event, Operation, StringOrPlaceholder, Trigger,
};
use itertools::Itertools;
use std::fmt::{self, Display};

impl Display for Literal {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Literal::UberIdentifier(value) => value.fmt(f),
            Literal::Boolean(value) => value.fmt(f),
            Literal::Integer(value) => value.fmt(f),
            Literal::Float(value) => value.fmt(f),
            Literal::String(value) => value.fmt(f),
            Literal::Constant(value) => value.fmt(f),
            Literal::IconAsset(path) => write!(f, "icon asset: \"{path}\""),
            Literal::CustomIcon(path) => write!(f, "custom icon: \"{path}\""),
        }
    }
}
impl Display for StringOrPlaceholder {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StringOrPlaceholder::Value(string) => write!(f, "\"{string}\""),
            StringOrPlaceholder::ZoneOfPlaceholder(action) => write!(f, "zone_of({action})"),
            StringOrPlaceholder::ItemOnPlaceholder(trigger) => write!(f, "item_on({trigger})"),
            StringOrPlaceholder::CountInZonePlaceholder(actions, zone) => {
                write!(
                    f,
                    "count_in_zone({zone}, [{}])",
                    actions.iter().format(", ")
                )
            }
        }
    }
}

impl Display for Event {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "on {} {}", self.trigger, self.command)
    }
}

impl Display for Trigger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Trigger::ClientEvent(client) => client.fmt(f),
            // TODO not sure I decided on this syntax yet
            Trigger::Binding(uber_identifier) => write!(f, "[{uber_identifier}]"),
            Trigger::Condition(condition) => condition.fmt(f),
        }
    }
}

impl<Item: Display, Operator: Display> Display for Operation<Item, Operator> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} {} {}", self.left, self.operator, self.right)
    }
}

impl Display for Command {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Command::Boolean(command) => command.fmt(f),
            Command::Integer(command) => command.fmt(f),
            Command::Float(command) => command.fmt(f),
            Command::String(command) => command.fmt(f),
            Command::Zone(command) => command.fmt(f),
            Command::Void(command) => command.fmt(f),
        }
    }
}

impl Display for CommandBoolean {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandBoolean::Constant { value } => value.fmt(f),
            CommandBoolean::Multi { commands, last } => {
                write!(f, "{{ {}, {} }}", commands.iter().format(", "), last)
            }
            CommandBoolean::CompareBoolean { operation } => operation.fmt(f),
            CommandBoolean::CompareInteger { operation } => operation.fmt(f),
            CommandBoolean::CompareFloat { operation } => operation.fmt(f),
            CommandBoolean::CompareString { operation } => operation.fmt(f),
            CommandBoolean::CompareZone { operation } => operation.fmt(f),
            CommandBoolean::LogicOperation { operation } => operation.fmt(f),
            CommandBoolean::FetchBoolean { uber_identifier } => {
                write!(f, "fetch({uber_identifier})")
            }
            CommandBoolean::GetBoolean { id } => write!(f, "get_boolean({id})"),
            CommandBoolean::IsInHitbox { x1, y1, x2, y2 } => {
                write!(f, "is_in_hitbox({x1}, {y1}, {x2}, {y2})")
            }
        }
    }
}

impl Display for CommandInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandInteger::Constant { value } => value.fmt(f),
            CommandInteger::Multi { commands, last } => {
                write!(f, "{{ {}, {} }}", commands.iter().format(", "), last)
            }
            CommandInteger::Arithmetic { operation } => operation.fmt(f),
            CommandInteger::FetchInteger { uber_identifier } => {
                write!(f, "fetch({uber_identifier})")
            }
            CommandInteger::GetInteger { id } => write!(f, "get_integer({id})"),
            CommandInteger::FromFloat { float } => write!(f, "to_integer({float})"),
        }
    }
}

impl Display for CommandFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandFloat::Constant { value } => value.fmt(f),
            CommandFloat::Multi { commands, last } => {
                write!(f, "{{ {}, {} }}", commands.iter().format(", "), last)
            }
            CommandFloat::Arithmetic { operation } => operation.fmt(f),
            CommandFloat::FetchFloat { uber_identifier } => write!(f, "fetch({uber_identifier})"),
            CommandFloat::GetFloat { id } => write!(f, "get_float({id})"),
            CommandFloat::FromInteger { integer } => write!(f, "to_float({integer})"),
        }
    }
}

impl Display for CommandString {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandString::Constant { value } => write!(f, "{value}"),
            CommandString::Multi { commands, last } => {
                write!(f, "{{ {}, {} }}", commands.iter().format(", "), last)
            }
            CommandString::Concatenate { left, right } => write!(f, "{left} + {right}"),
            CommandString::GetString { id } => write!(f, "get_string({id})"),
            CommandString::WorldName { index } => write!(f, "world_name({index})"),
            CommandString::FromBoolean { boolean } => write!(f, "to_string({boolean})"),
            CommandString::FromInteger { integer } => write!(f, "to_string({integer})"),
            CommandString::FromFloat { float } => write!(f, "to_string({float})"),
        }
    }
}

impl Display for CommandZone {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandZone::Constant { value } => value.fmt(f),
            CommandZone::Multi { commands, last } => {
                write!(f, "{{ {}, {} }}", commands.iter().format(", "), last)
            }
            CommandZone::CurrentZone {} => write!(f, "current_zone()"),
            CommandZone::CurrentMapZone {} => write!(f, "current_map_zone()"),
        }
    }
}

impl Display for CommandVoid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandVoid::Multi { commands } => write!(f, "{{ {} }}", commands.iter().format(" ")),
            CommandVoid::Lookup { index } => write!(f, "lookup({index})"),
            CommandVoid::If { condition, command } => write!(f, "if ({condition}) {{ {command} }}"),
            CommandVoid::DefineTimer { toggle, timer } => {
                write!(f, "define_timer({toggle}, {timer})")
            }
            // TODO this logic depends on implementation details of compilation
            CommandVoid::QueuedMessage {
                id,
                priority,
                message,
                timeout,
            } => match timeout {
                None => write!(f, "item_message({message})"),
                Some(timeout) => match id {
                    None => {
                        let function = if *priority {
                            "priority_message"
                        } else {
                            "item_message_with_timeout"
                        };
                        write!(f, "{function}({message}, {timeout})")
                    }
                    Some(id) => {
                        write!(f, "free_message({id}, {message}, {timeout})")
                    }
                },
            },
            CommandVoid::FreeMessage { id, message } => {
                write!(f, "free_message({id}, {message})")
            }
            CommandVoid::MessageDestroy { id } => {
                write!(f, "destroy_message({id})")
            }
            CommandVoid::MessageText { id, message } => {
                write!(f, "set_message_text({id}, {message})")
            }
            CommandVoid::MessageTimeout { id, timeout } => {
                write!(f, "set_message_timeout({id}, {timeout})")
            }
            CommandVoid::MessageBackground { id, background } => {
                write!(f, "set_message_background({id}, {background})")
            }
            CommandVoid::FreeMessagePosition { id, x, y } => {
                write!(f, "set_message_position({id}, {x}, {y})")
            }
            CommandVoid::FreeMessageAlignment { id, alignment } => {
                write!(f, "set_message_alignment({id}, {alignment})")
            }
            CommandVoid::FreeMessageScreenPosition {
                id,
                screen_position,
            } => write!(f, "set_message_screen_position({id}, {screen_position})"),
            CommandVoid::SetMapMessage { value } => write!(f, "set_map_message({value})"),
            CommandVoid::StoreBoolean {
                uber_identifier,
                value,
                trigger_events,
            } => write!(
                f,
                "store_boolean{}({uber_identifier}, {value})",
                if *trigger_events {
                    ""
                } else {
                    "_without_triggers"
                }
            ),
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                trigger_events,
            } => write!(
                f,
                "store_integer{}({uber_identifier}, {value})",
                if *trigger_events {
                    ""
                } else {
                    "_without_triggers"
                }
            ),
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                trigger_events,
            } => write!(
                f,
                "store_float{}({uber_identifier}, {value})",
                if *trigger_events {
                    ""
                } else {
                    "_without_triggers"
                }
            ),
            CommandVoid::SetBoolean { id, value } => write!(f, "set_boolean({id}, {value})"),
            CommandVoid::SetInteger { id, value } => write!(f, "set_integer({id}, {value})"),
            CommandVoid::SetFloat { id, value } => write!(f, "set_float({id}, {value})"),
            CommandVoid::SetString { id, value } => write!(f, "set_string({id}, {value})"),
            CommandVoid::Save {} => write!(f, "save()"),
            CommandVoid::SaveToMemory {} => write!(f, "save_to_memory()"),
            CommandVoid::Warp { x, y } => write!(f, "warp({x}, {y})"),
            CommandVoid::Equip { slot, equipment } => write!(f, "equip({slot}, {equipment})"),
            CommandVoid::Unequip { equipment } => write!(f, "unequip({equipment})"),
            CommandVoid::TriggerKeybind { bind } => write!(f, "trigger_keybind({bind})"),
            CommandVoid::EnableServerSync { uber_identifier } => {
                write!(f, "enable_server_sync({uber_identifier})")
            }
            CommandVoid::DisableServerSync { uber_identifier } => {
                write!(f, "disable_server_sync({uber_identifier})")
            }
            CommandVoid::SetSpoilerMapIcon {
                location,
                icon,
                label,
            } => write!(f, "set_map_icon({location}, {icon}, {label})"),
            CommandVoid::CreateWarpIcon { id, x, y } => {
                write!(f, "create_warp_icon({id}, {x}, {y})")
            }
            CommandVoid::SetWarpIconLabel { id, label } => {
                write!(f, "set_warp_icon_label({id}, {label})")
            }
            CommandVoid::DestroyWarpIcon { id } => write!(f, "destroy_warp_icon({id})"),
            CommandVoid::SetShopItemPrice {
                uber_identifier,
                price,
            } => write!(f, "set_shop_item_price({uber_identifier}, {price})"),
            CommandVoid::SetShopItemName {
                uber_identifier,
                name,
            } => write!(f, "set_shop_item_name({uber_identifier}, {name})"),
            CommandVoid::SetShopItemDescription {
                uber_identifier,
                description,
            } => write!(
                f,
                "set_shop_item_description({uber_identifier}, {description})"
            ),
            CommandVoid::SetShopItemIcon {
                uber_identifier,
                icon,
            } => write!(f, "set_shop_item_icon({uber_identifier}, {icon})"),
            CommandVoid::SetShopItemHidden {
                uber_identifier,
                hidden,
            } => write!(f, "set_shop_item_hidden({uber_identifier}, {hidden})"),
            CommandVoid::SetShopItemLocked {
                uber_identifier,
                locked,
            } => write!(f, "set_shop_item_locked({uber_identifier}, {locked})"),
            CommandVoid::SetWheelItemName {
                wheel,
                position,
                name,
            } => write!(f, "set_wheel_item_name({wheel}, {position}, {name})"),
            CommandVoid::SetWheelItemDescription {
                wheel,
                position,
                description,
            } => write!(
                f,
                "set_wheel_item_description({wheel}, {position}, {description})"
            ),
            CommandVoid::SetWheelItemIcon {
                wheel,
                position,
                icon,
            } => write!(f, "set_wheel_item_icon({wheel}, {position}, {icon})"),
            CommandVoid::SetWheelItemColor {
                wheel,
                position,
                red,
                green,
                blue,
                alpha,
            } => write!(
                f,
                "set_wheel_item_color({wheel}, {position}, {red}, {green}, {blue}, {alpha})"
            ),
            CommandVoid::SetWheelItemAction {
                wheel,
                position,
                bind,
                action,
            } => write!(
                f,
                "set_wheel_item_action({wheel}, {position}, {bind}, {action})"
            ),
            CommandVoid::DestroyWheelItem { wheel, position } => {
                write!(f, "destroy_wheel_item({wheel}, {position})")
            }
            CommandVoid::SwitchWheel { wheel } => write!(f, "switch_wheel({wheel})"),
            CommandVoid::SetWheelPinned { wheel, pinned } => {
                write!(f, "set_wheel_pinned({wheel}, {pinned})")
            }
            CommandVoid::ResetAllWheels {} => write!(f, "reset_all_wheels()"),
            CommandVoid::DebugLog { message } => write!(f, "debug_log({message})"),
        }
    }
}
