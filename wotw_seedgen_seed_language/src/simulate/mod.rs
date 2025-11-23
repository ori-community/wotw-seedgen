mod simulation;
mod uber_states;
mod variables;
mod world_state;

pub use simulation::Simulation;
pub use uber_states::UberStates;
pub use variables::Variables;
pub use world_state::WorldState;

use crate::{
    ast::ClientEvent,
    output::{
        Command, CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandVoid,
        CommandZone, Event, ExecuteOperator, Operation, StringOrPlaceholder, Trigger,
    },
};
use ordered_float::OrderedFloat;
use wotw_seedgen_assets::UberStateValue;
use wotw_seedgen_data::{UberIdentifier, Zone};

pub trait Simulate<S: Simulation> {
    type Return;

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return;
}

impl<S: Simulation, T: Simulate<S>> Simulate<S> for Vec<T> {
    type Return = ();

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        for t in self {
            t.simulate(simulation, events);
        }
    }
}

impl<S: Simulation> Simulate<S> for ClientEvent {
    type Return = ();

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        events
            .iter()
            .filter(|event| event.trigger == Trigger::ClientEvent(*self))
            .for_each(|event| {
                event.command.simulate(simulation, events);
            })
    }
}

impl<S: Simulation> Simulate<S> for Command {
    type Return = ();

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        match self {
            Command::Boolean(command) => {
                command.simulate(simulation, events);
            }
            Command::Integer(command) => {
                command.simulate(simulation, events);
            }
            Command::Float(command) => {
                command.simulate(simulation, events);
            }
            Command::String(command) => {
                command.simulate(simulation, events);
            }
            Command::Zone(command) => {
                command.simulate(simulation, events);
            }
            Command::Void(command) => {
                command.simulate(simulation, events);
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

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        let left = self.left.simulate(simulation, events);
        let right = self.right.simulate(simulation, events);

        self.operator.execute(left, right)
    }
}

impl<S: Simulation> Simulate<S> for CommandBoolean {
    type Return = bool;

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        match self {
            CommandBoolean::Constant { value } => *value,
            CommandBoolean::Multi { commands, last } => {
                commands.simulate(simulation, events);
                last.simulate(simulation, events)
            }
            CommandBoolean::CompareBoolean { operation } => operation.simulate(simulation, events),
            CommandBoolean::CompareInteger { operation } => operation.simulate(simulation, events),
            CommandBoolean::CompareFloat { operation } => operation.simulate(simulation, events),
            CommandBoolean::CompareString { operation } => operation.simulate(simulation, events),
            CommandBoolean::CompareZone { operation } => operation.simulate(simulation, events),
            CommandBoolean::LogicOperation { operation } => operation.simulate(simulation, events),
            CommandBoolean::FetchBoolean { uber_identifier } => simulation
                .uber_states()
                .fetch(*uber_identifier)
                .as_boolean(),
            CommandBoolean::GetBoolean { id } => simulation.variables().get_boolean(id),
            CommandBoolean::IsInBox { .. } => false,
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandInteger {
    type Return = i32;

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        match self {
            CommandInteger::Constant { value } => *value,
            CommandInteger::Multi { commands, last } => {
                commands.simulate(simulation, events);
                last.simulate(simulation, events)
            }
            CommandInteger::Arithmetic { operation } => operation.simulate(simulation, events),
            CommandInteger::FetchInteger { uber_identifier } => simulation
                .uber_states()
                .fetch(*uber_identifier)
                .as_integer(),
            CommandInteger::GetInteger { id } => simulation.variables().get_integer(id),
            CommandInteger::FromFloat { float } => {
                float.simulate(simulation, events).into_inner().round() as i32
            }
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandFloat {
    type Return = OrderedFloat<f32>;

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        match self {
            CommandFloat::Constant { value } => *value,
            CommandFloat::Multi { commands, last } => {
                commands.simulate(simulation, events);
                last.simulate(simulation, events)
            }
            CommandFloat::Arithmetic { operation } => operation.simulate(simulation, events),
            CommandFloat::FetchFloat { uber_identifier } => simulation
                .uber_states()
                .fetch(*uber_identifier)
                .as_float()
                .into(),
            CommandFloat::GetFloat { id } => simulation.variables().get_float(id),
            CommandFloat::FromInteger { integer } => {
                (integer.simulate(simulation, events) as f32).into()
            }
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandString {
    type Return = String;

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        match self {
            CommandString::Constant { value } => match value {
                StringOrPlaceholder::Value(value) => value.clone(),
                other => other.to_string(),
            },
            CommandString::Multi { commands, last } => {
                commands.simulate(simulation, events);
                last.simulate(simulation, events)
            }
            CommandString::Concatenate { operation } => operation.simulate(simulation, events),
            CommandString::GetString { id } => simulation.variables().get_string(id),
            CommandString::WorldName { .. } => Default::default(),
            CommandString::FromBoolean { boolean } => {
                boolean.simulate(simulation, events).to_string()
            }
            CommandString::FromInteger { integer } => {
                integer.simulate(simulation, events).to_string()
            }
            CommandString::FromFloat { float } => float.simulate(simulation, events).to_string(),
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandZone {
    type Return = Zone;

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        match self {
            CommandZone::Constant { value } => *value,
            CommandZone::Multi { commands, last } => {
                commands.simulate(simulation, events);
                last.simulate(simulation, events)
            }
            CommandZone::CurrentZone {} | CommandZone::CurrentMapZone {} => Zone::Void,
        }
    }
}

impl<S: Simulation> Simulate<S> for CommandVoid {
    type Return = ();

    fn simulate(&self, simulation: &mut S, events: &[Event]) -> Self::Return {
        match self {
            CommandVoid::Multi { commands } => commands.simulate(simulation, events),
            CommandVoid::If { condition, command } => {
                if condition.simulate(simulation, events) {
                    command.simulate(simulation, events)
                }
            }
            CommandVoid::StoreBoolean {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = UberStateValue::Boolean(value.simulate(simulation, events));
                set_uber_state(simulation, events, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::StoreInteger {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = UberStateValue::Integer(value.simulate(simulation, events));
                set_uber_state(simulation, events, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::StoreFloat {
                uber_identifier,
                value,
                trigger_events,
            } => {
                let value = UberStateValue::Float(*value.simulate(simulation, events));
                set_uber_state(simulation, events, *uber_identifier, value, *trigger_events);
            }
            CommandVoid::SetBoolean { id, value } => {
                let value = value.simulate(simulation, events);
                simulation.variables_mut().set_boolean(*id, value);
            }
            CommandVoid::SetInteger { id, value } => {
                let value = value.simulate(simulation, events);
                simulation.variables_mut().set_integer(*id, value);
            }
            CommandVoid::SetFloat { id, value } => {
                let value = value.simulate(simulation, events);
                simulation.variables_mut().set_float(*id, value);
            }
            CommandVoid::SetString { id, value } => {
                let value = value.simulate(simulation, events);
                simulation.variables_mut().set_string(*id, value);
            }
            CommandVoid::TriggerClientEvent { client_event } => {
                client_event.simulate(simulation, events)
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
            | CommandVoid::FreeMessageHorizontalAnchor { .. }
            | CommandVoid::FreeMessageVerticalAnchor { .. }
            | CommandVoid::FreeMessageBoxWidth { .. }
            | CommandVoid::FreeMessageCoordinateSystem { .. }
            | CommandVoid::SetMapMessage { .. }
            | CommandVoid::CreateWarpIcon { .. }
            | CommandVoid::DestroyWarpIcon { .. }
            | CommandVoid::Lookup { .. }
            | CommandVoid::BoxTrigger { .. }
            | CommandVoid::BoxTriggerDestroy { .. }
            | CommandVoid::BoxTriggerEnterCallback { .. }
            | CommandVoid::BoxTriggerLeaveCallback { .. }
            | CommandVoid::Save { .. }
            | CommandVoid::SaveAt { .. }
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
            | CommandVoid::ResetAllWheels { .. }
            | CommandVoid::CloseMenu { .. }
            | CommandVoid::CloseWeaponWheel { .. }
            | CommandVoid::DebugLog { .. } => {}
        }
    }
}

fn set_uber_state<S: Simulation>(
    simulation: &mut S,
    events: &[Event],
    uber_identifier: UberIdentifier,
    value: UberStateValue,
    trigger_events: bool,
) {
    // TODO virtual uberstate simulation?
    if simulation
        .uber_states()
        .prevent_change(uber_identifier, value)
    {
        return;
    }

    if trigger_events {
        let triggers = simulation
            .uber_states_mut()
            .set_and_return_triggers(uber_identifier, value)
            .collect();
        side_effects(simulation, events, uber_identifier, value);
        process_triggers(simulation, events, triggers);
    } else {
        simulation.uber_states_mut().set(uber_identifier, value);
    }

    simulation.on_change(uber_identifier, events);
}

fn process_triggers<S: Simulation>(simulation: &mut S, events: &[Event], triggers: Vec<usize>) {
    // Trigger conditions have to be evaluated ahead of time in case any
    // triggered commands modify states relevant to the conditions.
    let triggered_events = triggers
        .into_iter()
        .map(|index| &events[index])
        .filter(|event| match &event.trigger {
            Trigger::ClientEvent(_) => false,
            Trigger::Binding(_) => true,
            Trigger::Condition(condition) => condition.simulate(simulation, events),
        })
        .collect::<Vec<_>>();

    for event in triggered_events {
        event.command.simulate(simulation, events);
    }
}

fn side_effects<S: Simulation>(
    simulation: &mut S,
    events: &[Event],
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
        simulation.add_max_health(10, events);
        simulation.add_max_energy(1., events);
    }
}
