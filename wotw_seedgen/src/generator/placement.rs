use super::{
    item_pool::ItemPool, spirit_light::SpiritLightProvider, weight::weight, Seed, SeedUniverse,
    SEED_FAILED_MESSAGE,
};
use crate::{
    contained_uber_identifiers::ContainedWrites,
    spoiler::{NodeSummary, SeedSpoiler, SpoilerGroup, SpoilerItem, SpoilerPlacement},
    world::{node_condition, node_trigger, UberStateValue},
    World,
};
use itertools::Itertools;
use log::{log_enabled, trace, warn, Level::Trace};
use ordered_float::OrderedFloat;
use rand::{
    distributions::{Uniform, WeightedIndex},
    prelude::Distribution,
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use std::{cmp::Ordering, iter, mem, ops::RangeFrom};
use wotw_seedgen_data::{CommonUberIdentifier, Skill, UberIdentifier};
use wotw_seedgen_logic_language::output::Node;
use wotw_seedgen_seed::SeedgenInfo;
use wotw_seedgen_seed_language::{
    compile,
    output::{
        ClientEvent, CommandString, CommandVoid, Event, IntermediateOutput, ItemMetadata,
        StringOrPlaceholder, Trigger,
    },
};
use wotw_seedgen_settings::UniverseSettings;

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
pub(super) const PREFERRED_SPAWN_SLOTS: usize = 3;
const _: usize = SPAWN_SLOTS - PREFERRED_SPAWN_SLOTS; // check that SPAWN_SLOTS >= PREFERRED_SPAWN_SLOTS
const UNSHARED_ITEMS: usize = 5; // How many items to place per world that are guaranteed not being sent to another world
const TOTAL_SPIRIT_LIGHT: i32 = 20000;

pub fn generate_placements(
    rng: &mut Pcg64Mcg,
    worlds: Vec<(World, IntermediateOutput)>,
    settings: &UniverseSettings,
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

    Ok(context.finish(debug))
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
    /// all remaining nodes which need to be assigned random placements
    // TODO store indices instead?
    needs_placement: Vec<&'graph Node>,
    /// nodes which have been reached but explicitely haven't been asigned a placement yet to leave space for later progressions
    placeholders: Vec<&'graph Node>,
    /// indices into `needs_placement` for nodes that are reachable and may be used for placements in this step
    reached_needs_placement: Vec<usize>,
    /// indices into `needs_placement` for nodes that have received a placement and should be removed before the next placement step
    received_placement: Vec<usize>,
    /// number of nodes in `reached` that can give items
    reached_item_locations: usize,
    /// number of remaining allowed placements on spawn
    spawn_slots: usize,
    // TODO is this still needed for multiworld quality?
    /// number of remaining placements that should not be placed outside of the own world
    unshared_items: usize,
    /// generates random factors to modify shop prices with
    price_distribution: Uniform<f32>,
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
            .filter_map(|node| {
                node.get_anchor().and_then(|anchor| {
                    anchor
                        .door
                        .as_ref()
                        .map(|door| (door.id, &anchor.identifier))
                })
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
                                .uber_states
                                .get(UberIdentifier::new(27, door_id))
                                .as_integer();

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
                    .map(|index| NodeSummary::new(&world_context.needs_placement[*index]))
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
                        .any(|node| node.identifier() == *identifier)
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

            let spoiler_item = self.spoiler_item(0, &keystone);
            self.spoiler.groups[self.step - 1]
                .forced_items
                .extend(iter::repeat_with(|| spoiler_item.clone()).take(missing_keystones));

            for _ in 0..missing_keystones {
                let command = self.worlds[world_index].item_pool.remove_command(&keystone).unwrap_or_else(|| {
                    warn!("Not enough keystones in the item pool for forced keystone progression, placing anyway");
                    keystone.clone()
                });
                self.force_place_command(command, world_index);
            }
        }

        new_progressions
    }

    fn place_remaining(&mut self) {
        trace!("All locations reached. Placing remaining items");
        for target_world_index in 0..self.worlds.len() {
            let items = mem::take(&mut *self.worlds[target_world_index].item_pool);
            for command in items {
                self.force_place_command(command, target_world_index);
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
            for node in needs_random_placement {
                any_placed = true; // TODO pull out of loop and skip some more calculations that way
                let origin_world = &mut self.worlds[origin_world_index];
                let should_place_spirit_light = !node.uber_identifier().unwrap().is_shop()
                    && self.rng.gen_bool(
                        spirit_light_placements_remaining as f64 / placements_remaining as f64,
                    );

                let (target_world_index, command) = if should_place_spirit_light {
                    let batch = origin_world
                        .spirit_light_provider
                        .take(spirit_light_placements_remaining);
                    (
                        origin_world_index,
                        compile::spirit_light((batch as i32).into(), &mut self.rng),
                    )
                } else {
                    let target_world_index = self.choose_target_world_for_random_placement();
                    (
                        target_world_index,
                        self.worlds[target_world_index]
                            .item_pool
                            .choose_random()
                            .unwrap(),
                    )
                };

                let name = self.name(&command, origin_world_index, target_world_index);
                self.place_command_at(command, name, node, origin_world_index, target_world_index);

                placements_remaining -= 1;
                spirit_light_placements_remaining =
                    spirit_light_placements_remaining.saturating_sub(1);
            }
        }
        any_placed
    }

    fn choose_progression(&mut self) -> Result<Option<(usize, Progression)>, String> {
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
                    identifiers = world_context
                        .needs_placement
                        .iter()
                        .map(|node| node.identifier())
                        .format(", "),
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

    fn place_forced(&mut self, target_world_index: usize, progression: Progression) {
        match progression {
            Progression::ItemPool(items) => {
                for index in items.into_iter().rev() {
                    let command = self.worlds[target_world_index].item_pool.swap_remove(index);
                    self.force_place_command(command, target_world_index);
                }
            }
            Progression::SpiritLight(amount) => self.worlds[target_world_index]
                .place_spirit_light(amount, &mut self.spoiler.groups[self.step - 1].placements),
        }
    }

    fn force_place_command(&mut self, command: CommandVoid, target_world_index: usize) {
        let origin_world_index = self.choose_origin_world_for_forced_placement(target_world_index);
        let name = self.name(&command, origin_world_index, target_world_index);
        let origin_world = &mut self.worlds[origin_world_index];
        match origin_world.choose_placement_node::<false>() {
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
                );
                self.push_command(
                    Trigger::ClientEvent(ClientEvent::Spawn),
                    command,
                    name,
                    origin_world_index,
                    target_world_index,
                );
            }
            Some(node) => {
                self.place_command_at(command, name, node, origin_world_index, target_world_index);
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

    fn name(
        &self,
        command: &CommandVoid,
        origin_world_index: usize,
        target_world_index: usize,
    ) -> CommandString {
        let name = self.worlds[target_world_index].name(command);
        if origin_world_index == target_world_index {
            name
        } else {
            let right = match name.as_constant() {
                Some(value) => format!("'s {value}").into(),
                _ => CommandString::Concatenate {
                    left: Box::new("'s".into()),
                    right: Box::new(name),
                },
            };

            CommandString::Concatenate {
                left: Box::new(CommandString::WorldName {
                    index: target_world_index,
                }),
                right: Box::new(right),
            }
        }
    }

    fn place_command_at(
        &mut self,
        command: CommandVoid,
        name: CommandString,
        node: &Node,
        origin_world_index: usize,
        target_world_index: usize,
    ) {
        trace!(
            "Placing {target_index}{log_name} at {origin_index}{node}",
            log_name = self.worlds[target_world_index].log_name(&command),
            target_index = self.worlds[target_world_index].log_index,
            origin_index = self.worlds[origin_world_index].log_index,
            node = node.identifier()
        );

        // TODO spoiler icons for snippet-placed items
        // TODO spoiler icons for plandos?
        self.worlds[origin_world_index].map_icon(node, &command, name.clone());

        let uber_identifier = node.uber_identifier().unwrap();
        if uber_identifier.is_shop() {
            self.worlds[origin_world_index].shop_item_data(&command, uber_identifier, name.clone());
        }

        self.write_placement_spoiler(
            origin_world_index,
            target_world_index,
            NodeSummary::new(node),
            &command,
        );
        self.push_command(
            node_trigger(node).unwrap(),
            command,
            name,
            origin_world_index,
            target_world_index,
        );
    }

    fn push_command(
        &mut self,
        trigger: Trigger,
        command: CommandVoid,
        name: CommandString,
        origin_world_index: usize,
        target_world_index: usize,
    ) {
        if origin_world_index == target_world_index {
            self.worlds[origin_world_index].push_command(trigger, command);
        } else {
            let uber_identifier = self.multiworld_state();
            self.worlds[origin_world_index].push_command(
                trigger,
                CommandVoid::Multi {
                    commands: vec![
                        CommandVoid::QueuedMessage {
                            id: None,
                            priority: false,
                            message: name,
                            timeout: None,
                        },
                        compile::set_boolean_value(uber_identifier, true),
                    ],
                },
            );
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
    ) {
        let placement = SpoilerPlacement {
            origin_world_index,
            target_world_index,
            location,
            item: self.spoiler_item(target_world_index, command),
        };

        self.spoiler.groups[self.step - 1]
            .placements
            .push(placement);
    }

    fn spoiler_item(&mut self, target_world_index: usize, command: &CommandVoid) -> SpoilerItem {
        SpoilerItem {
            command: command.clone(),
            name: self.worlds[target_world_index].log_name(command),
        }
    }

    fn finish(mut self, debug: bool) -> SeedUniverse {
        self.resolve_placeholders();

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
                    Seed::new(world_context.output, debug).with_seedgen_info(seedgen_info)
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

        let mut item_pool = ItemPool::new(&mut rng);

        for (command, amount) in mem::take(&mut output.item_pool_changes) {
            if amount >= 0 {
                for _ in 0..amount {
                    item_pool.push(command.clone());
                }
            } else {
                for _ in 0..-amount {
                    item_pool.remove_command(&command);
                }
            }
        }

        item_pool.shuffle(&mut rng);

        world.simulate(&ClientEvent::Spawn, &output.events);
        world.simulate(&ClientEvent::Reload, &output.events);

        let needs_placement = total_reach_check(&mut world, &log_index, &output.events, &item_pool);

        world.traverse_spawn(&output.events);

        // TODO how should !add(spirit_light(100)) behave?
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
            price_distribution: Uniform::new_inclusive(0.75, 1.25),
        };

        world_context.generate_doors()?;

        Ok(world_context)
    }

    fn preplacements(&mut self, preplacement_spoiler: &mut Vec<SpoilerPlacement>) {
        trace!("{}Generating preplacements", self.log_index);

        self.hi_sigma(preplacement_spoiler);

        let mut zone_needs_placement = FxHashMap::default();
        for (command, zone) in mem::take(&mut self.output.preplacements) {
            let nodes = zone_needs_placement.entry(zone).or_insert_with(|| {
                self.needs_placement
                    .iter()
                    .enumerate()
                    .filter(|(_, node)| node.zone() == Some(zone))
                    .map(|(index, _)| index)
                    .collect::<Vec<_>>()
            });
            if nodes.is_empty() {
                let name = self.log_name(&command);
                warn!(
                    "{}Failed to preplace {name} in {zone} since no free placement location was available",
                    self.log_index
                );
            }
            // We prefer generating indices over shuffling the nodes because usually there aren't many zone preplacements (relics)
            let node_index = nodes.swap_remove(self.rng.gen_range(0..nodes.len()));
            let node = self.needs_placement[node_index];
            self.place(node, command, preplacement_spoiler);
            self.received_placement.push(node_index);
        }
    }

    // TODO name change
    fn hi_sigma(&mut self, preplacement_spoiler: &mut Vec<SpoilerPlacement>) {
        // TODO implement From<{number}> for Constant commands?
        let command = compile::spirit_light(1.into(), &mut self.rng);
        if self.needs_placement.is_empty() {
            let name = self.log_name(&command);
            warn!(
                "{}Failed to preplace {name} since no free placement location was available",
                self.log_index
            );
        } else {
            let node = self
                .needs_placement
                .swap_remove(self.rng.gen_range(0..self.needs_placement.len()));
            self.place(node, command, preplacement_spoiler);
        }
    }

    fn update_reached(&mut self) {
        self.update_needs_placement();

        self.reached_needs_placement = self
            .needs_placement
            .iter()
            .enumerate()
            .filter(|(_, node)| self.world.reached_nodes().contains(**node))
            .map(|(index, _)| index)
            .collect();
        self.reached_item_locations = self
            .world
            .reached_nodes()
            .filter(|node| node.can_place())
            .count();
        trace!(
            "{log_index}{amount} reached locations that need placements: {reached_needs_placement}",
            log_index = self.log_index,
            amount = self.reached_needs_placement.len(),
            reached_needs_placement = self
                .reached_needs_placement
                .iter()
                .map(|index| self.needs_placement[*index].identifier())
                .format(", ")
        );
    }

    fn update_needs_placement(&mut self) {
        let mut received_placement = mem::take(&mut self.received_placement);
        received_placement.sort();
        for node_index in received_placement.into_iter().rev() {
            self.needs_placement.swap_remove(node_index);
        }
    }

    fn placements_remaining(&self) -> usize {
        self.needs_placement.len() - self.received_placement.len() + self.placeholders.len()
    }

    fn spirit_light_placements_remaining(&self) -> usize {
        self.placements_remaining()
            .saturating_sub(self.item_pool.len())
    }

    fn reserve_placeholders(&mut self) -> Vec<&'graph Node> {
        self.received_placement
            .extend(self.reached_needs_placement.clone());
        let desired_placeholders = usize::max(
            usize::max(3, self.placeholders.len()),
            (self.reached_needs_placement.len() + self.placeholders.len()) / 2,
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
            placeholders = self
                .placeholders
                .iter()
                .map(|node| node.identifier())
                .format(", ")
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
            .map(|node_index| &self.needs_placement[*node_index])
            .chain(&self.placeholders)
            .filter(|node| !node.uber_identifier().unwrap().is_shop())
            .count()
    }

    fn choose_progression(&mut self, slots: usize) -> Option<Progression> {
        trace!("{}Attempting forced progression", self.log_index);

        let mut progressions = self.find_progressions(slots);

        // TODO filter redundancies?

        if log_enabled!(Trace) {
            self.log_weights(&progressions);
        }

        let weighted_index =
            WeightedIndex::new(progressions.iter().map(|progression| progression.weight)).ok()?;
        let weighted_progression = progressions.swap_remove(weighted_index.sample(&mut self.rng));

        Some(weighted_progression.items)
    }

    fn find_progressions(&mut self, slots: usize) -> Vec<WeightedProgression> {
        let spirit_light_slots = self.spirit_light_progression_slots();

        self.world
            .reach
            .uber_state_progressions
            .keys()
            .copied()
            .collect::<Vec<_>>()
            .into_iter()
            .filter_map(|uber_identifier| {
                self.find_progression(uber_identifier, slots, spirit_light_slots)
            })
            .collect()
    }

    fn find_progression(
        &mut self,
        uber_identifier: UberIdentifier,
        slots: usize,
        spirit_light_slots: usize,
    ) -> Option<WeightedProgression> {
        let value = self.world.uber_states.get(uber_identifier);

        match value {
            UberStateValue::Boolean(_) => self.find_boolean_progression(uber_identifier, slots),
            UberStateValue::Integer(_) => {
                self.find_integer_progression(uber_identifier, slots, spirit_light_slots)
            }
            UberStateValue::Float(_) => {
                warn!("Attempted to find progression for float UberState {uber_identifier}");
                None
            }
        }
    }

    fn find_boolean_progression(
        &mut self,
        uber_identifier: UberIdentifier,
        slots: usize,
    ) -> Option<WeightedProgression> {
        let reached = self.world.reached_len();

        self.item_pool
            .progression_for(uber_identifier)
            .next()
            .map(|(item_pool_index, item)| {
                let progression = Progression::ItemPool(vec![item_pool_index]);

                self.world.snapshot();

                self.world.simulate(item, &self.output.events);

                let new_reached = self.world.reached_len().saturating_sub(reached);
                let weight = weight(new_reached, uber_identifier, 1, 1, slots);

                self.world.restore_snapshot();

                WeightedProgression {
                    items: progression,
                    weight,
                }
            })
    }

    fn find_integer_progression(
        &mut self,
        uber_identifier: UberIdentifier,
        slots: usize,
        spirit_light_slots: usize,
    ) -> Option<WeightedProgression> {
        if uber_identifier == UberIdentifier::SPIRIT_LIGHT {
            self.find_spirit_light_progression(spirit_light_slots)
        } else {
            let initial_reached = self.world.reached_len();
            let mut reached = initial_reached;

            self.world.snapshot();

            let mut items = vec![];

            for (item_pool_index, item) in
                self.item_pool.progression_for(uber_identifier).take(slots)
            {
                self.world.simulate(item, &self.output.events);
                items.push(item_pool_index);

                reached = self.world.reached_len();
                if reached > initial_reached {
                    break;
                }
            }

            self.world.restore_snapshot();

            if items.is_empty() {
                return None;
            }

            let new_reached = reached - initial_reached;
            let weight = weight(
                new_reached,
                uber_identifier,
                items.len(),
                items.len(),
                slots,
            );

            Some(WeightedProgression {
                items: Progression::ItemPool(items),
                weight,
            })
        }
    }

    fn find_spirit_light_progression(&mut self, slots: usize) -> Option<WeightedProgression> {
        let initial_reached = self.world.reached_len();
        let mut reached = 0;

        self.world.snapshot();

        let mut amount = 0;

        while amount < self.spirit_light_provider.amount as usize {
            self.world.modify_spirit_light(1, &self.output.events);
            amount += 1;

            reached = self.world.reached_len();

            if reached > initial_reached {
                break;
            }
        }

        self.world.restore_snapshot();

        if amount == 0 {
            return None;
        }

        let new_reached = reached - initial_reached;
        let weight = weight(
            new_reached,
            UberIdentifier::SPIRIT_LIGHT,
            amount,
            amount / 50,
            slots,
        );

        Some(WeightedProgression {
            items: Progression::SpiritLight(amount),
            weight,
        })
    }

    fn log_weights(&mut self, progressions: &[WeightedProgression]) {
        // seedgen output should remain the same whether logging is enabled or not, so we sort a clone
        let mut sorted_progressions = progressions.to_vec();
        sorted_progressions.sort_unstable_by_key(|progression| OrderedFloat(progression.weight));

        let log_index = self.log_index.clone();

        let amount = sorted_progressions.len();

        let total_weight = sorted_progressions
            .iter()
            .map(|progression| progression.weight)
            .sum::<f32>();

        let progressions = sorted_progressions
            .into_iter()
            .format_with("\n", |progression, f| {
                let chance = (progression.weight / total_weight) * 100.;

                f(&format_args!("- {chance:.1}%: "))?;

                match progression.items {
                    Progression::ItemPool(items) => f(&format_args!(
                        "{}",
                        items
                            .into_iter()
                            .map(|index| self.log_name(&self.item_pool[index].clone()))
                            .format(", ")
                    )),
                    Progression::SpiritLight(amount) => f(&format_args!("{amount} Spirit Light")),
                }
            });

        trace!("{log_index}{amount} options for forced progression:\n{progressions}");
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

            match self.choose_placement_node::<true>() {
                None => {
                    warn!(
                        "Not enough space to place {name}, aborting progression",
                        name = self.log_name(&command)
                    );
                    break;
                }
                Some(node) => self.place(node, command, placement_spoiler),
            }
        }
    }

    fn choose_placement_node<const SPIRIT_LIGHT: bool>(&mut self) -> Option<&'graph Node> {
        if SPIRIT_LIGHT {
            self.reached_needs_placement
                .iter()
                .enumerate()
                .filter(|(_, node_index)| {
                    !self.needs_placement[**node_index]
                        .uber_identifier()
                        .unwrap()
                        .is_shop()
                })
                .map(|(index, _)| index)
                .choose(&mut self.rng) // TODO shuffle instead?
        } else {
            (!self.reached_needs_placement.is_empty())
                .then(|| self.rng.gen_range(0..self.reached_needs_placement.len()))
        }
        .map(|index| {
            let node_index = self.reached_needs_placement.swap_remove(index);
            self.received_placement.push(node_index);
            self.needs_placement[node_index]
        })
        .or_else(|| {
            if SPIRIT_LIGHT {
                let (index, _) = self
                    .placeholders
                    .iter()
                    .enumerate()
                    .find(|(_, node)| !node.uber_identifier().unwrap().is_shop())?;
                Some(self.placeholders.swap_remove(index))
            } else {
                self.placeholders.pop()
            }
        })
    }

    fn map_icon(&mut self, node: &Node, command: &CommandVoid, label: CommandString) {
        let icon = self
            .output
            .item_metadata
            .map_icon(command)
            .unwrap_or_else(|| {
                command
                    .contained_common_write_identifiers()
                    .into_iter()
                    .next()
                    .map(CommonUberIdentifier::map_icon)
                    .unwrap_or_default()
            });

        self.on_load(CommandVoid::SetSpoilerMapIcon {
            location: node.identifier().to_string(),
            icon,
            label,
        });
    }

    pub fn name(&self, command: &CommandVoid) -> CommandString {
        command_name(command, &self.output.item_metadata)
    }

    fn log_name(&mut self, command: &CommandVoid) -> String {
        self.output
            .item_metadata
            .name(command)
            .map(|s| match s {
                StringOrPlaceholder::Value(value) => strip_control_characters(&value),
                other => other.to_string(),
            })
            .or_else(|| {
                self.simulate_message(command)
                    .map(|message| strip_control_characters(&message))
            })
            .unwrap_or_else(|| {
                let value = command.to_string();
                warn!("No name specified for custom command: {value}");
                value
            })
    }

    fn simulate_message(&mut self, command: &CommandVoid) -> Option<String> {
        find_message(command).map(|message| self.world.simulate(message, &self.output.events))
    }

    fn on_load(&mut self, command: CommandVoid) {
        self.push_command(Trigger::ClientEvent(ClientEvent::Reload), command);
    }

    fn shop_item_data(
        &mut self,
        command: &CommandVoid,
        uber_identifier: UberIdentifier,
        name: CommandString,
    ) {
        let (price, description, icon) = self.output.item_metadata.shop_data(command);

        let price = price.unwrap_or_else(|| self.shop_price(command).into());
        let icon = icon.or_else(|| {
            command
                .contained_common_write_identifiers()
                .next()
                .and_then(CommonUberIdentifier::icon)
        });

        let mut commands = vec![
            CommandVoid::SetShopItemPrice {
                uber_identifier,
                price,
            },
            CommandVoid::SetShopItemName {
                uber_identifier,
                name,
            },
        ];
        if let Some(description) = description {
            commands.push(CommandVoid::SetShopItemDescription {
                uber_identifier,
                description,
            })
        }
        if let Some(icon) = icon {
            commands.push(CommandVoid::SetShopItemIcon {
                uber_identifier,
                icon,
            })
        }

        self.on_load(CommandVoid::Multi { commands });
    }

    fn shop_price(&mut self, command: &CommandVoid) -> i32 {
        let mut price = command
            .contained_common_write_identifiers()
            .map(CommonUberIdentifier::shop_price)
            .sum::<f32>();

        if price == 0. {
            price = 200.
        }
        if price != CommonUberIdentifier::Skill(Skill::Blaze).shop_price() {
            price *= self.price_distribution.sample(&mut self.rng);
        }

        price.round() as i32
    }

    fn fill_remaining(&mut self, placement_spoiler: &mut Vec<SpoilerPlacement>) {
        trace!(
            "{}Filling remaining locations with spirit light",
            self.log_index
        );

        let mut needs_placement = mem::take(&mut self.needs_placement);
        needs_placement.extend(mem::take(&mut self.placeholders));
        needs_placement.shuffle(&mut self.rng);

        for (placements_remaining, node) in needs_placement.into_iter().enumerate().rev() {
            let uber_identifier = node.uber_identifier().unwrap();
            let is_shop = uber_identifier.is_shop();
            let command = if is_shop {
                // TODO try to avoid
                let command = compile::gorlek_ore();
                warn!(
                    "{index}Placing more {name} than intended to avoid placing Spirit Light in a shop",
                    name = self.log_name(&command),
                    index = self.log_index,
                );
                command
            } else {
                let amount = self.spirit_light_provider.take(1 + placements_remaining) as i32;
                compile::spirit_light(amount.into(), &mut self.rng)
            };
            self.place(node, command, placement_spoiler);
        }
        // TODO unreachable items that should be filled
    }

    fn place(
        &mut self,
        node: &Node,
        command: CommandVoid,
        placement_spoiler: &mut Vec<SpoilerPlacement>,
    ) {
        trace!(
            "{index}Placing {name} at {node}",
            name = self.log_name(&command),
            index = self.log_index,
            node = node.identifier()
        );
        let uber_identifier = node.uber_identifier().unwrap();
        if uber_identifier.is_shop() {
            self.shop_item_data(&command, uber_identifier, self.name(&command))
        }
        self.write_placement_spoiler(node, &command, placement_spoiler);
        self.push_command(node_trigger(node).unwrap(), command);
    }

    fn push_command(&mut self, trigger: Trigger, command: CommandVoid) {
        self.world.uber_states.register_trigger(&trigger); // TODO unnecessary?
        self.world.simulate(&command, &self.output.events);
        self.output.events.push(Event { trigger, command });
    }

    fn write_placement_spoiler(
        &mut self,
        node: &Node,
        command: &CommandVoid,
        into: &mut Vec<SpoilerPlacement>,
    ) {
        let origin_world_index = self.index;
        into.push(SpoilerPlacement {
            origin_world_index,
            target_world_index: origin_world_index,
            location: NodeSummary::new(node),
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

#[derive(Clone)]
struct WeightedProgression {
    items: Progression,
    weight: f32,
}

#[derive(Clone)]
enum Progression {
    // TODO smallvec
    ItemPool(Vec<usize>),
    SpiritLight(usize),
}

fn total_reach_check<'graph>(
    world: &mut World<'graph, '_>,
    log_index: &str,
    events: &[Event],
    item_pool: &ItemPool,
) -> Vec<&'graph Node> {
    let mut complete_world = world.clone();
    for command in &**item_pool {
        complete_world.simulate(command, events);
    }
    complete_world.modify_spirit_light(TOTAL_SPIRIT_LIGHT, events);
    complete_world.traverse_spawn(events);

    let needs_placement = complete_world
        .reached_indices()
        .map(|index| &world.graph.nodes[index])
        .filter(|node| {
            node.can_place() && {
                let condition = node_condition(node).unwrap();
                if world.simulate(&condition, events) {
                    trace!("Removing {node} from placement locations since the condition was met on spawn", node = node.identifier());
                    return false;
                }

                // TODO maybe optimize based on shape of events, many of which can't possibly be loc_data events
                if events.iter().any(|event|
                    matches!(&event.trigger, Trigger::Condition(trigger) if trigger == &condition)
                ) {
                    trace!("Removing {node} from placement locations since an item was preplaced", node = node.identifier());
                    return false;
                }

                true
            }
        })
        .collect::<Vec<_>>();

    trace!(
        "{log_index}{amount} total locations that need placements: {needs_placement}",
        amount = needs_placement.len(),
        needs_placement = needs_placement
            .iter()
            .copied()
            .map(Node::identifier)
            .format(", ")
    );

    needs_placement
}

pub fn command_name(command: &CommandVoid, item_metadata: &ItemMetadata) -> CommandString {
    item_metadata
        .name(command)
        .map(CommandString::from)
        .or_else(|| find_message(command).cloned())
        .unwrap_or_else(|| {
            let value = command.to_string();
            warn!("No name specified for custom command: {value}");
            value.into()
        })
}
fn find_message(command: &CommandVoid) -> Option<&CommandString> {
    match command {
        CommandVoid::Multi { commands } => commands.iter().find_map(find_message),
        CommandVoid::QueuedMessage { message, .. } => Some(message),
        _ => None,
    }
}
fn strip_control_characters(s: &str) -> String {
    let mut result = String::new();
    let mut last_end = 0;
    let mut in_tag = false;

    for (index, byte) in s.as_bytes().iter().enumerate() {
        match (in_tag, byte) {
            (_, b'@' | b'#' | b'$' | b'*') => {
                result.push_str(&s[last_end..index]);
                last_end = index + 1;
            }
            (false, b'<') => {
                result.push_str(&s[last_end..index]);
                in_tag = true;
            }
            (true, b'>') => {
                last_end = index + 1;
                in_tag = false;
            }
            _ => {}
        }
    }
    result.push_str(&s[last_end..]);

    result
}

#[cfg(test)]
mod tests {
    use super::strip_control_characters as strip;

    #[test]
    fn strip_control_characters() {
        assert_eq!(strip(""), "");
        assert_eq!(strip("aaa"), "aaa");
        assert_eq!(strip("@#$"), "");
        assert_eq!(strip("@@@a@a@@a@"), "aaa");
        assert_eq!(strip("a<aaa>a</><aaaaa>a"), "aaa");
    }
}
