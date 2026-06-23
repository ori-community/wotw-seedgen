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
    logic_language::output::Graph,
    seed_language::{
        ast::ClientEvent,
        compile::store_boolean,
        output::{CommandVoid, CommandsOutput, Event, Trigger},
        simulate::{Simulate, Simulation, Snapshot},
    },
    Difficulty, Spawn, UberIdentifier, DEFAULT_SPAWN,
};

use crate::{
    generator::{placement::TOTAL_SPIRIT_LIGHT, SEED_FAILED_MESSAGE},
    item_pool::ItemPool,
    LogicalDifficulty, World,
};

pub fn choose_spawn<'graph>(
    rng: &mut Pcg64Mcg,
    world: &mut World<'graph, '_>,
    log_index: &str,
    item_pool: &ItemPool,
    output: &mut CommandsOutput,
) -> Result<Vec<&'graph LocDataEntry>, String> {
    let mut context = SpawnContext::new(rng, world, log_index, item_pool, output);
    context.choose_spawn()?;
    Ok(context.finish())
}

struct SpawnContext<'world, 'graph, 'settings, 'log, 'pool, 'output> {
    rng: Pcg64Mcg,
    world: &'world mut World<'graph, 'settings>,
    log_index: &'log str,
    item_pool: &'pool ItemPool,
    output: &'output mut CommandsOutput,
    default_spawn: usize,
    total_reach: Vec<&'graph LocDataEntry>,
}

impl<'world, 'graph, 'settings, 'log, 'pool, 'output>
    SpawnContext<'world, 'graph, 'settings, 'log, 'pool, 'output>
{
    fn new(
        rng: &mut Pcg64Mcg,
        world: &'world mut World<'graph, 'settings>,
        log_index: &'log str,
        item_pool: &'pool ItemPool,
        output: &'output mut CommandsOutput,
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
            total_reach: Vec::new(),
        };

        context.world.snapshot();

        context.world.spawn = default_spawn;
        context.total_reach_check();
        context.total_reach = context.world.reached_pickups().collect::<Vec<_>>();

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
                        reached_count + self.world.graph.nodes[index].is_pickup() as usize,
                    )
                },
            );

            self.world.restore_snapshot();

            if default_spawn_reached == false {
                trace!(
                    "{log_index}Discarding spawn {spawn} since {default_spawn} wasn't reached",
                    log_index = self.log_index,
                    spawn = self.world.graph.nodes[spawn].identifier(),
                    default_spawn = DEFAULT_SPAWN,
                );
            } else if reached_count != self.total_reach.len() {
                trace!(
                    "{log_index}Discarding spawn {spawn} since only {reached_count}/{total_count} locations were reached",
                    log_index = self.log_index,
                    spawn = self.world.graph.nodes[spawn].identifier(),
                    total_count = self.total_reach.len(),
                );
            } else {
                trace!(
                    "{log_index}Spawning on {spawn}",
                    log_index = self.log_index,
                    spawn = self.world.graph.nodes[spawn].identifier(),
                );

                return Ok(());
            }
        }

        Err("All available spawn locations failed to reach all locations".to_string())
    }

    fn total_reach_check<'s>(&'s mut self) {
        // TODO could avoid redoing this with a snapshot stack
        for command in &**self.item_pool {
            command.simulate(self.world, &self.output);
        }
        self.world
            .add_spirit_light(TOTAL_SPIRIT_LIGHT, &self.output);

        self.world.traverse_spawn(&self.output);
    }

    fn finish(self) -> Vec<&'graph LocDataEntry> {
        let spawn_node = &self.world.graph.nodes[self.world.spawn];

        // TODO something less specialized?
        match spawn_node.identifier() {
            "EastPools.Teleporter" => {
                // Lower the water at the pools teleporter if we spawn there
                self.output.events.push(Event {
                    trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                    command: store_boolean(UberIdentifier::new(5377, 63173), true),
                })
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
                    self.output.events.push(Event {
                        trigger: Trigger::ClientEvent(ClientEvent::Spawn),
                        command: CommandVoid::CreateWarpIcon {
                            id: 0,
                            x: spawn_position.x.into(),
                            y: spawn_position.y.into(),
                        },
                    })
                }
            }
        }

        self.total_reach
    }
}

pub struct RandomSpawnGenerator {
    spawns: arrayvec::IntoIter<usize, 13>,
}

impl RandomSpawnGenerator {
    pub fn new(rng: &mut Pcg64Mcg, graph: &Graph, difficulty: Difficulty) -> Self {
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

pub struct FullyRandomSpawnGenerator<'graph> {
    rng: Pcg64Mcg,
    graph: &'graph Graph,
    first: Option<usize>,
    spawns: Option<Vec<usize>>,
}

impl<'graph> FullyRandomSpawnGenerator<'graph> {
    pub fn new(rng: &mut Pcg64Mcg, graph: &'graph Graph) -> Result<Self, String> {
        let mut rng = Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE);

        // Postpone allocating indices because the size is big
        let (index, _) = graph
            .nodes
            .iter()
            .enumerate()
            .filter(|(_, node)| node.can_spawn())
            .choose(&mut rng)
            .ok_or_else(|| "No valid spawn locations available")?;

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
