use crate::seed_language::output::{AsConstant, TriggerCondition};

use super::{
    intermediate::Literal, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
    CommandVoid, CommandZone, Event, Operation, StringOrPlaceholder, Trigger,
};
use itertools::Itertools;
use logos::Logos;
use std::{
    fmt::{self, Display},
    iter,
};

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
            StringOrPlaceholder::ZoneOfPlaceholder(uber_identifiers) => {
                write!(f, "zone_of({})", uber_identifiers.iter().format(", "))
            }
            StringOrPlaceholder::ItemOnPlaceholder(trigger) => write!(f, "item_on({trigger})"),
            StringOrPlaceholder::CountInZonePlaceholder(uber_identifiers, zone) => {
                write!(
                    f,
                    "count_in_zone({zone}, [{uber_identifiers}])",
                    uber_identifiers = uber_identifiers.iter().format(", ")
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
            Trigger::Binding(uber_identifier) => write!(f, "change {uber_identifier}"),
            Trigger::Condition(condition) => condition.fmt(f),
        }
    }
}

impl Display for TriggerCondition {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.condition.fmt(f)
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
            CommandBoolean::FunctionArgument { index } => write!(f, "boolean_arg({index})"),
            CommandBoolean::Multi { commands, last } => multi(f, commands, last),
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
            CommandBoolean::IsInBox { x1, y1, x2, y2 } => {
                write!(f, "is_in_box({x1}, {y1}, {x2}, {y2})")
            }
        }
    }
}

impl Display for CommandInteger {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandInteger::Constant { value } => value.fmt(f),
            CommandInteger::FunctionArgument { index } => write!(f, "integer_arg({index})"),
            CommandInteger::Multi { commands, last } => multi(f, commands, last),
            CommandInteger::Arithmetic { operation } => operation.fmt(f),
            CommandInteger::FetchInteger { uber_identifier } => {
                write!(f, "fetch({uber_identifier})")
            }
            CommandInteger::GetInteger { id } => write!(f, "get_integer({id})"),
            CommandInteger::FromFloat { float } => write!(f, "to_integer({float})"),
            CommandInteger::StringLength { string } => write!(f, "string_length({string})"),
        }
    }
}

impl Display for CommandFloat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandFloat::Constant { value } => value.fmt(f),
            CommandFloat::FunctionArgument { index } => write!(f, "float_arg({index})"),
            CommandFloat::Multi { commands, last } => multi(f, commands, last),
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
            CommandString::FunctionArgument { index } => write!(f, "string_arg({index})"),
            CommandString::Multi { commands, last } => multi(f, commands, last),
            CommandString::Concatenate { operation } => operation.fmt(f),
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
            CommandZone::Multi { commands, last } => multi(f, commands, last),
            CommandZone::CurrentZone {} => write!(f, "current_zone()"),
            CommandZone::CurrentMapZone {} => write!(f, "current_map_zone()"),
        }
    }
}

impl Display for CommandVoid {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            CommandVoid::Multi { commands } => write!(f, "{{ {} }}", commands.iter().format(" ")),
            CommandVoid::CallFunction {
                booleans,
                integers,
                floats,
                strings,
                index,
            } => {
                if booleans.is_empty()
                    && integers.is_empty()
                    && floats.is_empty()
                    && strings.is_empty()
                {
                    write!(f, "call_function({index})")
                } else {
                    write!(f, "{{ ")?;

                    for boolean in booleans {
                        write!(f, "boolean_arg({boolean})")?;
                    }
                    for integer in integers {
                        write!(f, "integer_arg({integer})")?;
                    }
                    for float in floats {
                        write!(f, "float_arg({float})")?;
                    }
                    for string in strings {
                        write!(f, "string_arg({string})")?;
                    }

                    write!(f, "call_function({index}) }}")
                }
            }
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
            CommandVoid::QueuedMessageScopedPickupPosition { x, y } => {
                write!(f, "queued_message_scoped_pickup_position({x}, {y})")
            }
            CommandVoid::FreeMessage { id, message } => {
                write!(f, "free_message({id}, {message})")
            }
            Self::FreeMessageUninitialized { id } => write!(f, "free_message_uninitialized({id})"),
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
            CommandVoid::FreeMessageHorizontalAnchor {
                id,
                horizontal_anchor,
            } => write!(
                f,
                "set_message_horizontal_anchor({id}, {horizontal_anchor})"
            ),
            CommandVoid::FreeMessageVerticalAnchor {
                id,
                vertical_anchor,
            } => write!(f, "set_message_vertical_anchor({id}, {vertical_anchor})"),
            CommandVoid::FreeMessageBoxWidth { id, width } => {
                write!(f, "set_message_box_width({id}, {width})")
            }
            CommandVoid::FreeMessageCoordinateSystem {
                id,
                coordinate_system,
            } => write!(
                f,
                "set_message_coordinate_system({id}, {coordinate_system})"
            ),
            CommandVoid::FreeMessageShow { id, fade, sound } => {
                write!(f, "free_message_show({id}, {fade}, {sound})")
            }
            CommandVoid::FreeMessageHide { id, fade } => {
                write!(f, "free_message_hide({id}, {fade})")
            }
            CommandVoid::StoreBoolean {
                uber_identifier,
                value,
                trigger_events,
            } => write!(
                f,
                "store_boolean{}({uber_identifier}, {value})",
                store_suffix(*trigger_events)
            ),
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                trigger_events,
            } => write!(
                f,
                "store_integer{}({uber_identifier}, {value})",
                store_suffix(*trigger_events)
            ),
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                trigger_events,
            } => write!(
                f,
                "store_float{}({uber_identifier}, {value})",
                store_suffix(*trigger_events)
            ),
            CommandVoid::SetBoolean { id, value } => write!(f, "set_boolean({id}, {value})"),
            CommandVoid::SetInteger { id, value } => write!(f, "set_integer({id}, {value})"),
            CommandVoid::SetFloat { id, value } => write!(f, "set_float({id}, {value})"),
            CommandVoid::SetString { id, value } => write!(f, "set_string({id}, {value})"),
            CommandVoid::BoxTrigger { id, x1, y1, x2, y2 } => {
                write!(f, "box_trigger({id}, {x1}, {y1}, {x2}, {y2})")
            }
            CommandVoid::BoxTriggerDestroy { id } => {
                write!(f, "box_trigger_destroy({id})")
            }
            CommandVoid::BoxTriggerEnterCallback { id, action } => {
                write!(f, "box_trigger_enter_callback({id}, {action})")
            }
            CommandVoid::BoxTriggerLeaveCallback { id, action } => {
                write!(f, "box_trigger_leave_callback({id}, {action})")
            }
            CommandVoid::Save { to_disk } => write!(f, "save{}()", save_suffix(*to_disk)),
            CommandVoid::SaveAt { to_disk, x, y } => {
                write!(f, "save{}_at({x}, {y})", save_suffix(*to_disk))
            }
            CommandVoid::Warp { x, y } => write!(f, "warp({x}, {y})"),
            CommandVoid::InstantWarp { x, y } => write!(f, "instant_warp({x}, {y})"),
            CommandVoid::Equip { slot, equipment } => write!(f, "equip({slot}, {equipment})"),
            CommandVoid::Unequip { equipment } => write!(f, "unequip({equipment})"),
            CommandVoid::TriggerClientEvent { client_event } => {
                write!(f, "trigger_client_event({client_event})")
            }
            CommandVoid::TriggerKeybind { bind } => write!(f, "trigger_keybind({bind})"),
            CommandVoid::EnableServerSync { uber_identifier } => {
                write!(f, "enable_server_sync({uber_identifier})")
            }
            CommandVoid::DisableServerSync { uber_identifier } => {
                write!(f, "disable_server_sync({uber_identifier})")
            }
            CommandVoid::CreateSpoilerMapIcon { icon, x, y, label } => {
                write!(f, "create_spoiler_map_icon({icon}, {x}, {y}, {label})")
            }
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
            CommandVoid::CloseMenu {} => write!(f, "close_menu()"),
            CommandVoid::CloseWeaponWheel {} => write!(f, "close_weapon_wheel()"),
            CommandVoid::DebugLog { message } => write!(f, "debug_log({message})"),
            CommandVoid::DealEnemyDamage { amount } => write!(f, "deal_enemy_damage({amount})"),
            CommandVoid::ForceDealEnemyDamage { amount } => {
                write!(f, "force_deal_enemy_damage({amount})")
            }
        }
    }
}

impl CommandVoid {
    pub fn log_display(&self) -> CommandVoidLogDisplay<'_> {
        CommandVoidLogDisplay { command: self }
    }

    pub fn contained_messages(&self) -> Box<dyn Iterator<Item = &CommandString> + '_> {
        match self {
            CommandVoid::Multi { commands } => {
                Box::new(commands.iter().flat_map(Self::contained_messages))
            }
            CommandVoid::QueuedMessage { message, .. }
            | CommandVoid::FreeMessage { message, .. } => Box::new(iter::once(message)),
            _ => Box::new(iter::empty()),
        }
    }

    // TODO: Doesn't look into function invocations yet
    pub fn contained_messages_mut(&mut self) -> Box<dyn Iterator<Item = &mut CommandString> + '_> {
        match self {
            CommandVoid::Multi { commands } => {
                Box::new(commands.iter_mut().flat_map(Self::contained_messages_mut))
            }
            CommandVoid::QueuedMessage { message, .. }
            | CommandVoid::FreeMessage { message, .. } => Box::new(iter::once(message)),
            _ => Box::new(iter::empty()),
        }
    }
}

pub struct CommandVoidLogDisplay<'s> {
    command: &'s CommandVoid,
}

impl Display for CommandVoidLogDisplay<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut messages = self
            .command
            .contained_messages()
            .filter_map(CommandString::as_constant)
            .map(String::as_str)
            .map(strip_invisible_characters);

        match messages.next() {
            None => self.command.fmt(f),
            Some(first) => {
                first.fmt(f)?;

                for message in messages {
                    write!(f, ", {message}")?;
                }

                Ok(())
            }
        }
    }
}

// TODO why not in place?
pub fn strip_invisible_characters(s: &str) -> String {
    #[derive(Logos)]
    enum Token {
        #[regex(r"<world>[^<]*<|<icon>[^<]*<|[^@#$*<]")]
        Visible,
        #[regex(r"@|#|\$|\*|<[^>]*>")]
        Invisible,
    }

    let mut result = String::new();
    let mut start = 0;

    for (token, span) in Token::lexer(s).spanned() {
        match token {
            Ok(Token::Visible) => {}
            Ok(Token::Invisible) => {
                result.push_str(&s[start..span.start]);
                start = span.end;
            }
            Err(()) => unreachable!(),
        }
    }

    result.push_str(&s[start..]);

    result
}

#[cfg(test)]
mod tests {
    #[test]
    fn strip_invisible_characters() {
        use super::strip_invisible_characters as strip;

        assert_eq!(strip(""), "");
        assert_eq!(strip("aaa"), "aaa");
        assert_eq!(strip("@#$"), "");
        assert_eq!(strip("@@@a@a@@a@"), "aaa");
        assert_eq!(strip("a<aaa>a</><aaaaa>a"), "aaa");
        assert_eq!(
            strip(r"<worldn>1<\><world>1<\><nicon>x<\><icon>x<\>"),
            r"1<world>1<\>x<icon>x<\>"
        );
    }
}

fn store_suffix(trigger_events: bool) -> &'static str {
    if trigger_events {
        ""
    } else {
        "_without_triggers"
    }
}

fn save_suffix(to_disk: bool) -> &'static str {
    if to_disk {
        ""
    } else {
        "_to_memory"
    }
}

fn multi<T: Display>(f: &mut fmt::Formatter, commands: &[CommandVoid], last: T) -> fmt::Result {
    write!(f, "{{ ")?;

    for command in commands {
        write!(f, "{command}, ")?;
    }

    write!(f, "{last} }}")
}
