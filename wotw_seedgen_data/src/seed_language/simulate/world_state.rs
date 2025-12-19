use crate::seed_language::simulate::{Simulation, UberStates, Variables};

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
    #[inline]
    fn uber_states(&self) -> &UberStates {
        &self.uber_states
    }

    #[inline]
    fn uber_states_mut(&mut self) -> &mut UberStates {
        &mut self.uber_states
    }

    #[inline]
    fn variables(&self) -> &Variables {
        &self.variables
    }

    #[inline]
    fn variables_mut(&mut self) -> &mut Variables {
        &mut self.variables
    }
}
