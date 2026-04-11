use super::{
    item_pool::ItemPool, spirit_light::SpiritLightProvider, Seed, SeedUniverse, SEED_FAILED_MESSAGE,
};
use crate::{
    generator::solutions::{solution_weights, Solution, SolutionLike, SOLUTION_MAX_ITEMS},
    item_pool::ItemPoolBuilder,
    spoiler::{NodeSummary, SeedSpoiler, SpoilerGroup, SpoilerItem, SpoilerPlacement},
    World,
};
use itertools::Itertools;
use log::{log_enabled, trace, warn, Level::Trace};
use rand::{
    distributions::WeightedIndex,
    prelude::Distribution,
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use std::{cmp::Ordering, fmt::Display, mem, ops::RangeFrom, sync::LazyLock};
use wotw_seedgen_data::seed_language::{compile::store_boolean, output::AsConstant};
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
    Position, UberIdentifier, UniverseSettings,
};
use wotw_seedgen_seed::SeedgenInfo;

const KEYSTONE_DOORS: &[(&str, usize)] = &[
    ("MarshSpawn.KeystoneDoor", 2),
    ("HowlsDen.KeystoneDoor", 2),
    ("MarshPastOpher.EyestoneDoor", 2),
    ("MidnightBurrows.KeystoneDoor", 4),
    ("WoodsEntry.KeystoneDoor", 2),
    ("WoodsMain.KeystoneDoor", 4),
    ("LowerReach.KeystoneDoor", 4),
    ("UpperReach.KeystoneDoor", 4),
    ("UpperDepths.EntryKeystoneDoor", 2),
    ("UpperDepths.CentralKeystoneDoor", 2),
    ("UpperPools.KeystoneDoor", 4),
    ("UpperWastes.KeystoneDoor", 2),
];
pub(super) const SPAWN_SLOTS: usize = 7;
const UNSHARED_ITEMS: usize = 5; // How many items to place per world that are guaranteed not being sent to another world
const TOTAL_SPIRIT_LIGHT: i32 = 20000;

const MIN_PLACEHOLDERS: usize = 3;
static MAX_PLACEHOLDERS: LazyLock<usize> = LazyLock::new(|| SOLUTION_MAX_ITEMS.saturating_mul(2));

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

        let worlds = worlds
            .into_iter()
            .enumerate()
            .map(|(index, (world, output))| {
                WorldContext::new(rng, world, output, index, multiworld)
            })
            .collect::<Result<Vec<_>, _>>()?;

        let spawns = worlds
            .iter()
            .map(|world_context| {
                world_context.world.graph.nodes[world_context.world.spawn]
                    .identifier()
                    .to_string()
            })
            .collect();

        // TODO is this possible earlier to avoid the need to filter through nodes?
        // otherwise, it would at least be unnecessary if no world has door randomization
        let door_identifier_map = worlds[0]
            .world
            .graph
            .nodes
            .iter()
            .filter_map(Node::try_as_anchor_ref)
            .filter_map(|anchor| {
                anchor
                    .door
                    .as_ref()
                    .map(|door| (door.id, &anchor.identifier))
            })
            .collect::<FxHashMap<_, _>>();

        let doors = worlds
            .iter()
            .map(|world_context| {
                if world_context.world.settings.randomize_doors.is_some() {
                    let mut doors = (1..=32)
                        .map(|door_id| {
                            let target_door_id = world_context
                                .world
                                .fetch_integer(UberIdentifier::new(27, door_id));

                            (door_id, target_door_id)
                        })
                        .collect::<Vec<_>>();

                    doors.sort_by_key(|(_, target)| *target);

                    doors
                        .into_iter()
                        .map(|(from, to)| {
                            (
                                door_identifier_map[&from].clone(),
                                door_identifier_map[&to].clone(),
                            )
                        })
                        .collect()
                } else {
                    vec![]
                }
            })
            .collect();

        // TODO move some of the above logic into SeedSpoiler::new?
        let spoiler = SeedSpoiler::new(spawns, doors);

        Ok(Self {
            rng: Pcg64Mcg::from_rng(&mut *rng).expect(SEED_FAILED_MESSAGE),
            worlds,
            settings,
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

            let required_keystones = KEYSTONE_DOORS
                .iter()
                .filter_map(|(identifier, amount)| {
                    world_context
                        .world
                        .reached_nodes()
                        .filter_map(Node::try_as_state_ref)
                        .any(|state| &state.identifier == identifier)
                        .then_some(*amount)
                })
                .sum::<usize>();
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
            let mut spirit_light_placements_remaining =
                placements_remaining.saturating_sub(origin_world.item_pool.len());

            for pickup in needs_random_placement {
                any_placed = true; // TODO pull out of loop and skip some more calculations that way

                let origin_world = &mut self.worlds[origin_world_index];

                let should_place_spirit_light = !pickup.uber_identifier.is_shop()
                    && self.rng.gen_bool(
                        spirit_light_placements_remaining as f64 / placements_remaining as f64,
                    );

                let (target_world_index, command) = if should_place_spirit_light {
                    let batch = origin_world
                        .spirit_light_provider
                        .take(spirit_light_placements_remaining);

                    // Placements_remaining has reduced by one, item_pool.len() remained the same.
                    // If should_place_spirit_light is true, spirit_light_placements_remaining must be
                    // greater than one, so this branch doesn't need a saturating sub.
                    spirit_light_placements_remaining -= 1;

                    (
                        origin_world_index,
                        compile::spirit_light((batch as i32).into(), &mut self.rng),
                    )
                } else {
                    let target_world_index = self.choose_target_world_for_random_placement();
                    let target_world = &mut self.worlds[target_world_index];

                    let item = match target_world.item_pool.choose_random() {
                        None => {
                            // Since this is not taken from the item pool, placements_remaining
                            // has reduced by one and item_pool.len() remained the same.
                            spirit_light_placements_remaining =
                                spirit_light_placements_remaining.saturating_sub(1);

                            target_world.backup_gorlek_ore()
                        }
                        Some(item) => {
                            if origin_world_index != target_world_index {
                                // If the item is taken from another item pool, then placements_remaining
                                // has reduced by one and item_pool.len() remained the same.
                                // If it's taken from the own item pool, both have reduced by one
                                // and spirit_light_placements_remaining remains the same
                                spirit_light_placements_remaining =
                                    spirit_light_placements_remaining.saturating_sub(1);
                            }

                            item
                        }
                    };

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

        let mut world_indices = (0..self.worlds.len()).collect::<Vec<_>>();
        world_indices.sort_by_key(|index| self.worlds[*index].placements_remaining());

        for target_world_index in world_indices.into_iter().rev() {
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

        self.worlds[target_world_index].place_spirit_light(
            progression.spirit_light as usize, // TODO why is this not i32
            &mut self.spoiler.groups[self.step - 1].placements,
        )
    }

    fn force_place_command(
        &mut self,
        command: CommandVoid,
        target_world_index: usize,
        mark_forced: bool,
    ) {
        let origin_world_index = self.choose_origin_world_for_forced_placement(target_world_index);
        let origin_world = &mut self.worlds[origin_world_index];

        match origin_world.choose_location::<false>() {
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
                    None,
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
                .find(|index| !self.worlds[**index].reached_needs_placement.is_empty())
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
            pickup.position,
            command,
            origin_world_index,
            target_world_index,
        );
    }

    fn push_command(
        &mut self,
        trigger: Trigger,
        pickup_position: Option<Position>,
        mut command: CommandVoid,
        origin_world_index: usize,
        target_world_index: usize,
    ) {
        let pickup_position_command =
            pickup_position.map(
                |pickup_position| CommandVoid::QueuedMessageScopedPickupPosition {
                    x: pickup_position.x.into(),
                    y: pickup_position.y.into(),
                },
            );

        if origin_world_index == target_world_index {
            self.worlds[origin_world_index].push_command(
                trigger,
                match pickup_position_command {
                    None => command,
                    Some(pickup_position_command) => CommandVoid::Multi {
                        commands: vec![pickup_position_command, command],
                    },
                },
            );
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
                    commands: match pickup_position_command {
                        None => vec![message_command, store_command],
                        Some(pickup_position_command) => {
                            vec![pickup_position_command, message_command, store_command]
                        }
                    },
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
        UberIdentifier {
            group: 12,
            member: self.multiworld_state_index.next().unwrap(),
        }
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

    fn finish(self, loc_data: &LocData, debug: bool, rng: &mut Pcg64Mcg) -> SeedUniverse {
        SeedUniverse {
            worlds: self
                .worlds
                .into_iter()
                .map(|mut world_context| {
                    assert!(
                        world_context.output.icons.is_empty(),
                        "custom icons in seedgen aren't supported"
                    ); // TODO custom icons in snippets

                    let spawn = &world_context.world.graph.nodes[world_context.world.spawn];
                    world_context.output.spawn = Some(*spawn.position().unwrap());

                    let seedgen_info = SeedgenInfo {
                        universe_settings: self.settings.clone(),
                        world_index: world_context.index,
                        spawn_identifier: spawn.identifier().to_string(),
                    };

                    let string_placeholder_map = world_context.output.postprocess(loc_data, rng);

                    Seed::new(world_context.output, string_placeholder_map, debug)
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

        let needs_placement = total_reach_check(&mut world, &log_index, &output, &item_pool);

        world.traverse_spawn(&output.events);

        // TODO how should !add_item(spirit_light(100)) behave?
        let spirit_light_provider = SpiritLightProvider::new(TOTAL_SPIRIT_LIGHT, &mut rng);

        let mut world_context = Self {
            rng,
            world,
            output,
            index,
            log_index,
            item_pool,
            spirit_light_provider,
            needs_placement,
            placeholders: Default::default(),
            reached_needs_placement: Default::default(),
            received_placement: Default::default(),
            reached_item_locations: Default::default(),
            spawn_slots: SPAWN_SLOTS,
            unshared_items: UNSHARED_ITEMS,
        };

        world_context.generate_doors()?;

        Ok(world_context)
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
            }

            // We prefer generating indices over shuffling the nodes because usually there aren't many zone preplacements (relics)
            let pickup_index =
                pickup_indices.swap_remove(self.rng.gen_range(0..pickup_indices.len()));
            let pickup = self.needs_placement[pickup_index];

            self.place(pickup, command, preplacement_spoiler);
            self.received_placement.push(pickup_index);
        }
    }

    // TODO it looks like the simulated world has the 1 spirit light on spawn for some reason?
    fn hi_torin(&mut self, preplacement_spoiler: &mut Vec<SpoilerPlacement>) {
        // TODO implement From<{number}> for Constant commands?
        let command = compile::spirit_light(1.into(), &mut self.rng);

        if self.needs_placement.is_empty() {
            let name = self.log_name(&command);

            warn!(
                "{}Failed to preplace {name} since no free placement location was available",
                self.log_index
            );
        } else {
            let pickup = self
                .needs_placement
                .swap_remove(self.rng.gen_range(0..self.needs_placement.len()));

            self.place(pickup, command, preplacement_spoiler);
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
        received_placement.sort();

        for pickup_index in received_placement.into_iter().rev() {
            self.needs_placement.swap_remove(pickup_index);
        }
    }

    fn placements_remaining(&self) -> usize {
        self.needs_placement.len() - self.received_placement.len() + self.placeholders.len()
    }

    fn spirit_light_placements_remaining(&self) -> usize {
        self.placements_remaining()
            .saturating_sub(self.item_pool.len())
    }

    fn reserve_placeholders(&mut self) -> Vec<&'graph LocDataEntry> {
        self.received_placement
            .extend(self.reached_needs_placement.clone());

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

        trace!(
            "{log_index}Keeping {amount} placeholders: {placeholders}",
            log_index = self.log_index,
            amount = self.placeholders.len(),
            placeholders = format_pickups(&self.placeholders),
        );

        mem::take(&mut self.reached_needs_placement)
            .into_iter()
            .map(|index| self.needs_placement[index])
            .chain(released_placeholders)
            .collect()
    }

    fn progression_slots(&self) -> usize {
        self.reached_needs_placement.len() + self.placeholders.len() + self.spawn_slots
    }

    fn spirit_light_progression_slots(&self) -> usize {
        self.reached_needs_placement
            .iter()
            .map(|pickup_index| &self.needs_placement[*pickup_index])
            .chain(&self.placeholders)
            .filter(|pickup| !pickup.uber_identifier.is_shop())
            .count()
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

    fn place_spirit_light(
        &mut self,
        mut amount: usize,
        placement_spoiler: &mut Vec<SpoilerPlacement>,
    ) {
        while amount > 0 {
            let batch = self
                .spirit_light_provider
                .take(self.spirit_light_placements_remaining());

            amount = amount.saturating_sub(batch);
            let command = compile::spirit_light((batch as i32).into(), &mut self.rng);

            match self.choose_location::<true>() {
                None => {
                    warn!(
                        "Not enough space to place {name}, aborting progression",
                        name = self.log_name(&command)
                    );
                    break;
                }
                Some(pickup) => self.place(pickup, command, placement_spoiler),
            }
        }
    }

    fn choose_location<const SPIRIT_LIGHT: bool>(&mut self) -> Option<&'graph LocDataEntry> {
        if SPIRIT_LIGHT {
            self.reached_needs_placement
                .iter()
                .enumerate()
                .filter(|(_, pickup_index)| {
                    !self.needs_placement[**pickup_index]
                        .uber_identifier
                        .is_shop()
                })
                .map(|(index, _)| index)
                .choose(&mut self.rng) // TODO shuffle instead?
        } else {
            (!self.reached_needs_placement.is_empty())
                .then(|| self.rng.gen_range(0..self.reached_needs_placement.len()))
        }
        .map(|index| {
            let pickup_index = self.reached_needs_placement.swap_remove(index);
            self.received_placement.push(pickup_index);

            self.needs_placement[pickup_index]
        })
        .or_else(|| {
            if SPIRIT_LIGHT {
                let (index, _) = self
                    .placeholders
                    .iter()
                    .enumerate()
                    .find(|(_, pickup)| !pickup.uber_identifier.is_shop())?;

                Some(self.placeholders.swap_remove(index))
            } else {
                self.placeholders.pop()
            }
        })
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

        for (placements_remaining, pickup) in needs_placement.into_iter().enumerate().rev() {
            let is_shop = pickup.uber_identifier.is_shop();

            let command = if is_shop {
                self.backup_gorlek_ore()
            } else {
                let amount = self.spirit_light_provider.take(1 + placements_remaining) as i32;
                compile::spirit_light(amount.into(), &mut self.rng)
            };
            self.place(pickup, command, placement_spoiler);
        }
        // TODO unreachable items that should be filled
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

    fn place(
        &mut self,
        pickup: &LocDataEntry,
        command: CommandVoid,
        placement_spoiler: &mut Vec<SpoilerPlacement>,
    ) {
        trace!(
            "{index}Placing {name} at {pickup}",
            name = self.log_name(&command),
            index = self.log_index,
            pickup = pickup.identifier,
        );

        self.write_placement_spoiler(pickup, &command, placement_spoiler);

        self.push_command(
            Trigger::loc_data_trigger(pickup.uber_identifier, pickup.value),
            command,
        );
    }

    fn push_command(&mut self, trigger: Trigger, command: CommandVoid) {
        // TODO not sure what this did and why
        // self.world.uber_states.register_trigger(&trigger);

        // TODO many paths leading here can do the simulation more efficiently
        self.world.simulate(&command, &self.output.events);

        self.output.events.push(Event { trigger, command });
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

    world.traverse_spawn(&output.events);

    let mut needs_placement = world.reached_pickups().collect::<Vec<_>>();

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

        true
    });

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
