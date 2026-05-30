use crate::{
    assets::UberStateValue,
    seed_language::{
        output::Event,
        simulate::{
            condition_values::ConditionValues, CloneSnapshot, Simulation, Snapshot, UberStates,
            Variables,
        },
    },
    UberIdentifier,
};

#[derive(Debug, Clone)]
pub struct WorldState {
    pub uber_states: UberStates,
    pub variables: Variables,
    pub condition_values: CloneSnapshot<ConditionValues>,
}

impl WorldState {
    #[inline]
    pub fn new(uber_states: UberStates, events: &mut [Event]) -> Self {
        let mut world_state = Self {
            uber_states,
            variables: Default::default(),
            condition_values: CloneSnapshot::new(ConditionValues::default()),
        };

        for event in events {
            world_state.register_trigger(&mut event.trigger);
        }

        world_state
    }
}

impl Simulation for WorldState {
    fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        self.uber_states.fetch(uber_identifier)
    }

    fn store_impl(&mut self, uber_identifier: UberIdentifier, value: UberStateValue) -> &[usize] {
        self.uber_states.store(uber_identifier, value)
    }

    #[inline]
    fn variables(&self) -> &Variables {
        &self.variables
    }

    #[inline]
    fn variables_mut(&mut self) -> &mut Variables {
        &mut self.variables
    }

    #[inline]
    fn condition_values(&mut self) -> &mut ConditionValues {
        &mut self.condition_values
    }
}

impl Snapshot for WorldState {
    fn snapshot(&mut self) {
        self.uber_states.snapshot();
        self.condition_values.snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.uber_states.restore_snapshot();
        self.condition_values.restore_snapshot();
    }
}
