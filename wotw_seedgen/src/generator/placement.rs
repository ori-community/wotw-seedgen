use super::{
    item_pool::ItemPool, spirit_light::SpiritLightProvider, Seed, SeedUniverse, SEED_FAILED_MESSAGE,
};
use crate::{
    generator::{
        entrances::generate_entrances,
        solutions::{solution_weights, Solution, SolutionLike, SOLUTION_MAX_ITEMS},
    },
    item_pool::ItemPoolBuilder,
    logical_difficulty::LogicalDifficulty,
    spoiler::{NodeSummary, SeedSpoiler, SpoilerGroup, SpoilerItem, SpoilerPlacement},
    World,
};
use itertools::Itertools;
use log::{log_enabled, trace, warn, Level::Trace};
use rand::{
    distributions::{Uniform, WeightedIndex},
    prelude::Distribution,
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use std::{cmp::Ordering, fmt::Display, iter, mem, ops::RangeFrom, sync::LazyLock};
use wotw_seedgen_data::{
    assets::{LocData, LocDataEntry},
    logic_language::output::Node,
    seed_language::{
        compile,
        output::{
            ClientEvent, CommandBoolean, CommandString, CommandVoid, Concatenator, ContainedWrites,
            Event, IntermediateOutput, IntoConstant, Operation, Trigger,
        },
        simulate::{Simulate, Simulation, Snapshot},
    },
    UberIdentifier, UniverseSettings,
};
use wotw_seedgen_data::{
    seed_language::{
        compile::store_boolean,
        output::{postprocess, AsConstant},
    },
    DEFAULT_SPAWN,
};
use wotw_seedgen_seed::SeedgenInfo;

pub(super) const SPAWN_SLOTS: usize = 7;
const UNSHARED_ITEMS: usize = 5; // How many items to place per world that are guaranteed not being sent to another world
const TOTAL_SPIRIT_LIGHT: i32 = 20000;

const MIN_PLACEHOLDERS: usize = 3;
static MAX_PLACEHOLDERS: LazyLock<usize> =
    LazyLock::new(|| 10 + SOLUTION_MAX_ITEMS.saturating_mul(2));

pub fn generate_placements(
    rng: &mut Pcg64Mcg,
    worlds: Vec<(World, IntermediateOutput)>,
    settings: &UniverseSettings,
    loc_data: &LocData,
    debug: bool,
) -> Result<SeedUniverse, String> {
    assert!(
        !worlds.is_empty(),
        "Need at least one world to generate a seed"
    );
    let mut context = Context::new(rng, worlds, settings)?;

    context.preplacements();

    loop {
        context.next_step();
        context.update_reached();

        if context.is_everything_reached() {
            context.place_remaining();
            context.sort_spoiler_placements();

            break;
        }

        if context.force_keystones() {
            continue;
        }

        if !context.place_random() {
            if let Some((target_world_index, progression)) = context.choose_progression()? {
                context.place_forced(target_world_index, progression);
            }
        }
    }

    Ok(context.finish(loc_data, debug, rng))
}

pub struct Context<'graph, 'settings> {
    pub rng: Pcg64Mcg,
    pub worlds: Vec<WorldContext<'graph, 'settings>>,
    settings: &'settings UniverseSettings,
    /// Distribution for random orderings
    ordering_distribution: OrderingDistribution,
    /// next multiworld uberState id to use
    multiworld_state_index: RangeFrom<i32>,
    /// current placement step
    step: usize,
    /// spoiler being populated over the course of generation
    spoiler: SeedSpoiler,
}

pub struct WorldContext<'graph, 'settings> {
    pub rng: Pcg64Mcg,
    pub world: World<'graph, 'settings>,
    pub output: IntermediateOutput,
    /// world index of this world
    index: usize,
    /// ready-made string for referencing this world in the log
    log_index: String,
    /// remaining items to place
    item_pool: ItemPool,
    /// generates appropriate spirit light amounts
    spirit_light_provider: SpiritLightProvider,
    /// all remaining pickups which need to be assigned random placements
    needs_placement: Vec<&'graph LocDataEntry>,
    /// initial length of needs_placement
    total_pickups: f32,
    /// cost of ks doors already opened on spawn, which will be ignored for forced keystones
    initial_ks_cost: usize,
    /// how many pickups should be assigned spirit light placements
    spirit_light_placements_remaining: usize,
    /// pickups which have been reached but explicitely haven't been assigned a placement yet to leave space for later progressions
    placeholders: Vec<&'graph LocDataEntry>,
    /// indices into `needs_placement` for pickups that are reachable and may be used for placements in this step
    reached_needs_placement: Vec<usize>,
    /// indices into `needs_placement` for pickups that have received a placement and should be removed before the next placement step
    received_placement: Vec<usize>,
    /// number of pickups in `reached` that can give items
    reached_item_locations: usize,
    /// number of remaining allowed placements on spawn
    spawn_slots: usize,
    // TODO is this still needed for multiworld quality?
    /// number of remaining placements that should not be placed outside of the own world
    unshared_items: usize,
}

impl<'graph, 'settings> Context<'graph, 'settings> {
    fn new(
        rng: &mut Pcg64Mcg,
        worlds: Vec<(World<'graph, 'settings>, IntermediateOutput)>,
        settings: &'settings UniverseSettings,
    ) -> Result<Self, String> {
        let multiworld = worlds.len() > 1;

        let mut worlds = worlds
            .into_iter()
            .enumerate()
            .map(|(index, (world, output))| {
                WorldContext::new(rng, world, output, index, multiworld)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let mut total_needs_placement = 0;
        let mut total_item_count = 0;

        for world in &worlds {
            total_needs_placement += world.needs_placement.len();
            total_item_count += world.item_pool.len();
        }

        let mut total_spirit_light_placements = usize::max(
            total_needs_placement.saturating_sub(total_item_count),
            worlds.len(),
        ) as f32;
        let mut total_needs_placement = total_needs_placement as f32;

        for world in &mut worlds {
            let needs_placement = world.needs_placement.len() as f32;
            let spirit_light_placements =
                (total_spirit_light_placements * (needs_placement / total_needs_placement)).round();

            total_spirit_light_placements -= spirit_light_placements;
            total_needs_placement -= needs_placement;

            trace!(
                "{log_index}Assigned {spirit_light_placements}/{needs_placement} placements for spirit light",
                log_index = world.log_index,
            );

            debug_assert!(spirit_light_placements <= needs_placement);

            world.spirit_light_placements_remaining = spirit_light_placements as usize;
            // TODO how should !add_item(spirit_light(100)) behave?
            world
                .spirit_light_provider
                .init(TOTAL_SPIRIT_LIGHT, world.spirit_light_placements_remaining);
        }

        let ordering_distribution = OrderingDistribution::new(rng);

        let spawns = worlds
            .iter()
            .map(|world_context| {
                world_context.world.graph.nodes[world_context.world.spawn]
                    .identifier()
                    .to_string()
            })
            .collect();

        // TODO is this possible earlier to avoid the need to filter through nodes?
        // otherwise, it would at least be unnecessary if no world has entrance randomization
        let entrance_identifier_map = worlds[0]
            .world
            .graph
            .nodes
            .iter()
            .filter_map(Node::try_as_anchor_ref)
            .filter_map(|anchor| {
                anchor
                    .entrance
                    .as_ref()
                    .map(|entrance| (entrance.id, &anchor.identifier))
            })
            .collect::<FxHashMap<_, _>>();

        let entrances = worlds
            .iter()
            .map(|world_context| {
                if world_context.world.settings.randomize_entrances.is_some() {
                    let mut entrances = (1..=32)
                        .map(|entrance_id| {
                            let target_entrance_id = world_context
                                .world
                                .fetch_integer(UberIdentifier::entrances(entrance_id));

                            (entrance_id, target_entrance_id)
                        })
                        .collect::<Vec<_>>();

                    entrances.sort_unstable_by_key(|(_, target)| *target);

                    entrances
                        .into_iter()
                        .map(|(from, to)| {
                            (
                                entrance_identifier_map[&from].clone(),
                                entrance_identifier_map[&to].clone(),
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect();

        // TODO move some of the above logic into SeedSpoiler::new?
        let spoiler = SeedSpoiler::new(spawns, entrances);

        Ok(Self {
            rng: Pcg64Mcg::from_rng(&mut *rng).expect(SEED_FAILED_MESSAGE),
            worlds,
            settings,
            ordering_distribution,
            multiworld_state_index: 0..,
            step: 0,
            spoiler,
        })
    }

    fn preplacements(&mut self) {
        for world_context in &mut self.worlds {
            world_context.preplacements(&mut self.spoiler.preplacements);
        }
    }

    fn next_step(&mut self) {
        self.sort_spoiler_placements();

        self.step += 1;
        trace!("--- Placement step #{}", self.step);

        self.spoiler.groups.push(SpoilerGroup::default());
    }

    fn sort_spoiler_placements(&mut self) {
        if self.step > 0 {
            self.spoiler.groups[self.step - 1]
                .placements
                .sort_unstable_by(|a, b| {
                    match (
                        a.item.command.contained_common_items().next(),
                        b.item.command.contained_common_items().next(),
                    ) {
                        (None, None) => b.item.name.cmp(&a.item.name),
                        (Some(_), None) => Ordering::Greater,
                        (None, Some(_)) => Ordering::Less,
                        // TODO spirit light amount ordering
                        (Some(a), Some(b)) => b.cmp(&a),
                    }
                });
        }
    }

    fn update_reached(&mut self) {
        for world_context in &mut self.worlds {
            world_context.update_reached();
        }

        self.write_reachable_spoiler()
    }

    fn write_reachable_spoiler(&mut self) {
        self.spoiler.groups[self.step - 1].reachable = self
            .worlds
            .iter()
            .map(|world_context| {
                world_context
                    .reached_needs_placement
                    .iter()
                    .map(|index| NodeSummary::new(world_context.needs_placement[*index]))
                    .collect()
            })
            .collect();
    }

    fn is_everything_reached(&self) -> bool {
        self.worlds
            .iter()
            .all(|world| world.reached_needs_placement.len() == world.needs_placement.len())
    }

    fn force_keystones(&mut self) -> bool {
        let mut new_progressions = false;

        for world_index in 0..self.worlds.len() {
            let world_context = &mut self.worlds[world_index];

            let owned_keystones = world_context.world.keystones();
            if owned_keystones < 2 {
                continue;
            }

            let required_keystones =
                world_context.world.reached_ks_cost() - world_context.initial_ks_cost;
            let missing_keystones = required_keystones.saturating_sub(owned_keystones as usize);
            if missing_keystones == 0 {
                continue;
            }

            // If we had fewer than 4 keystones total so far, the forced keystones might open new progressions.
            // Keystones never get removed from the inventory, so once 4 have been placed doors are always solved.
            new_progressions = owned_keystones < 4;

            trace!(
                "{}Placing {missing_keystones} keystones to avoid keylocks",
                world_context.log_index
            );

            let keystone = compile::keystone();

            if !self.worlds[world_index]
                .item_pool
                .find_remove_amount(&keystone, missing_keystones)
            {
                warn!("Not enough keystones in the item pool for forced keystone progression, placing anyway");
            }

            for _ in 0..missing_keystones {
                self.force_place_command(keystone.clone(), world_index, true);
            }
        }

        new_progressions
    }

    fn place_remaining(&mut self) {
        trace!("All locations reached. Placing remaining items");

        for target_world_index in 0..self.worlds.len() {
            for command in self.worlds[target_world_index].item_pool.take() {
                self.force_place_command(command, target_world_index, false);
            }
        }

        for world_context in &mut self.worlds {
            world_context.update_needs_placement();
            world_context.fill_remaining(&mut self.spoiler.groups[self.step - 1].placements);
        }
    }

    fn place_random(&mut self) -> bool {
        let mut any_placed = false;

        for origin_world_index in 0..self.worlds.len() {
            let origin_world = &mut self.worlds[origin_world_index];

            let needs_random_placement = origin_world.reserve_placeholders();
            let mut placements_remaining =
                origin_world.placements_remaining() + needs_random_placement.len();

            for pickup in needs_random_placement {
                any_placed = true; // TODO pull out of loop and skip some more calculations that way

                let origin_world = &mut self.worlds[origin_world_index];

                let should_place_spirit_light = !pickup.uber_identifier.is_shop()
                    && self.rng.gen_bool(
                        origin_world.spirit_light_placements_remaining as f64
                            / placements_remaining as f64,
                    );

                let (target_world_index, command) = if should_place_spirit_light {
                    let batch = origin_world.spirit_light_provider.take();
                    origin_world.spirit_light_placements_remaining -= 1;

                    (
                        origin_world_index,
                        compile::spirit_light(batch.into(), &mut self.rng),
                    )
                } else {
                    let target_world_index = self.choose_target_world_for_random_placement();
                    let target_world = &mut self.worlds[target_world_index];

                    let item = target_world
                        .item_pool
                        .choose_random()
                        .unwrap_or_else(|| target_world.backup_gorlek_ore());

                    (target_world_index, item)
                };

                self.place_command_at(
                    command,
                    pickup,
                    origin_world_index,
                    target_world_index,
                    false,
                );

                placements_remaining -= 1;
            }
        }

        any_placed
    }

    fn choose_progression(&mut self) -> Result<Option<(usize, Solution)>, String> {
        let slots = self.progression_slots();

        let mut world_indices = (0..self.worlds.len())
            .map(|index| {
                let world = &self.worlds[index];
                let placements_remaining =
                    world.placements_remaining() as f32 / world.total_pickups;

                trace!(
                    "{log_index}{placements_remaining:.2}% placements remaining",
                    log_index = world.log_index,
                    placements_remaining = placements_remaining * 100.,
                );

                // Needs to be rolled in advance, otherwise we'd violate the total ordering that the sort expects
                let random_ordering = self.ordering_distribution.sample();

                (index, (placements_remaining, random_ordering))
            })
            .collect::<Vec<_>>();

        world_indices.sort_unstable_by(
            |(_, (a_placements_remaining, a_random_ordering)),
             (_, (b_placements_remaining, b_random_ordering))| {
                b_placements_remaining
                    .total_cmp(a_placements_remaining)
                    .then_with(|| a_random_ordering.cmp(b_random_ordering))
            },
        );

        for (target_world_index, _) in world_indices {
            if let Some(progression) = self.worlds[target_world_index].choose_progression(slots) {
                return Ok(Some((target_world_index, progression)));
            }
        }

        trace!(
            "Unable to find any possible forced progression\n{}",
            self.worlds.iter().format_with("\n", |world_context, f| {
                f(&format_args!(
                    "{index}{len} unreached locations: {identifiers}\nwith these items: {inventory}\nand this item pool: {item_pool}",
                    index = world_context.log_index,
                    len = world_context.needs_placement.len(),
                    identifiers = format_pickups(&world_context.needs_placement),
                    inventory = world_context.world.inventory_display(),
                    item_pool = world_context.item_pool,
                ))
            })
        );

        self.flush_item_pool()?;
        Ok(None)
    }

    fn progression_slots(&self) -> usize {
        self.worlds
            .iter()
            .map(|world_context| world_context.progression_slots())
            .sum()
    }

    fn flush_item_pool(&mut self) -> Result<(), String> {
        // TODO implement new recovery mechanism
        // trace!("Placing items which modify uberStates to attempt recovery");

        Err("Failed to reach all locations".to_string())
    }

    fn place_forced(&mut self, target_world_index: usize, mut progression: Solution) {
        progression.items.sort_unstable();
        for item in progression.items.into_iter().rev() {
            let command = self.worlds[target_world_index].item_pool.remove(item);
            self.force_place_command(command, target_world_index, true);
        }

        self.worlds[target_world_index].force_place_spirit_light(
            progression.spirit_light,
            &mut self.spoiler.groups[self.step - 1].placements,
        );
    }

    fn force_place_command(
        &mut self,
        command: CommandVoid,
        target_world_index: usize,
        mark_forced: bool,
    ) {
        let origin_world_index = self.choose_origin_world_for_forced_placement(target_world_index);
        let origin_world = &mut self.worlds[origin_world_index];

        match origin_world.choose_non_spirit_light_location() {
            None => {
                if origin_world.spawn_slots > 0 {
                    origin_world.spawn_slots -= 1;

                    trace!(
                        "Placing {target_index}{name} at {origin_index}Spawn",
                        name = self.worlds[target_world_index].log_name(&command),
                        target_index = self.worlds[target_world_index].log_index,
                        origin_index = self.worlds[origin_world_index].log_index
                    );
                } else {
                    warn!(
                        "Not enough space to place {target_index}{name}, placing at Spawn despite already having too many spawn items",
                        name = self.worlds[target_world_index].log_name(&command),
                        target_index = self.worlds[target_world_index].log_index,
                    );
                }

                self.write_placement_spoiler(
                    origin_world_index,
                    target_world_index,
                    NodeSummary::spawn(),
                    &command,
                    mark_forced,
                );

                self.push_command(
                    Trigger::ClientEvent(ClientEvent::Spawn),
                    command,
                    origin_world_index,
                    target_world_index,
                );
            }
            Some(pickup) => {
                self.place_command_at(
                    command,
                    pickup,
                    origin_world_index,
                    target_world_index,
                    mark_forced,
                );
            }
        }
    }

    // TODO might be worth to do some more single-world happy paths?
    fn choose_origin_world_for_forced_placement(&mut self, target_world_index: usize) -> usize {
        if self.worlds.len() == 1 {
            return target_world_index;
        }

        let target_world = &mut self.worlds[target_world_index];

        if target_world.unshared_items > 0 {
            trace!(
                "{}is not allowed to share items yet, forcing item placement in own world",
                target_world.log_index
            );

            target_world.unshared_items -= 1;
            target_world_index
        } else {
            let mut world_indices = (0..self.worlds.len()).collect::<Vec<_>>();
            world_indices.shuffle(&mut self.rng);

            // TODO we're doing some redundant work here
            // we already figure out whether we have to use the spawn slots here but later we don't use that information
            // and have to recalculate it
            let origin_world_index = world_indices
                .iter()
                .find(|index| self.worlds[**index].non_spawn_progression_slots() > 0)
                .copied()
                .or_else(|| {
                    world_indices
                        .into_iter()
                        .find(|index| self.worlds[*index].spawn_slots > 0)
                })
                .unwrap_or(target_world_index); // Overplace spawn slots if there's no other way

            origin_world_index
        }
    }

    fn choose_target_world_for_random_placement(&mut self) -> usize {
        let mut world_indices = (0..self.worlds.len()).collect::<Vec<_>>();
        world_indices.shuffle(&mut self.rng);
        world_indices
            .into_iter()
            .find_or_last(|index| !self.worlds[*index].item_pool.is_empty())
            .unwrap()
    }

    fn origin_name(&self, command: &CommandVoid, target_world_index: usize) -> CommandString {
        let name = self.worlds[target_world_index].name(command);

        match name.into_constant() {
            Ok(value) => format!("<world>{target_world_index}</>'s {value}").into(),
            Err(name) => CommandString::Concatenate {
                operation: Box::new(Operation {
                    left: format!("<world>{target_world_index}</>'s").into(),
                    operator: Concatenator::Concat,
                    right: name,
                }),
            },
        }
    }

    fn place_command_at(
        &mut self,
        command: CommandVoid,
        pickup: &LocDataEntry,
        origin_world_index: usize,
        target_world_index: usize,
        mark_forced: bool,
    ) {
        trace!(
            "Placing {target_index}{log_name} at {origin_index}{pickup}",
            log_name = self.worlds[target_world_index].log_name(&command),
            target_index = self.worlds[target_world_index].log_index,
            origin_index = self.worlds[origin_world_index].log_index,
            pickup = pickup.identifier,
        );

        self.write_placement_spoiler(
            origin_world_index,
            target_world_index,
            NodeSummary::new(pickup),
            &command,
            mark_forced,
        );

        self.push_command(
            Trigger::loc_data_trigger(pickup.uber_identifier, pickup.value),
            command,
            origin_world_index,
            target_world_index,
        );
    }

    fn push_command(
        &mut self,
        trigger: Trigger,
        mut command: CommandVoid,
        origin_world_index: usize,
        target_world_index: usize,
    ) {
        if origin_world_index == target_world_index {
            self.worlds[origin_world_index].push_command(trigger, command);
        } else {
            let uber_identifier = self.multiworld_state();
            let message = self.origin_name(&command, target_world_index);
            let message_command = CommandVoid::QueuedMessage {
                id: None,
                priority: false,
                message,
                timeout: None,
            };
            let store_command = store_boolean(uber_identifier, true);

            self.worlds[origin_world_index].push_command(
                trigger,
                CommandVoid::Multi {
                    commands: vec![message_command, store_command],
                },
            );

            // Append 'from <world>' to all messages
            for message in command.contained_messages_mut() {
                *message = match message.as_constant() {
                    Some(value) => format!("{value} from <world>{origin_world_index}</>").into(),
                    _ => CommandString::Concatenate {
                        operation: Box::new(Operation {
                            left: message.clone(),
                            operator: Concatenator::Concat,
                            right: format!("from <world>{origin_world_index}</>").into(),
                        }),
                    },
                }
            }

            self.worlds[target_world_index].push_command(
                Trigger::Binding(uber_identifier), // this is server synced and can't change to false
                command,
            );
        }
    }

    fn multiworld_state(&mut self) -> UberIdentifier {
        UberIdentifier::multiworld(self.multiworld_state_index.next().unwrap())
    }

    fn write_placement_spoiler(
        &mut self,
        origin_world_index: usize,
        target_world_index: usize,
        location: NodeSummary,
        command: &CommandVoid,
        mark_forced: bool,
    ) {
        let item = self.spoiler_item(target_world_index, command);

        let group = &mut self.spoiler.groups[self.step - 1];

        if mark_forced {
            group.forced_items.push(item.clone());
        }

        let placement = SpoilerPlacement {
            origin_world_index,
            target_world_index,
            location,
            item,
        };

        group.placements.push(placement);
    }

    fn spoiler_item(&mut self, target_world_index: usize, command: &CommandVoid) -> SpoilerItem {
        SpoilerItem {
            command: command.clone(),
            name: self.worlds[target_world_index].log_name(command),
        }
    }

    fn finish(mut self, loc_data: &LocData, debug: bool, rng: &mut Pcg64Mcg) -> SeedUniverse {
        let mut output = self
            .worlds
            .iter_mut()
            .map(|world| &mut world.output)
            .collect::<Vec<_>>();

        let placeholder_maps = postprocess(&mut output, loc_data, rng);

        SeedUniverse {
            worlds: self
                .worlds
                .into_iter()
                .zip(placeholder_maps)
                .map(|(mut world_context, placeholder_map)| {
                    assert!(
                        world_context.output.icons.is_empty(),
                        "custom icons in seedgen aren't supported"
                    ); // TODO custom icons in snippets

                    let spawn = &world_context.world.graph.nodes[world_context.world.spawn];
                    world_context.output.spawn = Some(*spawn.position().unwrap());

                    // Debug variant for the uppercase formatting
                    world_context
                        .output
                        .tags
                        .push(format!("{:?}", world_context.world.settings.difficulty));

                    let seedgen_info = SeedgenInfo {
                        universe_settings: self.settings.clone(),
                        world_index: world_context.index,
                        spawn_identifier: spawn.identifier().to_string(),
                    };

                    Seed::new(world_context.output, placeholder_map, debug)
                        .with_seedgen_info(seedgen_info)
                })
                .collect(),
            spoiler: self.spoiler,
        }
    }
}

impl<'graph, 'settings> WorldContext<'graph, 'settings> {
    fn new(
        rng: &mut Pcg64Mcg,
        mut world: World<'graph, 'settings>,
        mut output: IntermediateOutput,
        index: usize,
        multiworld: bool,
    ) -> Result<Self, String> {
        let mut rng = Pcg64Mcg::from_rng(&mut *rng).expect(SEED_FAILED_MESSAGE);

        let log_index = if multiworld {
            format!("[{index}] ")
        } else {
            String::new()
        };

        trace!(
            "{log_index}Spawning on {}",
            world.graph.nodes[world.spawn].identifier()
        );

        let mut item_pool = ItemPoolBuilder::new(&mut rng);

        for (command, amount) in mem::take(&mut output.item_pool_changes) {
            if amount >= 0 {
                item_pool.add_amount(command.clone(), amount as usize);
            } else {
                item_pool.remove_amount(&command, (-amount) as usize);
            }
        }

        let item_pool = item_pool.finish();

        world.simulate(&ClientEvent::Spawn, &output.events);
        world.simulate(&ClientEvent::Reload, &output.events);

        let initial_ks_cost = world.reached_ks_cost();

        // TODO instead of timing this after the spawn simulation to avoid unsettings the known entrance connections,
        // maybe this could be resolved with whatever mechanism will implement launch fragments behaving differently
        // between client and simulation?
        generate_entrances(&mut world, &mut output.events, &mut rng)?;

        let needs_placement = total_reach_check(&mut world, &log_index, &output, &item_pool);
        let total_pickups = needs_placement.len() as f32;

        world.traverse_spawn(&output.events);

        let spirit_light_provider = SpiritLightProvider::new(&mut rng);

        Ok(Self {
            rng,
            world,
            output,
            index,
            log_index,
            item_pool,
            spirit_light_provider,
            needs_placement,
            total_pickups,
            initial_ks_cost,
            spirit_light_placements_remaining: 0,
            placeholders: Default::default(),
            reached_needs_placement: Default::default(),
            received_placement: Default::default(),
            reached_item_locations: Default::default(),
            spawn_slots: SPAWN_SLOTS,
            unshared_items: UNSHARED_ITEMS,
        })
    }

    fn preplacements(&mut self, preplacement_spoiler: &mut Vec<SpoilerPlacement>) {
        trace!("{}Generating preplacements", self.log_index);

        self.hi_torin(preplacement_spoiler);

        let mut zone_needs_placement = FxHashMap::default();

        for (command, zone) in mem::take(&mut self.output.preplacements) {
            let pickup_indices = zone_needs_placement.entry(zone).or_insert_with(|| {
                self.needs_placement
                    .iter()
                    .enumerate()
                    .filter(|(_, pickup)| pickup.zone == zone)
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>()
            });

            if pickup_indices.is_empty() {
                let name = self.log_name(&command);
                warn!(
                    "{}Failed to preplace {name} in {zone} since no free placement location was available",
                    self.log_index
                );

                continue;
            }

            // We prefer generating indices over shuffling the nodes because usually there aren't many zone preplacements (relics)
            let pickup_index =
                pickup_indices.swap_remove(self.rng.gen_range(0..pickup_indices.len()));
            let pickup = self.needs_placement[pickup_index];

            self.place_without_simulation(pickup, command, preplacement_spoiler);
            self.received_placement.push(pickup_index);
        }
    }

    fn hi_torin(&mut self, preplacement_spoiler: &mut Vec<SpoilerPlacement>) {
        let command = compile::spirit_light(1.into(), &mut self.rng);

        if self.needs_placement.is_empty() {
            let name = self.log_name(&command);

            warn!(
                "{}Failed to preplace {name} since no free placement location was available",
                self.log_index
            );
        } else if self.spirit_light_placements_remaining == 0 {
            let name = self.log_name(&command);

            warn!(
                "{}Failed to preplace {name} since no spirit light placement location was available",
                self.log_index
            );
        } else {
            self.spirit_light_placements_remaining -= 1;

            let pickup = self
                .needs_placement
                .swap_remove(self.rng.gen_range(0..self.needs_placement.len()));

            self.place_without_simulation(pickup, command, preplacement_spoiler);
        }
    }

    fn update_reached(&mut self) {
        self.update_needs_placement();

        self.reached_needs_placement = self
            .needs_placement
            .iter()
            .enumerate()
            .filter(|(_, pickup)| self.world.reached_pickups().contains(**pickup))
            .map(|(index, _)| index)
            .collect();

        self.reached_item_locations = self.world.reached_pickup_count();

        trace!(
            "{log_index}{amount} reached location{location_s} that need{need_s} placements: {reached_needs_placement}",
            log_index = self.log_index,
            amount = self.reached_needs_placement.len(),
            location_s = if self.reached_needs_placement.len() != 1 { "s" } else { "" },
            need_s = if self.reached_needs_placement.len() == 1 { "s" } else { "" },
            reached_needs_placement = self
                .reached_needs_placement
                .iter()
                .map(|index| &self.needs_placement[*index].identifier)
                .format(", ")
        );
    }

    fn update_needs_placement(&mut self) {
        let mut received_placement = mem::take(&mut self.received_placement);
        received_placement.sort_unstable();

        for pickup_index in received_placement.into_iter().rev() {
            self.needs_placement.swap_remove(pickup_index);
        }
    }

    fn placements_remaining(&self) -> usize {
        self.needs_placement.len() - self.received_placement.len() + self.placeholders.len()
    }

    fn reserve_placeholders(&mut self) -> Vec<&'graph LocDataEntry> {
        self.received_placement
            .extend(self.reached_needs_placement.iter().copied());

        let desired_placeholders = usize::max(
            MIN_PLACEHOLDERS,
            usize::min(
                *MAX_PLACEHOLDERS,
                usize::max(
                    self.placeholders.len(),
                    (self.reached_needs_placement.len() + self.placeholders.len()) / 2,
                ),
            ),
        );
        let new_placeholders = usize::min(desired_placeholders, self.reached_needs_placement.len());
        let kept_placeholders = usize::min(
            desired_placeholders - new_placeholders,
            self.placeholders.len(),
        );

        let released_placeholders = self.placeholders.split_off(kept_placeholders);
        let placeholders = self
            .reached_needs_placement
            .split_off(self.reached_needs_placement.len() - new_placeholders)
            .into_iter()
            .map(|index| self.needs_placement[index]);

        self.placeholders.extend(placeholders);
        self.placeholders.shuffle(&mut self.rng);

        let needs_placement = mem::take(&mut self.reached_needs_placement)
            .into_iter()
            .map(|index| self.needs_placement[index])
            .chain(released_placeholders)
            .collect::<Vec<_>>();

        trace!(
            "{log_index}Keeping {amount}/{total} placeholders: {placeholders}",
            log_index = self.log_index,
            amount = self.placeholders.len(),
            total = self.placeholders.len() + needs_placement.len(),
            placeholders = format_pickups(&self.placeholders),
        );

        needs_placement
    }

    fn progression_slots(&self) -> usize {
        self.non_spawn_progression_slots() + self.spawn_slots
    }

    fn non_spawn_progression_slots(&self) -> usize {
        usize::min(
            self.reached_needs_placement.len() + self.placeholders.len(),
            self.placements_remaining() - self.spirit_light_placements_remaining,
        )
    }

    fn spirit_light_progression_slots(&self) -> usize {
        usize::min(
            self.reached_needs_placement
                .iter()
                .map(|pickup_index| &self.needs_placement[*pickup_index])
                .chain(&self.placeholders)
                .filter(|pickup| !pickup.uber_identifier.is_shop())
                .count(),
            self.spirit_light_placements_remaining,
        )
    }

    fn choose_progression(&mut self, slots: usize) -> Option<Solution> {
        trace!("{}Attempting forced progression", self.log_index);

        let progressions = self.world.find_solutions(
            &self.item_pool,
            &self.output.events,
            slots,
            self.spirit_light_progression_slots(),
            None,
        );

        if progressions.is_empty() {
            trace!("{}No forced progression found", self.log_index);

            return None;
        }

        let mut with_weights = self.calculate_weights(progressions, slots);

        let weights = WeightedIndex::new(with_weights.iter().map(|(_, weight)| *weight)).unwrap();
        let (progression, _) = with_weights.swap_remove(weights.sample(&mut self.rng));

        Some(progression)
    }

    fn calculate_weights(
        &mut self,
        progressions: Vec<Solution>,
        slots: usize,
    ) -> Vec<(Solution, f32)> {
        let mut with_weights =
            solution_weights(progressions, &self.item_pool, slots, self.spawn_slots);

        // The order returned by find_solutions is not portable, so we have to sort before our weighted choice.
        // The weights are a good pick for primary key because they are fast to compare and quite unique;
        // the backup solution comparison has some minor optimizations as well.
        // As a bonus, we already have sorted weights for the trace log, which is why we order bigger first.
        with_weights.sort_unstable_by(|(a, a_weight), (b, b_weight)| {
            b_weight.total_cmp(a_weight).then_with(|| a.cmp(&b))
        });

        if log_enabled!(Trace) {
            self.log_weights(&with_weights);
        }

        with_weights
    }

    // seedgen output should remain the same whether logging is enabled or not, so we have to sort an owned clone
    fn log_weights(&mut self, progressions: &[(Solution, f32)]) {
        let total_weight = progressions.iter().map(|(_, weight)| weight).sum::<f32>();

        trace!(
            "{log_index}{amount} option{s} for forced progression:\n{progressions}",
            log_index = self.log_index.clone(),
            amount = progressions.len(),
            s = if progressions.len() == 1 { "" } else { "s" },
            progressions = {
                progressions
                    .into_iter()
                    .format_with("\n", |(solution, weight), f| {
                        f(&format_args!(
                            "- {chance:.2}% (reaches {new_reached}): {items}",
                            chance = (weight / total_weight) * 100.,
                            new_reached = solution.new_reached,
                            items = solution.display(&self.item_pool, None), // TODO this was able to use log_name before
                        ))
                    })
            }
        );
    }

    fn force_place_spirit_light(
        &mut self,
        mut amount: i32,
        placement_spoiler: &mut Vec<SpoilerPlacement>,
    ) {
        while amount > 0 {
            let batch = self.spirit_light_provider.take();
            amount = amount.saturating_sub(batch);

            match self.choose_spirit_light_location() {
                None => {
                    warn!("Not enough space to place spirit light, aborting progression");
                    break;
                }
                Some(pickup) => self.place_spirit_light(pickup, batch, placement_spoiler),
            }
        }
    }

    fn choose_non_spirit_light_location(&mut self) -> Option<&'graph LocDataEntry> {
        let any_remaining = self.placements_remaining() > self.spirit_light_placements_remaining;

        if any_remaining {
            if !self.reached_needs_placement.is_empty() {
                let index = self.rng.gen_range(0..self.reached_needs_placement.len());
                Some(self.commit_chosen_location(index))
            } else {
                self.placeholders.pop()
            }
        } else {
            None
        }
    }

    fn choose_spirit_light_location(&mut self) -> Option<&'graph LocDataEntry> {
        match self
            .reached_needs_placement
            .iter()
            .enumerate()
            .filter(|(_, pickup_index)| {
                !self.needs_placement[**pickup_index]
                    .uber_identifier
                    .is_shop()
            })
            .map(|(index, _)| index)
            // TODO shuffle instead?
            .choose(&mut self.rng)
        {
            None => {
                let (index, _) = self
                    .placeholders
                    .iter()
                    .enumerate()
                    .find(|(_, pickup)| !pickup.uber_identifier.is_shop())?;

                Some(self.placeholders.swap_remove(index))
            }
            Some(index) => Some(self.commit_chosen_location(index)),
        }
    }

    fn commit_chosen_location(&mut self, index: usize) -> &'graph LocDataEntry {
        let pickup_index = self.reached_needs_placement.swap_remove(index);
        self.received_placement.push(pickup_index);

        self.needs_placement[pickup_index]
    }

    pub fn name(&self, command: &CommandVoid) -> CommandString {
        self.output.item_metadata.get(command).force_name()
    }

    fn log_name(&mut self, command: &CommandVoid) -> String {
        self.output
            .item_metadata
            .get(command)
            .log_name(&mut self.world, &self.output.events)
    }

    fn fill_remaining(&mut self, placement_spoiler: &mut Vec<SpoilerPlacement>) {
        trace!(
            "{}Filling remaining locations with spirit light",
            self.log_index
        );

        let mut needs_placement = mem::take(&mut self.needs_placement);
        needs_placement.extend(mem::take(&mut self.placeholders));
        needs_placement.shuffle(&mut self.rng);

        for pickup in needs_placement {
            let is_shop = pickup.uber_identifier.is_shop();

            if is_shop {
                let command = self.backup_gorlek_ore();
                self.place_with_simulation(
                    pickup,
                    command,
                    placement_spoiler,
                    |_, world, events| world.add_gorlek_ore(1, events),
                );
            } else {
                let amount = self.spirit_light_provider.take();
                self.place_spirit_light(pickup, amount, placement_spoiler);
            }
        }

        let mut unreachable_count = 0;

        for (index, node) in self.world.graph.nodes.iter().enumerate() {
            if let Node::Pickup(pickup) = node {
                if !self.world.has_reached(index) {
                    trace!(
                        "{}Placing extra spirit light in unreachable location {}",
                        self.log_index,
                        pickup.identifier,
                    );

                    let amount = self.spirit_light_provider.take_exceed();
                    let command = compile::spirit_light(amount.into(), &mut self.rng);
                    self.place_without_simulation(pickup, command, placement_spoiler);

                    unreachable_count += 1;
                }
            }
        }

        if unreachable_count != self.world.settings.difficulty.expected_unreachable() {
            warn!(
                "{}{unreachable_count} location{} unreachable on these settings!",
                self.log_index,
                if unreachable_count == 1 {
                    " is"
                } else {
                    "s are"
                }
            );
        }
    }

    fn backup_gorlek_ore(&mut self) -> CommandVoid {
        // TODO try to avoid
        let command = compile::gorlek_ore();

        warn!(
            "{index}Placing more {name} than intended to avoid placing Spirit Light in a shop",
            name = self.log_name(&command),
            index = self.log_index,
        );

        command
    }

    fn place_with_simulation<F>(
        &mut self,
        pickup: &LocDataEntry,
        command: CommandVoid,
        placement_spoiler: &mut Vec<SpoilerPlacement>,
        simulate: F,
    ) where
        F: FnOnce(&CommandVoid, &mut World<'graph, 'settings>, &[Event]),
    {
        trace!(
            "{index}Placing {name} at {pickup}",
            name = self.log_name(&command),
            index = self.log_index,
            pickup = pickup.identifier,
        );

        self.write_placement_spoiler(pickup, &command, placement_spoiler);

        self.push_command_with_simulation(
            Trigger::loc_data_trigger(pickup.uber_identifier, pickup.value),
            command,
            simulate,
        );
    }

    fn place_spirit_light(
        &mut self,
        pickup: &LocDataEntry,
        amount: i32,
        placement_spoiler: &mut Vec<SpoilerPlacement>,
    ) {
        // TODO not sure this should need a saturating sub, maybe there's a logic error somewhere
        self.spirit_light_placements_remaining =
            self.spirit_light_placements_remaining.saturating_sub(1);
        let command = compile::spirit_light(amount.into(), &mut self.rng);
        self.place_with_simulation(pickup, command, placement_spoiler, |_, world, events| {
            world.add_spirit_light(amount, events)
        });
    }

    fn place_without_simulation(
        &mut self,
        pickup: &LocDataEntry,
        command: CommandVoid,
        placement_spoiler: &mut Vec<SpoilerPlacement>,
    ) {
        self.place_with_simulation(pickup, command, placement_spoiler, |_, _, _| {});
    }

    fn push_command_with_simulation<F>(
        &mut self,
        trigger: Trigger,
        command: CommandVoid,
        simulate: F,
    ) where
        F: FnOnce(&CommandVoid, &mut World<'graph, 'settings>, &[Event]),
    {
        simulate(&command, &mut self.world, &self.output.events);

        self.output.events.push(Event { trigger, command });
    }

    fn push_command(&mut self, trigger: Trigger, command: CommandVoid) {
        self.push_command_with_simulation(trigger, command, CommandVoid::simulate);
    }

    fn write_placement_spoiler(
        &mut self,
        pickup: &LocDataEntry,
        command: &CommandVoid,
        into: &mut Vec<SpoilerPlacement>,
    ) {
        let origin_world_index = self.index;

        into.push(SpoilerPlacement {
            origin_world_index,
            target_world_index: origin_world_index,
            location: NodeSummary::new(pickup),
            item: self.spoiler_item(command),
        });
    }

    fn spoiler_item(&mut self, command: &CommandVoid) -> SpoilerItem {
        SpoilerItem {
            command: command.clone(),
            name: self.log_name(command),
        }
    }
}

struct OrderingDistribution {
    rng: Pcg64Mcg,
    distribution: Uniform<i8>,
}

impl OrderingDistribution {
    fn new(rng: &mut Pcg64Mcg) -> Self {
        let rng = Pcg64Mcg::from_rng(rng).expect(SEED_FAILED_MESSAGE);
        let distribution = Uniform::new_inclusive(-1, 1);

        Self { rng, distribution }
    }

    fn sample(&mut self) -> Ordering {
        match self.distribution.sample(&mut self.rng) {
            -1 => Ordering::Less,
            0 => Ordering::Equal,
            1 => Ordering::Greater,
            _ => unreachable!(),
        }
    }
}

fn total_reach_check<'graph>(
    world: &mut World<'graph, '_>,
    log_index: &str,
    output: &IntermediateOutput,
    item_pool: &ItemPool,
) -> Vec<&'graph LocDataEntry> {
    world.snapshot();

    for command in &**item_pool {
        command.simulate(world, &output.events);
    }
    world.add_spirit_light(TOTAL_SPIRIT_LIGHT, &output.events);

    let spawn = world.spawn;
    world.spawn = world.graph.find_node(DEFAULT_SPAWN).unwrap();
    world.traverse_spawn(&output.events);

    let mut needs_placement = world.reached_pickups().collect::<Vec<_>>();
    let mut extra_slots = vec![];

    world.spawn = spawn;
    world.restore_snapshot();

    needs_placement.retain(|pickup| {
        let condition = CommandBoolean::loc_data_condition(pickup.uber_identifier, pickup.value);
        // TODO remove by identifier instead?
        if output.removed_locations.contains(&condition) {
            trace!(
                "{log_index}Manually removed {pickup} from placement locations",
                pickup = pickup.identifier
            );

            return false;
        }

        if world.loc_data_condition_met(pickup.uber_identifier, pickup.value) {
            trace!(
                "{log_index}Removing {pickup} from placement locations since the condition was met on spawn",
                pickup = pickup.identifier
            );

            return false;
        }

        match output.location_slots.get(&condition) {
            None | Some(1) => {},
            Some(0) => {
                trace!(
                    "{log_index}Removing {pickup} from placement locations since location slots were set to zero",
                    pickup = pickup.identifier
                );

                return false;
            }
            Some(slots) => {
                trace!(
                    "{log_index}Increasing {pickup} slots to {slots}",
                    pickup = pickup.identifier
                );

                extra_slots.extend(iter::repeat(pickup).take((slots - 1) as usize));
            }
        }

        true
    });

    needs_placement.append(&mut extra_slots);

    trace!(
        "{log_index}{amount} total locations that need placements: {needs_placement}",
        amount = needs_placement.len(),
        needs_placement = format_pickups(&needs_placement)
    );

    needs_placement
}

fn format_pickups<'a, 'graph>(
    pickups: &'a [&'graph LocDataEntry],
) -> impl Display + use<'a, 'graph> {
    pickups.iter().map(|pickup| &pickup.identifier).format(", ")
}
