use std::iter;

use arrayvec::ArrayVec;
use itertools::Itertools;
use log::{trace, warn};
use rand::{
    seq::{IteratorRandom, SliceRandom},
    SeedableRng,
};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashSet;
use wotw_seedgen_data::{
    assets::LocDataEntry,
    logic_language::output::{Graph, Node},
    seed_language::{
        ast::ClientEvent,
        compile::store_boolean,
        output::{CommandBoolean, CommandVoid, Event, IntermediateOutput, Trigger},
        simulate::{Simulate, Simulation, Snapshot},
    },
    Difficulty, Spawn, UberIdentifier, DEFAULT_SPAWN,
};
use wotw_seedgen_log_capture::LogCapture;

use crate::{
    generator::{format_pickups, SEED_FAILED_MESSAGE},
    item_pool::ItemPool,
    LogicalDifficulty, World,
};

pub fn choose_spawn<'graph, 'log>(
    rng: &mut Pcg64Mcg,
    world: &mut World<'graph, '_, '_, 'log>,
    log_index: &str,
    item_pool: &ItemPool<'log>,
    output: &mut IntermediateOutput<'log>,
) -> Result<SpawnOutput<'graph>, String> {
    let mut context = SpawnContext::new(rng, world, log_index, item_pool, output);
    context.choose_spawn()?;
    context.postprocess();
    Ok(context.finish())
}

struct SpawnContext<'world, 'graph, 'settings, 'perf, 'index, 'pool, 'output, 'log> {
    rng: Pcg64Mcg,
    world: &'world mut World<'graph, 'settings, 'perf, 'log>,
    log_index: &'index str,
    item_pool: &'pool ItemPool<'log>,
    output: &'output mut IntermediateOutput<'log>,
    default_spawn: usize,
    spawn_output: SpawnOutput<'graph>,
}

impl<'world, 'graph, 'settings, 'perf, 'index, 'pool, 'output, 'log>
    SpawnContext<'world, 'graph, 'settings, 'perf, 'index, 'pool, 'output, 'log>
{
    fn new(
        rng: &mut Pcg64Mcg,
        world: &'world mut World<'graph, 'settings, 'perf, 'log>,
        log_index: &'index str,
        item_pool: &'pool ItemPool<'log>,
        output: &'output mut IntermediateOutput<'log>,
    ) -> Self {
        let rng = Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE);

        let default_spawn = world.graph.find_node(DEFAULT_SPAWN).unwrap();

        let mut context = Self {
            rng,
            world,
            log_index,
            item_pool,
            output,
            default_spawn,
            spawn_output: SpawnOutput::default(),
        };

        context.world.snapshot();

        context.world.spawn = default_spawn;
        context.total_reach_check();

        for (index, node) in context.world.graph.nodes.iter().enumerate() {
            let Node::Pickup(pickup) = node else { continue };

            if context.world.has_reached(index) {
                context.spawn_output.total_reach.push(pickup);
            } else {
                context.spawn_output.unreachable.push(pickup);
            }
        }

        context.world.restore_snapshot();

        context
    }

    fn choose_spawn(&mut self) -> Result<(), String> {
        match &self.world.settings.spawn {
            Spawn::Set(identifier) => {
                let (index, node) = self
                    .world
                    .graph
                    .nodes
                    .iter()
                    .enumerate()
                    .find(|(_, node)| node.identifier() == identifier)
                    .ok_or_else(|| format!("Spawn {identifier} not found"))?;

                if !node.can_spawn() {
                    return Err(format!("{identifier} is not a valid spawn"));
                }

                self.world.spawn = index;

                Ok(())
            }
            Spawn::Random => {
                let spawns = RandomSpawnGenerator::new(
                    &mut self.rng,
                    self.world.graph,
                    self.world.settings.difficulty,
                    self.item_pool.log_capture,
                );

                self.choose_random_spawn(spawns)
            }
            Spawn::FullyRandom => {
                let spawns = FullyRandomSpawnGenerator::new(&mut self.rng, self.world.graph)?;

                self.choose_random_spawn(spawns)
            }
        }
    }

    fn choose_random_spawn<I>(&mut self, spawns: I) -> Result<(), String>
    where
        I: Iterator<Item = usize>,
    {
        for spawn in spawns {
            self.world.snapshot();

            self.world.spawn = spawn;
            self.total_reach_check();

            let (default_spawn_reached, reached_count) = self.world.reached_indices().fold(
                (false, 0),
                |(default_spawn_reached, reached_count), index| {
                    (
                        default_spawn_reached || index == self.default_spawn,
                        reached_count + usize::from(self.world.graph.nodes[index].is_pickup()),
                    )
                },
            );

            self.world.restore_snapshot();

            if !default_spawn_reached {
                trace!(
                    logger: self.item_pool.log_capture,
                    "{log_index}Discarding spawn {spawn} since {default_spawn} wasn't reached",
                    log_index = self.log_index,
                    spawn = self.world.graph.nodes[spawn].identifier(),
                    default_spawn = DEFAULT_SPAWN,
                );
            } else if reached_count != self.spawn_output.total_reach.len() {
                trace!(
                    logger: self.item_pool.log_capture,
                    "{log_index}Discarding spawn {spawn} since only {reached_count}/{total_count} locations were reached",
                    log_index = self.log_index,
                    spawn = self.world.graph.nodes[spawn].identifier(),
                    total_count = self.spawn_output.total_reach.len(),
                );
            } else {
                trace!(
                    logger: self.item_pool.log_capture,
                    "{log_index}Spawning on {spawn}",
                    log_index = self.log_index,
                    spawn = self.world.graph.nodes[spawn].identifier(),
                );

                return Ok(());
            }
        }

        Err("All available spawn locations failed to reach all locations".to_string())
    }

    fn total_reach_check(&mut self) {
        // TODO could avoid redoing this with a snapshot stack
        for command in &**self.item_pool {
            command.simulate(self.world, &self.output.commands);
        }
        self.world.add_spirit_light(
            self.output.modifiers.total_spirit_light(),
            &self.output.commands,
        );

        self.world.traverse_spawn(&self.output.commands);
    }

    fn postprocess(&mut self) {
        filter_placement_locations(
            &self.world,
            &self.log_index,
            &mut self.spawn_output.total_reach,
            &self.output,
            self.item_pool.log_capture,
        );

        trace!(
            logger: self.item_pool.log_capture,
            "{log_index}{amount} reachable placement location{s}: {total_reach}",
            log_index = self.log_index,
            amount = self.spawn_output.total_reach.len(),
            s = if self.spawn_output.total_reach.len() == 1 { "" } else { "s" },
            total_reach = format_pickups(&self.spawn_output.total_reach)
        );

        self.spawn_output.total_reach.shuffle(&mut self.rng);

        let unexpected_unreachable = self.spawn_output.unreachable.len()
            != self.world.settings.difficulty.expected_unreachable();

        filter_placement_locations(
            &self.world,
            &self.log_index,
            &mut self.spawn_output.unreachable,
            &self.output,
            self.item_pool.log_capture,
        );

        trace!(
            logger: self.item_pool.log_capture,
            "{log_index}{amount} unreachable placement location{s}: {unreachable}",
            log_index = self.log_index,
            amount = self.spawn_output.unreachable.len(),
            s = if self.spawn_output.unreachable.len() == 1 { "" } else { "s" },
            unreachable = format_pickups(&self.spawn_output.unreachable)
        );

        if unexpected_unreachable {
            if self.spawn_output.unreachable.len() == 1 {
                warn!(
                    logger: self.item_pool.log_capture,
                    "{log_index}{location} is unreachable on these settings!",
                    log_index = self.log_index,
                    location = self.spawn_output.unreachable.first().unwrap().identifier,
                );
            } else {
                warn!(
                    logger: self.item_pool.log_capture,
                    "{log_index}{amount} locations are unreachable on these settings!",
                    log_index = self.log_index,
                    amount = self.spawn_output.unreachable.len(),
                );
            }
        }

        self.spawn_output.unreachable.shuffle(&mut self.rng);
    }

    fn finish(self) -> SpawnOutput<'graph> {
        let spawn_node = &self.world.graph.nodes[self.world.spawn];

        // TODO something less specialized?
        match spawn_node.identifier() {
            "EastPools.Teleporter" => {
                // Lower the water at the pools teleporter if we spawn there
                self.output.commands.push_event(Event {
                    trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                    command: store_boolean(UberIdentifier::new(5377, 63173), true),
                });
            }
            "MidnightBurrows.Teleporter"
            | "MarshSpawn.Main"
            | "HowlsDen.Teleporter"
            | "EastHollow.Teleporter"
            | "GladesTown.Teleporter"
            | "InnerWellspring.Teleporter"
            | "WoodsEntry.Teleporter"
            | "WoodsMain.Teleporter"
            | "LowerReach.Teleporter"
            | "UpperDepths.Teleporter"
            | "WestPools.Teleporter"
            | "LowerWastes.FeedingGroundsTP"
            | "LowerWastes.CentralTP"
            | "UpperWastes.OuterRuinsTP"
            | "WindtornRuins.RuinsTP"
            | "WillowsEnd.InnerTP"
            | "WillowsEnd.ShriekArena" => {}
            _ => {
                if let Some(spawn_position) = spawn_node.position() {
                    self.output.commands.push_event(Event {
                        trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                        command: CommandVoid::CreateWarpIcon {
                            id: 0,
                            x: spawn_position.x.into(),
                            y: spawn_position.y.into(),
                        },
                    });
                }
            }
        }

        self.spawn_output
    }
}

#[derive(Default)]
pub struct SpawnOutput<'graph> {
    pub total_reach: Vec<&'graph LocDataEntry>,
    pub unreachable: Vec<&'graph LocDataEntry>,
}

struct RandomSpawnGenerator {
    spawns: arrayvec::IntoIter<usize, 13>,
}

impl RandomSpawnGenerator {
    fn new(
        rng: &mut Pcg64Mcg,
        graph: &Graph,
        difficulty: Difficulty,
        log_capture: &LogCapture,
    ) -> Self {
        // Precompute spawns because the size is small
        let mut spawn_identifiers = difficulty
            .spawn_locations()
            .iter()
            .copied()
            .collect::<FxHashSet<_>>();

        let mut spawns = graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| spawn_identifiers.remove(node.identifier()))
            .map(|(index, _)| index)
            .collect::<ArrayVec<_, 13>>();

        if !spawn_identifiers.is_empty() {
            warn!(
                logger: log_capture,
                "Failed to find spawn location{} {}",
                if spawn_identifiers.len() == 1 {
                    ""
                } else {
                    "s"
                },
                spawn_identifiers.iter().format(", ")
            );
        }

        spawns.shuffle(rng);

        Self {
            spawns: spawns.into_iter(),
        }
    }
}

impl Iterator for RandomSpawnGenerator {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.spawns.next()
    }
}

struct FullyRandomSpawnGenerator<'graph> {
    rng: Pcg64Mcg,
    graph: &'graph Graph,
    first: Option<usize>,
    spawns: Option<Vec<usize>>,
}

impl<'graph> FullyRandomSpawnGenerator<'graph> {
    fn new(rng: &mut Pcg64Mcg, graph: &'graph Graph) -> Result<Self, String> {
        let mut rng = Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE);

        // Postpone allocating indices because the size is big
        let (index, _) = graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.can_spawn())
            .choose(&mut rng)
            .ok_or("No valid spawn locations available")?;

        Ok(Self {
            rng,
            graph,
            first: Some(index),
            spawns: None,
        })
    }
}

impl Iterator for FullyRandomSpawnGenerator<'_> {
    type Item = usize;

    fn next(&mut self) -> Option<Self::Item> {
        self.first.take().or_else(|| {
            let spawns = self.spawns.get_or_insert_with(|| {
                let mut spawns = self
                    .graph
                    .nodes
                    .iter()
                    .enumerate()
                    .filter(|(_, node)| node.can_spawn())
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>();

                spawns.shuffle(&mut self.rng);

                spawns
            });

            spawns.pop()
        })
    }
}

fn filter_placement_locations<'a>(
    world: &World,
    log_index: &str,
    placement_locations: &mut Vec<&'a LocDataEntry>,
    output: &IntermediateOutput,
    log_capture: &LogCapture,
) {
    let mut extra_slots = vec![];

    placement_locations.retain(|pickup| {
        let condition = CommandBoolean::loc_data_condition(pickup.uber_identifier, pickup.value);
        // TODO remove by identifier instead?
        if output.modifiers.removed_locations.contains(&condition) {
            trace!(
                logger: log_capture,
                "{log_index}Manually removed {pickup} from placement locations",
                pickup = pickup.identifier
            );

            return false;
        }

        if world.loc_data_condition_met(pickup.uber_identifier, pickup.value) {
            trace!(
                logger: log_capture,
                "{log_index}Removing {pickup} from placement locations since the condition was met on spawn",
                pickup = pickup.identifier
            );

            return false;
        }

        match output.modifiers.location_slots.get(&condition) {
            None | Some(1) => {},
            Some(0) => {
                trace!(
                    logger: log_capture,
                    "{log_index}Removing {pickup} from placement locations since location slots were set to zero",
                    pickup = pickup.identifier
                );

                return false;
            }
            Some(slots) => {
                trace!(
                    logger: log_capture,
                    "{log_index}Increasing {pickup} slots to {slots}",
                    pickup = pickup.identifier
                );

                extra_slots.extend(iter::repeat_n(pickup, (slots - 1) as usize));
            }
        }

        true
    });

    placement_locations.append(&mut extra_slots);
}
