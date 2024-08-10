use crate::common_item::CommonItem;

use super::{uber_states::UberStateValue, World};
use ordered_float::OrderedFloat;
use std::ops::{Add, Div, Mul, Sub};
use wotw_seedgen_data::{UberIdentifier, Zone};
use wotw_seedgen_seed_language::output::{
    ArithmeticOperator, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
    CommandVoid, CommandZone, Comparator, EqualityComparator, IntermediateOutput, LogicOperator,
    Operation, StringOrPlaceholder, Trigger,
};

pub trait Simulate {
    type Return;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return;
}
impl<T: Simulate> Simulate for Vec<T> {
    type Return = ();

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        for t in self {
            t.simulate(world, output);
        }
    }
}
impl Simulate for Command {
    type Return = ();

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        match self {
            Command::Boolean(command) => {
                command.simulate(world, output);
            }
            Command::Integer(command) => {
                command.simulate(world, output);
            }
            Command::Float(command) => {
                command.simulate(world, output);
            }
            Command::String(command) => {
                command.simulate(world, output);
            }
            Command::Zone(command) => {
                command.simulate(world, output);
            }
            Command::Void(command) => {
                command.simulate(world, output);
            }
        }
    }
}
impl<Item: Simulate> Simulate for Operation<Item, EqualityComparator>
where
    Item::Return: PartialEq,
{
    type Return = bool;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        let left = self.left.simulate(world, output);
        let right = self.right.simulate(world, output);
        match self.operator {
            EqualityComparator::Equal => left == right,
            EqualityComparator::NotEqual => left != right,
        }
    }
}
impl<Item: Simulate> Simulate for Operation<Item, Comparator>
where
    Item::Return: PartialEq + PartialOrd,
{
    type Return = bool;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        let left = self.left.simulate(world, output);
        let right = self.right.simulate(world, output);
        match self.operator {
            Comparator::Equal => left == right,
            Comparator::NotEqual => left != right,
            Comparator::Less => left < right,
            Comparator::LessOrEqual => left <= right,
            Comparator::Greater => left > right,
            Comparator::GreaterOrEqual => left >= right,
        }
    }
}
impl<Item: Simulate<Return = bool>> Simulate for Operation<Item, LogicOperator> {
    type Return = bool;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        let left = self.left.simulate(world, output);
        let right = self.right.simulate(world, output);
        match self.operator {
            LogicOperator::And => left && right,
            LogicOperator::Or => left || right,
        }
    }
}
impl<Item: Simulate> Simulate for Operation<Item, ArithmeticOperator>
where
    Item::Return: Add<Output = Item::Return>
        + Sub<Output = Item::Return>
        + Mul<Output = Item::Return>
        + Div<Output = Item::Return>,
{
    type Return = Item::Return;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        let left = self.left.simulate(world, output);
        let right = self.right.simulate(world, output);
        match self.operator {
            ArithmeticOperator::Add => left + right,
            ArithmeticOperator::Subtract => left - right,
            ArithmeticOperator::Multiply => left * right,
            ArithmeticOperator::Divide => left / right,
        }
    }
}
impl Simulate for CommandBoolean {
    type Return = bool;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        match self {
            CommandBoolean::Constant { value } => *value,
            CommandBoolean::Multi { commands, last } => {
                commands.simulate(world, output);
                last.simulate(world, output)
            }
            CommandBoolean::CompareBoolean { operation } => operation.simulate(world, output),
            CommandBoolean::CompareInteger { operation } => operation.simulate(world, output),
            CommandBoolean::CompareFloat { operation } => operation.simulate(world, output),
            CommandBoolean::CompareString { operation } => operation.simulate(world, output),
            CommandBoolean::CompareZone { operation } => operation.simulate(world, output),
            CommandBoolean::LogicOperation { operation } => operation.simulate(world, output),
            CommandBoolean::FetchBoolean { uber_identifier } => {
                world.uber_states.get(*uber_identifier).as_boolean()
            }
            CommandBoolean::GetBoolean { id } => world.variables.get_boolean(id),
            CommandBoolean::IsInHitbox { .. } => false,
        }
    }
}
impl Simulate for CommandInteger {
    type Return = i32;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        match self {
            CommandInteger::Constant { value } => *value,
            CommandInteger::Multi { commands, last } => {
                commands.simulate(world, output);
                last.simulate(world, output)
            }
            CommandInteger::Arithmetic { operation } => operation.simulate(world, output),
            CommandInteger::FetchInteger { uber_identifier } => {
                world.uber_states.get(*uber_identifier).as_integer()
            }
            CommandInteger::GetInteger { id } => world.variables.get_integer(id),
            CommandInteger::FromFloat { float } => {
                float.simulate(world, output).into_inner().round() as i32
            }
        }
    }
}
impl Simulate for CommandFloat {
    type Return = OrderedFloat<f32>;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        match self {
            CommandFloat::Constant { value } => *value,
            CommandFloat::Multi { commands, last } => {
                commands.simulate(world, output);
                last.simulate(world, output)
            }
            CommandFloat::Arithmetic { operation } => operation.simulate(world, output),
            CommandFloat::FetchFloat { uber_identifier } => {
                world.uber_states.get(*uber_identifier).as_float()
            }
            CommandFloat::GetFloat { id } => world.variables.get_float(id),
            CommandFloat::FromInteger { integer } => {
                (integer.simulate(world, output) as f32).into()
            }
        }
    }
}
impl Simulate for CommandString {
    type Return = String;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        match self {
            CommandString::Constant { value } => match value {
                StringOrPlaceholder::Value(value) => value.clone(),
                _ => Default::default(),
            },
            CommandString::Multi { commands, last } => {
                commands.simulate(world, output);
                last.simulate(world, output)
            }
            CommandString::Concatenate { left, right } => {
                left.simulate(world, output) + &right.simulate(world, output)
            }
            CommandString::GetString { id } => world.variables.get_string(id),
            CommandString::WorldName { .. } => Default::default(),
            CommandString::FromBoolean { boolean } => boolean.simulate(world, output).to_string(),
            CommandString::FromInteger { integer } => integer.simulate(world, output).to_string(),
            CommandString::FromFloat { float } => float.simulate(world, output).to_string(),
        }
    }
}
impl Simulate for CommandZone {
    type Return = Zone;

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        match self {
            CommandZone::Constant { value } => *value,
            CommandZone::Multi { commands, last } => {
                commands.simulate(world, output);
                last.simulate(world, output)
            }
            CommandZone::CurrentZone {} => Zone::Void,
        }
    }
}
impl Simulate for CommandVoid {
    type Return = ();

    fn simulate(&self, world: &mut World, output: &IntermediateOutput) -> Self::Return {
        if !matches!(self, CommandVoid::Multi { .. }) {
            for common_item in CommonItem::from_command(self) {
                common_item.grant(&mut world.player.inventory);
            }
        }

        match self {
            CommandVoid::Multi { commands } => commands.simulate(world, output),
            CommandVoid::If { condition, command } => {
                if condition.simulate(world, output) {
                    command.simulate(world, output)
                }
            }
            CommandVoid::StoreBoolean {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = UberStateValue::Boolean(value.simulate(world, output));
                set_uber_state(world, output, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = UberStateValue::Integer(value.simulate(world, output));
                set_uber_state(world, output, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = UberStateValue::Float(value.simulate(world, output));
                set_uber_state(world, output, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::SetBoolean { id, value } => {
                let value = value.simulate(world, output);
                world.variables.set_boolean(*id, value);
            }
            CommandVoid::SetInteger { id, value } => {
                let value = value.simulate(world, output);
                world.variables.set_integer(*id, value);
            }
            CommandVoid::SetFloat { id, value } => {
                let value = value.simulate(world, output);
                world.variables.set_float(*id, value);
            }
            CommandVoid::SetString { id, value } => {
                let value = value.simulate(world, output);
                world.variables.set_string(*id, value);
            }
            // TODO simulate more maybe?
            CommandVoid::DefineTimer { .. }
            | CommandVoid::QueuedMessage { .. }
            | CommandVoid::FreeMessage { .. }
            | CommandVoid::MessageDestroy { .. }
            | CommandVoid::MessageText { .. }
            | CommandVoid::MessageTimeout { .. }
            | CommandVoid::MessageBackground { .. }
            | CommandVoid::FreeMessagePosition { .. }
            | CommandVoid::FreeMessageAlignment { .. }
            | CommandVoid::FreeMessageScreenPosition { .. }
            | CommandVoid::CreateWarpIcon { .. }
            | CommandVoid::DestroyWarpIcon { .. }
            | CommandVoid::Lookup { .. }
            | CommandVoid::Save { .. }
            | CommandVoid::SaveToMemory { .. }
            | CommandVoid::Warp { .. }
            | CommandVoid::Equip { .. }
            | CommandVoid::Unequip { .. }
            | CommandVoid::TriggerKeybind { .. }
            | CommandVoid::EnableServerSync { .. }
            | CommandVoid::DisableServerSync { .. }
            | CommandVoid::SetSpoilerMapIcon { .. }
            | CommandVoid::SetWarpIconLabel { .. }
            | CommandVoid::SetShopItemPrice { .. }
            | CommandVoid::SetShopItemName { .. }
            | CommandVoid::SetShopItemDescription { .. }
            | CommandVoid::SetShopItemIcon { .. }
            | CommandVoid::SetShopItemHidden { .. }
            | CommandVoid::SetShopItemLocked { .. }
            | CommandVoid::SetWheelItemName { .. }
            | CommandVoid::SetWheelItemDescription { .. }
            | CommandVoid::SetWheelItemIcon { .. }
            | CommandVoid::SetWheelItemColor { .. }
            | CommandVoid::SetWheelItemAction { .. }
            | CommandVoid::DestroyWheelItem { .. }
            | CommandVoid::SwitchWheel { .. }
            | CommandVoid::SetWheelPinned { .. }
            | CommandVoid::ClearAllWheels { .. } => {}
        }
    }
}

fn set_uber_state(
    world: &mut World,
    output: &IntermediateOutput,
    uber_identifier: UberIdentifier,
    value: UberStateValue,
    trigger_events: bool,
) {
    // TODO virtual uberstate simulation?
    if prevent_uber_state_change(world, uber_identifier, value) {
        return;
    }
    if trigger_events {
        let events = world
            .uber_states
            .set_and_return_triggers(uber_identifier, value)
            .collect();
        uber_state_side_effects(world, output, uber_identifier, value, trigger_events);
        process_triggers(world, output, events);
    } else {
        world.uber_states.set(uber_identifier, value);
    }
}
fn process_triggers(world: &mut World, output: &IntermediateOutput, events: Vec<usize>) {
    for index in events {
        let event = &output.events[index];
        if match &event.trigger {
            Trigger::ClientEvent(_) => false,
            Trigger::Binding(_) => true,
            Trigger::Condition(condition) => condition.simulate(world, output),
        } {
            event.command.simulate(world, output);
        }
    }
}

const WELLSPRING_QUEST: UberIdentifier = UberIdentifier::new(937, 34641);
const KU_QUEST: UberIdentifier = UberIdentifier::new(14019, 34504);
const POOLS_FIGHT_ARENA_2: UberIdentifier = UberIdentifier::new(5377, 53480);
const POOLS_FIGHT_ARENA_1: UberIdentifier = UberIdentifier::new(5377, 1373);
const DIAMOND_IN_THE_ROUGH_CUTSCENE: UberIdentifier = UberIdentifier::new(42178, 2654);
const DIAMOND_IN_THE_ROUGH_PICKUP: UberIdentifier = UberIdentifier::new(23987, 14832);
const WELLSPRING_ESCAPE_COMPLETE: UberIdentifier = UberIdentifier::new(37858, 12379);
const TULEY_IN_GLADES: UberIdentifier = UberIdentifier::new(6, 300);
const CAT_AND_MOUSE: UberIdentifier = UberIdentifier::new(58674, 32810);
const WILLOW_STONE_BOSS_HEART: UberIdentifier = UberIdentifier::new(16155, 28478);
const WILLOW_STONE_BOSS_STATE: UberIdentifier = UberIdentifier::new(16155, 12971);
const SWORD_TREE: UberIdentifier = UberIdentifier::new(0, 100);
const RAIN_LIFTED: UberIdentifier = UberIdentifier::new(6, 401);
const VOICE: UberIdentifier = UberIdentifier::new(46462, 59806);
const STRENGTH: UberIdentifier = UberIdentifier::new(945, 49747);
const MEMORY: UberIdentifier = UberIdentifier::new(28895, 25522);
const EYES: UberIdentifier = UberIdentifier::new(18793, 63291);
const HEART: UberIdentifier = UberIdentifier::new(10289, 22102);

fn prevent_uber_state_change(
    world: &World,
    uber_identifier: UberIdentifier,
    value: UberStateValue,
) -> bool {
    match uber_identifier {
        WELLSPRING_QUEST if world.uber_states.get(WELLSPRING_QUEST) >= value.as_integer() => true,
        KU_QUEST if value <= 4 => true,
        _ => false,
    }
}

// This should mirror https://github.com/ori-community/wotw-rando-client/blob/dev/projects/Randomizer/uber_states/misc_handlers.cpp
// TODO isn't most of this in seed core now?
fn uber_state_side_effects(
    world: &mut World,
    output: &IntermediateOutput,
    uber_identifier: UberIdentifier,
    value: UberStateValue,
    trigger_events: bool,
) {
    match uber_identifier {
        POOLS_FIGHT_ARENA_2 if value == 4 => {
            set_uber_state(
                world,
                output,
                POOLS_FIGHT_ARENA_1,
                UberStateValue::Integer(4),
                trigger_events,
            );
        }
        DIAMOND_IN_THE_ROUGH_CUTSCENE if matches!(value.as_integer(), 1 | 2) => {
            set_uber_state(
                world,
                output,
                DIAMOND_IN_THE_ROUGH_CUTSCENE,
                UberStateValue::Integer(3),
                trigger_events,
            );
            set_uber_state(
                world,
                output,
                DIAMOND_IN_THE_ROUGH_PICKUP,
                UberStateValue::Boolean(true),
                trigger_events,
            );
        }
        WELLSPRING_ESCAPE_COMPLETE if value == true => {
            set_uber_state(
                world,
                output,
                WELLSPRING_QUEST,
                UberStateValue::Integer(3),
                trigger_events,
            );
        }
        WELLSPRING_QUEST if value >= 3 => {
            set_uber_state(
                world,
                output,
                TULEY_IN_GLADES,
                UberStateValue::Boolean(true),
                trigger_events,
            );
        }
        CAT_AND_MOUSE if value == 7 => {
            set_uber_state(
                world,
                output,
                CAT_AND_MOUSE,
                UberStateValue::Integer(8),
                trigger_events,
            );
        }
        WILLOW_STONE_BOSS_HEART if value == true => {
            set_uber_state(
                world,
                output,
                WILLOW_STONE_BOSS_STATE,
                UberStateValue::Integer(4),
                trigger_events,
            );
        }
        SWORD_TREE if value == true => {
            set_uber_state(
                world,
                output,
                RAIN_LIFTED,
                UberStateValue::Boolean(true),
                trigger_events,
            );
        }
        VOICE | STRENGTH | MEMORY | EYES | HEART if value == true => {
            // TODO not strictly correct but not sure what else to do
            world.modify_max_health(10, output);
            world.modify_max_energy(1.0.into(), output);
        }
        _ => {}
    }
}
