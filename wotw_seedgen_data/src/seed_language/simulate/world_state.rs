use crate::seed_language::{
    output::Event,
    simulate::{CloneSnapshot, ConditionValues, Heap, Simulation, Snapshot, Stack, UberStates},
};

#[derive(Debug, Clone)]
pub struct WorldState {
    pub uber_states: UberStates,
    pub stack: Stack,
    pub heap: Heap,
    pub condition_values: CloneSnapshot<ConditionValues>,
}

impl WorldState {
    #[inline]
    pub fn new(uber_states: UberStates, events: &mut [Event]) -> Self {
        let mut world_state = Self {
            uber_states,
            stack: Stack::default(),
            heap: Heap::default(),
            condition_values: CloneSnapshot::new(ConditionValues::default()),
        };

        for (index, event) in events.iter_mut().enumerate() {
            world_state.register_trigger(&mut event.trigger, index);
        }

        world_state
    }
}

impl Simulation for WorldState {
    fn stack(&self) -> &Stack {
        &self.stack
    }

    fn stack_mut(&mut self) -> &mut Stack {
        &mut self.stack
    }

    #[inline]
    fn heap(&self) -> &Heap {
        &self.heap
    }

    #[inline]
    fn heap_mut(&mut self) -> &mut Heap {
        &mut self.heap
    }

    #[inline]
    fn uber_states(&self) -> &UberStates {
        &self.uber_states
    }

    #[inline]
    fn uber_states_mut(&mut self) -> &mut UberStates {
        &mut self.uber_states
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
