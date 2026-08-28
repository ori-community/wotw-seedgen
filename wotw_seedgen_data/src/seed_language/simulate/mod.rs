mod cache;
mod condition_values;
mod heap;
mod simulation;
mod snapshot;
mod stack;
mod uber_states;
mod world_state;

use std::mem;

pub use cache::SimulationCache;
pub use condition_values::ConditionValues;
pub use heap::Heap;
pub use simulation::Simulation;
pub use snapshot::{CloneSnapshot, Snapshot};
pub use stack::Stack;
pub use uber_states::{UberStates, UBER_STATES_TARGET_PREFIX};
pub use world_state::WorldState;

use crate::{
    assets::UberStateValue,
    seed_language::{
        ast::ClientEvent,
        output::{
            Command, CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandVoid,
            CommandZone, CommandsOutput, ExecuteOperator, Operation, StringOrPlaceholder, Trigger,
            TriggerCondition,
        },
    },
    UberIdentifier, Zone,
};

pub trait Simulate<S: Simulation> {
    type Return;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return;
}

impl<S: Simulation, T: Simulate<S>> Simulate<S> for Vec<T> {
    type Return = ();

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        for t in self {
            t.simulate(simulation, output);
        }
    }
}

impl<S: Simulation> Simulate<S> for ClientEvent {
    type Return = ();

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        output
            .events()
            .iter()
            .filter(|event| event.trigger == Trigger::ClientEvent(*self))
            .for_each(|event| {
                event.command.simulate(simulation, output);
            });
    }
}

impl<S: Simulation> Simulate<S> for TriggerCondition {
    type Return = bool;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        let value = self.condition.simulate(simulation, output);
        let previous_value =
            mem::replace(simulation.condition_values().get(self.id.unwrap()), value);
        value && !previous_value
    }
}

impl<S: Simulation> Simulate<S> for Command {
    type Return = ();

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        match self {
            Command::Boolean(command) => {
                command.simulate(simulation, output);
            }
            Command::Integer(command) => {
                command.simulate(simulation, output);
            }
            Command::Float(command) => {
                command.simulate(simulation, output);
            }
            Command::String(command) => {
                command.simulate(simulation, output);
            }
            Command::Zone(command) => {
                command.simulate(simulation, output);
            }
            Command::Void(command) => {
                command.simulate(simulation, output);
            }
        }
    }
}

impl<S, Item, Operator> Simulate<S> for Operation<Item, Operator>
where
    S: Simulation,
    Item: Simulate<S>,
    Operator: Copy + ExecuteOperator<Item::Return>,
{
    type Return = Operator::Output;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        let left = self.left.simulate(simulation, output);
        let right = self.right.simulate(simulation, output);

        self.operator.execute(left, right)
    }
}

impl<S: Simulation> Simulate<S> for CommandBoolean {
    type Return = bool;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        match self {
            CommandBoolean::Constant { value } => *value,
            CommandBoolean::FunctionArgument { index } => simulation.stack().get_boolean(*index),
            CommandBoolean::Multi { commands, last } => {
                commands.simulate(simulation, output);
                last.simulate(simulation, output)
            }
            CommandBoolean::CompareBoolean { operation } => operation.simulate(simulation, output),
            CommandBoolean::CompareInteger { operation } => operation.simulate(simulation, output),
            CommandBoolean::CompareFloat { operation } => operation.simulate(simulation, output),
            CommandBoolean::CompareString { operation } => operation.simulate(simulation, output),
            CommandBoolean::CompareZone { operation } => operation.simulate(simulation, output),
            CommandBoolean::LogicOperation { operation } => operation.simulate(simulation, output),
            CommandBoolean::FetchBoolean { uber_identifier } => {
                simulation.fetch(*uber_identifier).as_boolean()
            }
            CommandBoolean::GetBoolean { id } => simulation.heap().get_boolean(*id),
            CommandBoolean::IsInCircle { .. }
            | CommandBoolean::IsInPositionTrigger { .. }
            | CommandBoolean::IsInRectangle { .. } => false,
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandInteger {
    type Return = i32;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        match self {
            CommandInteger::Constant { value } => *value,
            CommandInteger::FunctionArgument { index } => simulation.stack().get_integer(*index),
            CommandInteger::Multi { commands, last } => {
                commands.simulate(simulation, output);
                last.simulate(simulation, output)
            }
            CommandInteger::Arithmetic { operation } => operation.simulate(simulation, output),
            CommandInteger::FetchInteger { uber_identifier } => {
                simulation.fetch(*uber_identifier).as_integer()
            }
            CommandInteger::GetInteger { id } => simulation.heap().get_integer(*id),
            CommandInteger::FromFloat { float } => {
                float.simulate(simulation, output).round() as i32
            }
            CommandInteger::StringLength { string } => {
                string.simulate(simulation, output).len() as i32
            }
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandFloat {
    type Return = f32;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        match self {
            CommandFloat::Constant { value } => **value,
            CommandFloat::FunctionArgument { index } => simulation.stack().get_float(*index),
            CommandFloat::Multi { commands, last } => {
                commands.simulate(simulation, output);
                last.simulate(simulation, output)
            }
            CommandFloat::Arithmetic { operation } => operation.simulate(simulation, output),
            CommandFloat::FetchFloat { uber_identifier } => {
                simulation.fetch(*uber_identifier).as_float()
            }
            CommandFloat::GetFloat { id } => simulation.heap().get_float(*id),
            CommandFloat::FromInteger { integer } => integer.simulate(simulation, output) as f32,
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandString {
    type Return = String;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        match self {
            CommandString::Constant { value } => match value {
                StringOrPlaceholder::Value(value) => value.clone(),
                other => other.to_string(),
            },
            CommandString::FunctionArgument { index } => simulation.stack().get_string(*index),
            CommandString::Multi { commands, last } => {
                commands.simulate(simulation, output);
                last.simulate(simulation, output)
            }
            CommandString::Concatenate { operation } => operation.simulate(simulation, output),
            CommandString::GetString { id } => simulation.heap().get_string(*id),
            CommandString::WorldName { .. } => String::new(),
            CommandString::FromBoolean { boolean } => {
                boolean.simulate(simulation, output).to_string()
            }
            CommandString::FromInteger { integer } => {
                integer.simulate(simulation, output).to_string()
            }
            CommandString::FromFloat { float } => float.simulate(simulation, output).to_string(),
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandZone {
    type Return = Zone;

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        match self {
            CommandZone::Constant { value } => *value,
            CommandZone::Multi { commands, last } => {
                commands.simulate(simulation, output);
                last.simulate(simulation, output)
            }
            CommandZone::CurrentZone {} | CommandZone::CurrentMapZone {} => Zone::Void,
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandVoid {
    type Return = ();

    fn simulate(&self, simulation: &mut S, output: &CommandsOutput) -> Self::Return {
        match self {
            CommandVoid::Multi { commands } => commands.simulate(simulation, output),
            CommandVoid::CallFunction {
                booleans,
                integers,
                floats,
                strings,
                index,
            } => {
                let booleans = booleans
                    .iter()
                    .map(|boolean| boolean.simulate(simulation, output))
                    .collect::<Vec<_>>();
                let integers = integers
                    .iter()
                    .map(|integer| integer.simulate(simulation, output))
                    .collect::<Vec<_>>();
                let floats = floats
                    .iter()
                    .map(|float| float.simulate(simulation, output))
                    .collect::<Vec<_>>();
                let strings = strings
                    .iter()
                    .map(|string| string.simulate(simulation, output))
                    .collect::<Vec<_>>();

                let stack = simulation.stack_mut();
                stack.push();

                for boolean in booleans {
                    stack.push_boolean(boolean);
                }
                for integer in integers {
                    stack.push_integer(integer);
                }
                for float in floats {
                    stack.push_float(float);
                }
                for string in strings {
                    stack.push_string(string);
                }

                output.lookup[*index].simulate(simulation, output);

                simulation.stack_mut().pop();
            }
            CommandVoid::If { condition, command } => {
                if condition.simulate(simulation, output) {
                    command.simulate(simulation, output);
                }
            }
            CommandVoid::StoreBoolean {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = value.simulate(simulation, output).into();
                set_uber_state(simulation, output, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = value.simulate(simulation, output).into();
                set_uber_state(simulation, output, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = value.simulate(simulation, output).into();
                set_uber_state(simulation, output, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::SetBoolean { id, value } => {
                let value = value.simulate(simulation, output);
                simulation.heap_mut().set_boolean(*id, value);
            }
            CommandVoid::SetInteger { id, value } => {
                let value = value.simulate(simulation, output);
                simulation.heap_mut().set_integer(*id, value);
            }
            CommandVoid::SetFloat { id, value } => {
                let value = value.simulate(simulation, output);
                simulation.heap_mut().set_float(*id, value);
            }
            CommandVoid::SetString { id, value } => {
                let value = value.simulate(simulation, output);
                simulation.heap_mut().set_string(*id, value);
            }
            CommandVoid::TriggerClientEvent { client_event } => {
                client_event.simulate(simulation, output);
            }
            // TODO simulate more maybe?
            CommandVoid::DefineTimer { .. }
            | CommandVoid::QueuedMessage { .. }
            | CommandVoid::QueuedMessageScopedPickupPosition { .. }
            | CommandVoid::FreeMessage { .. }
            | CommandVoid::FreeMessageUninitialized { .. }
            | CommandVoid::MessageDestroy { .. }
            | CommandVoid::MessageText { .. }
            | CommandVoid::MessageTimeout { .. }
            | CommandVoid::MessageBackground { .. }
            | CommandVoid::FreeMessagePosition { .. }
            | CommandVoid::FreeMessageAlignment { .. }
            | CommandVoid::FreeMessageHorizontalAnchor { .. }
            | CommandVoid::FreeMessageVerticalAnchor { .. }
            | CommandVoid::FreeMessageBoxWidth { .. }
            | CommandVoid::FreeMessageCoordinateSystem { .. }
            | CommandVoid::FreeMessageShow { .. }
            | CommandVoid::FreeMessageHide { .. }
            | CommandVoid::CreateWarpIcon { .. }
            | CommandVoid::DestroyWarpIcon { .. }
            | CommandVoid::PositionTriggerCircle { .. }
            | CommandVoid::PositionTriggerRectangle { .. }
            | CommandVoid::PositionTriggerDestroy { .. }
            | CommandVoid::PositionTriggerEnterCallback { .. }
            | CommandVoid::PositionTriggerLeaveCallback { .. }
            | CommandVoid::Save { .. }
            | CommandVoid::SaveAt { .. }
            | CommandVoid::Warp { .. }
            | CommandVoid::InstantWarp { .. }
            | CommandVoid::Equip { .. }
            | CommandVoid::Unequip { .. }
            | CommandVoid::TriggerKeybind { .. }
            | CommandVoid::EnableServerSync { .. }
            | CommandVoid::DisableServerSync { .. }
            | CommandVoid::CreateSpoilerMapIcon { .. }
            | CommandVoid::MarkSpoilerMapIconCollected { .. }
            | CommandVoid::CreateStatsEntry { .. }
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
            | CommandVoid::ResetAllWheels { .. }
            | CommandVoid::SetTrialHint { .. }
            | CommandVoid::CloseMenu { .. }
            | CommandVoid::CloseWeaponWheel { .. }
            | CommandVoid::DebugLog { .. }
            | CommandVoid::DealEnemyDamage { .. }
            | CommandVoid::ForceDealEnemyDamage { .. } => {}
        }
    }
}

// TODO is this better than using functions specialized for the UberState types?
fn set_uber_state<S: Simulation>(
    simulation: &mut S,
    output: &CommandsOutput,
    uber_identifier: UberIdentifier,
    value: UberStateValue,
    trigger_events: bool,
) {
    if simulation.fetch(uber_identifier) == value {
        return;
    }

    // TODO virtual uberstate simulation?
    if simulation.should_prevent_store(uber_identifier, value) {
        return;
    }

    if trigger_events {
        let triggers = simulation.store_impl(uber_identifier, value).to_vec();
        side_effects(simulation, output, uber_identifier, value);
        process_triggers(simulation, output, triggers);
    } else {
        let _ = simulation.store_impl(uber_identifier, value);
    }

    simulation.on_change(uber_identifier, output);
}

fn process_triggers<S: Simulation>(
    simulation: &mut S,
    output: &CommandsOutput,
    triggers: Vec<usize>,
) {
    // Trigger conditions have to be evaluated ahead of time in case any
    // triggered commands modify states relevant to the conditions.
    let triggered_events = triggers
        .into_iter()
        .map(|index| &output.events()[index])
        .filter(|event| match &event.trigger {
            Trigger::ClientEvent(_) => false,
            Trigger::Binding(_) => true,
            Trigger::Condition(condition) => condition.simulate(simulation, output),
        })
        .collect::<Vec<_>>();

    for event in triggered_events {
        event.command.simulate(simulation, output);
    }
}

fn side_effects<S: Simulation>(
    simulation: &mut S,
    output: &CommandsOutput,
    uber_identifier: UberIdentifier,
    value: UberStateValue,
) {
    const VOICE: UberIdentifier = UberIdentifier::new(46462, 59806);
    const STRENGTH: UberIdentifier = UberIdentifier::new(945, 49747);
    const MEMORY: UberIdentifier = UberIdentifier::new(28895, 25522);
    const EYES: UberIdentifier = UberIdentifier::new(18793, 63291);
    const HEART: UberIdentifier = UberIdentifier::new(10289, 22102);

    if matches!(uber_identifier, VOICE | STRENGTH | MEMORY | EYES | HEART) && value == true {
        // TODO not strictly correct but not sure what else to do
        simulation.add_base_max_health(10, output);
        simulation.add_base_max_energy(1., output);
    }
}
