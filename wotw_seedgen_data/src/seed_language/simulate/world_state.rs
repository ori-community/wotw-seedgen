use crate::seed_language::{
    output::Event,
    simulate::{
        shop_state::ShopState, CloneSnapshot, ConditionValues, Heap, Simulation, Snapshot, Stack,
        UberStates,
    },
};

#[derive(Debug, Clone)]
pub struct WorldState {
    pub uber_states: UberStates,
    pub stack: Stack,
    pub heap: Heap,
    pub condition_values: CloneSnapshot<ConditionValues>,
    pub shops: CloneSnapshot<ShopState>,
}

impl WorldState {
    #[inline]
    pub fn new(uber_states: UberStates, events: &mut [Event]) -> Self {
        let mut world_state = Self {
            uber_states,
            stack: Stack::default(),
            heap: Heap::default(),
            condition_values: CloneSnapshot::new(ConditionValues::default()),
            shops: CloneSnapshot::new(ShopState::default()),
        };

        for (index, event) in events.iter_mut().enumerate() {
            world_state.register_trigger(&mut event.trigger, index);
        }

        world_state
    }
}

impl Simulation for WorldState {
    fn world_state(&self) -> &WorldState {
        self
    }

    fn world_state_mut(&mut self) -> &mut WorldState {
        self
    }
}

impl Snapshot for WorldState {
    fn snapshot(&mut self) {
        self.uber_states.snapshot();
        self.condition_values.snapshot();
        self.shops.snapshot();
    }

    fn restore_snapshot(&mut self) {
        self.uber_states.restore_snapshot();
        self.condition_values.restore_snapshot();
        self.shops.restore_snapshot();
    }
}
