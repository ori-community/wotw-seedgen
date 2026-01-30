use crate::{
    assets::UberStateValue,
    seed_language::simulate::{Simulation, UberStates, Variables},
    UberIdentifier,
};

#[derive(Debug, Clone)]
pub struct WorldState {
    pub uber_states: UberStates,
    pub variables: Variables,
}

impl WorldState {
    #[inline]
    pub fn new(uber_states: UberStates) -> Self {
        Self {
            uber_states,
            variables: Default::default(),
        }
    }
}

impl Simulation for WorldState {
    fn fetch(&self, uber_identifier: UberIdentifier) -> UberStateValue {
        self.uber_states.fetch(uber_identifier)
    }

    fn store_impl(
        &mut self,
        uber_identifier: UberIdentifier,
        value: UberStateValue,
    ) -> impl Iterator<Item = usize> + '_ {
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

    fn snapshot(&mut self) {
        self.uber_states.snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.uber_states.restore_snapshot();
    }
}
