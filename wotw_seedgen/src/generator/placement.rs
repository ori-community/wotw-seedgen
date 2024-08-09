use super::{
    cost::Cost, item_pool::ItemPool, spirit_light::SpiritLightProvider, Seed, SeedUniverse,
    SEED_FAILED_MESSAGE,
};
use crate::{
    common_item::CommonItem,
    filter_redundancies,
    inventory::Inventory,
    log::{trace, warning},
    node_condition, node_trigger,
    orbs::OrbVariants,
    spoiler::{NodeSummary, SeedSpoiler, SpoilerGroup, SpoilerPlacement},
    World,
};
use itertools::Itertools;
#[cfg(any(feature = "log", test))]
use ordered_float::OrderedFloat;
use rand::{
    distributions::Uniform,
    prelude::Distribution,
    seq::{IteratorRandom, SliceRandom},
    Rng, SeedableRng,
};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use std::{cmp::Ordering, iter, mem, ops::RangeFrom};
use wotw_seedgen_data::{Equipment, MapIcon, OpherIcon, Skill, UberIdentifier, WeaponUpgrade};
use wotw_seedgen_logic_language::output::{Node, Requirement};
use wotw_seedgen_seed_language::{
    compile,
    output::{
        ClientEvent, CommandInteger, CommandString, CommandVoid, Event, Icon, IntermediateOutput,
        ItemMetadata, StringOrPlaceholder, Trigger,
    },
};

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
const SPAWN_SLOTS: usize = 7;
const PREFERRED_SPAWN_SLOTS: usize = 3;
const _: usize = SPAWN_SLOTS - PREFERRED_SPAWN_SLOTS; // check that SPAWN_SLOTS >= PREFERRED_SPAWN_SLOTS
const UNSHARED_ITEMS: usize = 5; // How many items to place per world that are guaranteed not being sent to another world
const TOTAL_SPIRIT_LIGHT: i32 = 20000;

pub fn generate_placements(
    rng: &mut Pcg64Mcg,
    worlds: Vec<(World, IntermediateOutput)>,
    debug: bool,
) -> Result<SeedUniverse, String> {
    assert!(
        !worlds.is_empty(),
        "Need at least one world to generate a seed"
    );
    let mut context = Context::new(rng, worlds);

    context.preplacements();

    loop {
        context.next_step();
        context.update_reached();
        if context.is_everything_reached() {
            context.place_remaining();
            context.sort_spoiler_placements();
            break;
        }
        context.force_keystones();
        if !context.place_random() {
            if let Some((target_world_index, progression)) = context.choose_progression()? {
                context.place_forced(target_world_index, progression);
            }
        }
    }

    Ok(context.finish(debug))
}

pub struct Context<'graph, 'settings> {
    rng: Pcg64Mcg,
    pub worlds: Vec<WorldContext<'graph, 'settings>>,
    /// next multiworld uberState id to use
    multiworld_state_index: RangeFrom<i32>,
    /// current placement step
    step: usize,
    /// spoiler being populated over the course of generation
    spoiler: SeedSpoiler,
}
pub struct WorldContext<'graph, 'settings> {
    rng: Pcg64Mcg,
    pub world: World<'graph, 'settings>,
    pub output: IntermediateOutput,
    /// world index of this world
    #[cfg_attr(not(any(feature = "log", test)), allow(unused))]
    index: usize,
    /// remaining items to place
    item_pool: ItemPool,
    /// generates appropriate spirit light amounts
    spirit_light_provider: SpiritLightProvider,
    /// all remaining nodes which need to be assigned random placements
    needs_placement: Vec<&'graph Node>,
    /// nodes which have been reached but explicitely haven't been asigned a placement yet to leave space for later progressions
    placeholders: Vec<&'graph Node>,
    /// reached nodes at the start of the current placement step
    reached: Vec<&'graph Node>,
    /// unmet requirements at the start of the current placement step
    progressions: Vec<(&'graph Requirement, OrbVariants)>,
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
    ) -> Self {
        let worlds = worlds
            .into_iter()
            .enumerate()
            .map(|(index, (world, output))| WorldContext::new(rng, world, output, index))
            .collect::<Vec<_>>();
        let spawns = worlds
            .iter()
            .map(|world_context| {
                world_context.world.graph.nodes[world_context.world.spawn]
                    .identifier()
                    .to_string()
            })
            .collect();
        let spoiler = SeedSpoiler::new(spawns);
        Self {
            rng: Pcg64Mcg::from_rng(&mut *rng).expect(SEED_FAILED_MESSAGE),
            worlds,
            multiworld_state_index: 0..,
            step: 0,
            spoiler,
        }
    }

    fn preplacements(&mut self) {
        for world_context in &mut self.worlds {
            world_context.preplacements(&mut self.spoiler.preplacements);
        }
    }

    fn next_step(&mut self) {
        self.sort_spoiler_placements();
        self.step += 1;
        trace!("Placement step #{}", self.step);
        self.spoiler.groups.push(SpoilerGroup::default());
    }

    fn sort_spoiler_placements(&mut self) {
        if self.step > 0 {
            self.spoiler.groups[self.step - 1]
                .placements
                .sort_unstable_by(|a, b| {
                    match (
                        CommonItem::from_command(&a.command).into_iter().next(),
                        CommonItem::from_command(&b.command).into_iter().next(),
                    ) {
                        (None, None) => b.item_name.cmp(&a.item_name),
                        (Some(_), None) => Ordering::Greater,
                        (None, Some(_)) => Ordering::Less,
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

    fn force_keystones(&mut self) {
        for world_index in 0..self.worlds.len() {
            let world_context = &mut self.worlds[world_index];
            let owned_keystones = world_context.world.inventory().keystones;
            if owned_keystones < 2 {
                continue;
            }

            let required_keystones = KEYSTONE_DOORS
                .iter()
                .filter_map(|(identifier, amount)| {
                    world_context
                        .reached
                        .iter()
                        .any(|node| node.identifier() == *identifier)
                        .then_some(*amount)
                })
                .sum::<usize>();
            let missing_keystones = required_keystones.saturating_sub(owned_keystones);
            if missing_keystones == 0 {
                continue;
            }

            let item_pool_keystones = world_context.item_pool.inventory().keystones;
            if item_pool_keystones < missing_keystones {
                warning!("Need to place {missing_keystones} to avoid keylocks, but the item pool only has {item_pool_keystones} left. Placing regardless", );
            } else {
                trace!("Placing {missing_keystones} keystones for World {world_index} to avoid keylocks");
            }

            self.spoiler.groups[self.step - 1].forced_items.keystones += missing_keystones;

            let keystone = compile::keystone();
            for _ in 0..missing_keystones {
                self.force_place_command(keystone.clone(), world_index);
            }
            self.worlds[world_index]
                .item_pool
                .change(keystone, -(missing_keystones as i32));
        }
    }

    fn place_remaining(&mut self) {
        trace!("All locations reached. Placing remaining items");
        for target_world_index in 0..self.worlds.len() {
            for command in self.worlds[target_world_index]
                .item_pool
                .drain_random(&mut self.rng)
                .collect::<Vec<_>>()
            {
                self.force_place_command(command, target_world_index);
            }
        }
        for world_context in &mut self.worlds {
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
                        compile::spirit_light(
                            CommandInteger::Constant {
                                value: batch as i32,
                            },
                            &mut self.rng,
                        ),
                    )
                } else {
                    let target_world_index = self.choose_target_world_for_random_placement();
                    (
                        target_world_index,
                        self.worlds[target_world_index]
                            .item_pool
                            .choose_random(&mut self.rng)
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

    fn choose_progression(&mut self) -> Result<Option<(usize, Inventory)>, String> {
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
            self.worlds
                .iter()
                .map(|world_context| {
                    format!(
                        "[World {}]: {} unreached locations: {}\nwith these items: {}",
                        world_context.index,
                        world_context.needs_placement.len(),
                        world_context
                            .needs_placement
                            .iter()
                            .map(|node| node.identifier())
                            .format(", "),
                        world_context.world.player.inventory,
                    )
                })
                .format("\n")
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
        trace!("Placing items which modify uberStates to attempt recovery");

        let initial_reached = self
            .worlds
            .iter()
            .map(|world_context| world_context.reached.len())
            .collect::<Vec<_>>();

        // TODO unbelievably inefficient
        let commands = self
            .worlds
            .iter()
            .flat_map(|world_context| {
                world_context
                    .item_pool
                    .items()
                    .filter(|command| modifies_uberstate(command, &world_context.output))
                    .cloned()
                    .map(|command| (world_context.index, command))
            })
            .collect::<Vec<_>>();
        for (target_world_index, command) in commands {
            self.force_place_command(command.clone(), target_world_index);
            let world_context = &mut self.worlds[target_world_index];
            world_context.item_pool.change(command, -1);
            if world_context.world.reached().len() > initial_reached[target_world_index] {
                trace!("World {target_world_index} reached additional locations, resuming normal placement loop");
                return Ok(());
            }
        }

        Err("Failed to reach all locations".to_string())
    }

    fn place_forced(&mut self, target_world_index: usize, progression: Inventory) {
        self.spoiler.groups[self.step - 1].forced_items += progression.clone();

        let Inventory {
            spirit_light,
            gorlek_ore,
            keystones,
            shard_slots,
            health,
            energy,
            skills,
            shards,
            teleporters,
            clean_water,
            weapon_upgrades,
        } = progression;

        self.worlds[target_world_index].place_spirit_light(
            spirit_light,
            &mut self.spoiler.groups[self.step - 1].placements,
        );
        iter::repeat_with(compile::gorlek_ore)
            .take(gorlek_ore)
            .chain(iter::repeat_with(compile::keystone).take(keystones))
            .chain(iter::repeat_with(compile::shard_slot).take(shard_slots))
            .chain(iter::repeat_with(compile::health_fragment).take(health / 5))
            .chain(iter::repeat_with(compile::energy_fragment).take((energy * 2.) as usize))
            .chain(skills.into_iter().map(compile::skill))
            .chain(shards.into_iter().map(compile::shard))
            .chain(teleporters.into_iter().map(compile::teleporter))
            .chain(clean_water.then(compile::clean_water))
            .chain(weapon_upgrades.into_iter().map(compile::weapon_upgrade))
            .for_each(|command| self.force_place_command(command, target_world_index));
    }

    fn force_place_command(&mut self, command: CommandVoid, target_world_index: usize) {
        let origin_world_index = self.choose_origin_world_for_forced_placement(target_world_index);
        let name = self.name(&command, origin_world_index, target_world_index);
        let origin_world = &mut self.worlds[origin_world_index];
        match origin_world.choose_placement_node(false) {
            None => {
                if origin_world.spawn_slots > 0 {
                    origin_world.spawn_slots -= 1;
                    trace!(
                        "Placing {} for World {target_world_index} at Spawn in World {origin_world_index}",
                        self.worlds[target_world_index].log_name(&command)
                    );
                } else {
                    warning!(
                        "Not enough space to place {} for World {target_world_index}, placing at Spawn despite already having too many spawn items",
                        self.worlds[target_world_index].log_name(&command)
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

    // TODO might be worth to do some single-world happy paths?
    fn choose_origin_world_for_forced_placement(&mut self, target_world_index: usize) -> usize {
        if self.worlds[target_world_index].unshared_items > 0 {
            trace!("World {target_world_index} is not allowed to share items yet, forcing item placement in own world");
            self.worlds[target_world_index].unshared_items -= 1;
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
            let right = match name {
                CommandString::Constant {
                    value: StringOrPlaceholder::Value(value),
                } => CommandString::Constant {
                    value: format!("'s {value}").into(),
                },
                dynamic => CommandString::Concatenate {
                    left: Box::new(CommandString::Constant { value: "'s".into() }),
                    right: Box::new(dynamic),
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
            "Placing {} for World {target_world_index} at {} in World {origin_world_index}",
            self.worlds[target_world_index].log_name(&command),
            node.identifier()
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
        self.spoiler.groups[self.step - 1]
            .placements
            .push(SpoilerPlacement {
                origin_world_index,
                target_world_index,
                location,
                command: command.clone(),
                item_name: self.worlds[target_world_index].log_name(command),
            });
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
                    Seed::new(world_context.output, debug)
                    // TODO add seedgen info (spawn.identifier().to_string())
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
    ) -> Self {
        let mut item_pool = ItemPool::default();

        for (command, amount) in mem::take(&mut output.item_pool_changes) {
            item_pool.change(command, amount);
        }

        world.simulate_client_event(ClientEvent::Spawn, &output);
        world.simulate_client_event(ClientEvent::Reload, &output);

        let mut needs_placement = total_reach_check(&world, &output, &item_pool);
        // TODO maybe optimize based on shape of events, many of which can't possibly be loc_data events
        needs_placement.retain(|node| {
            node.can_place() && {
                let condition = node_condition(node).unwrap();
                !world.simulate(&condition, &output) && !output.events.iter().any(|event|
                    matches!(&event.trigger, Trigger::Condition(trigger) if trigger == &condition)
                )
            }
        });

        Self {
            rng: Pcg64Mcg::from_rng(&mut *rng).expect(SEED_FAILED_MESSAGE),
            world,
            output,
            index,
            item_pool,
            spirit_light_provider: SpiritLightProvider::new(TOTAL_SPIRIT_LIGHT, rng), // TODO how should !add(spirit_light(100)) behave?
            needs_placement,
            placeholders: Default::default(),
            reached: Default::default(),
            progressions: Default::default(),
            reached_needs_placement: Default::default(),
            received_placement: Default::default(),
            reached_item_locations: Default::default(),
            spawn_slots: SPAWN_SLOTS,
            unshared_items: UNSHARED_ITEMS,
            price_distribution: Uniform::new_inclusive(0.75, 1.25),
        }
    }

    fn preplacements(&mut self, preplacement_spoiler: &mut Vec<SpoilerPlacement>) {
        trace!("[World {}] Generating preplacements", self.index);

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
                // TODO maybe remove the log feature gate
                #[cfg(any(feature = "log", test))]
                let index = self.index;
                warning!(
                    "[World {index}] Failed to preplace {} in {zone} since no free placement location was available",
                    self.log_name(&command)
                );
            }
            // We prefer generating indices over shuffling the nodes because usually there aren't many zone preplacements (relics)
            let node_index = nodes.swap_remove(self.rng.gen_range(0..nodes.len()));
            let node = self.needs_placement[node_index];
            self.place(node, command, preplacement_spoiler);
            self.received_placement.push(node_index);
        }
    }

    fn hi_sigma(&mut self, preplacement_spoiler: &mut Vec<SpoilerPlacement>) {
        let command = compile::spirit_light(CommandInteger::Constant { value: 1 }, &mut self.rng);
        if self.needs_placement.is_empty() {
            #[cfg(any(feature = "log", test))]
            let index = self.index;
            warning!(
                "[World {index}] Failed to preplace {} since no free placement location was available",
                self.log_name(&command)
            );
        } else {
            let node = self
                .needs_placement
                .swap_remove(self.rng.gen_range(0..self.needs_placement.len()));
            self.place(node, command, preplacement_spoiler);
        }
    }

    fn update_reached(&mut self) {
        let mut received_placement = mem::take(&mut self.received_placement);
        received_placement.sort();
        for node_index in received_placement.into_iter().rev() {
            self.needs_placement.swap_remove(node_index);
        }

        let reached_locations = self.world.reached_and_progressions();
        self.reached = reached_locations.reached;
        self.progressions = reached_locations.progressions;
        self.reached_needs_placement = self
            .needs_placement
            .iter()
            .enumerate()
            .filter(|(_, node)| self.reached.contains(node))
            .map(|(index, _)| index)
            .collect();
        self.reached_item_locations = self.reached.iter().filter(|node| node.can_place()).count();
        trace!(
            "[World {}] {} reached locations that need placements: {}",
            self.index,
            self.reached_needs_placement.len(),
            self.reached_needs_placement
                .iter()
                .map(|index| self.needs_placement[*index].identifier())
                .format(", ")
        );
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
            "[World {}] Keeping {} placeholders: {}",
            self.index,
            self.placeholders.len(),
            self.placeholders
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

    fn choose_progression(&mut self, slots: usize) -> Option<Inventory> {
        trace!("[World {}] Attempting forced progression", self.index);

        let world_slots = self.progression_slots();
        let mut progressions = mem::take(&mut self.progressions)
            .into_iter()
            .flat_map(|(requirement, best_orbs)| {
                self.world.player.solutions(
                    requirement,
                    &self.world.logic_states,
                    best_orbs,
                    slots,
                    world_slots,
                )
            })
            .filter(|solution| self.item_pool.inventory().contains(solution))
            .collect();
        // TODO is it desirable to filter here again? they have already been filterer per-solutions-call
        filter_redundancies(&mut progressions);

        #[cfg_attr(not(any(feature = "log", test)), allow(unused_mut))]
        let mut weights = progressions
            .iter()
            .enumerate()
            .map(|(index, inventory)| {
                self.world.player.inventory += inventory.clone();
                let mut lookahead_reachable = self.world.reached();
                self.world.player.inventory -= inventory;
                lookahead_reachable.retain(|&node| node.can_place());

                // Resource tracking can result in reaching less locations with an added teleporter, so prevent any overflows.
                // This is very rare and usually means the granted teleporter doesn't actually lead anywhere new, so 0 newly reached is accurate enough.
                let newly_reached = lookahead_reachable
                    .len()
                    .saturating_sub(self.reached_item_locations);

                let mut weight = 1.0 / inventory.cost() as f32 * (newly_reached + 1) as f32;

                // TODO make it less likely to use spawn slots for later progressions?
                let begrudgingly_used_slots = (inventory.item_count()
                    + (SPAWN_SLOTS - PREFERRED_SPAWN_SLOTS))
                    .saturating_sub(slots);
                if begrudgingly_used_slots > 0 {
                    weight *= (0.3_f32).powf(begrudgingly_used_slots as f32);
                }

                (index, weight)
            })
            .collect::<Vec<_>>();

        #[cfg(any(feature = "log", test))]
        {
            weights.sort_unstable_by(|(_, a), (_, b)| OrderedFloat(*b).cmp(&OrderedFloat(*a)));
            let weight_sum = weights.iter().map(|(_, weight)| weight).sum::<f32>();
            let options = weights.iter().map(|(index, weight)| {
                let inventory = &progressions[*index];
                let chance = (*weight / weight_sum) * 100.;
                format!("{chance:.1}%: {inventory}")
            });
            trace!(
                "[World {}] {} options for forced progression:\n{}",
                self.index,
                weights.len(),
                options.format("\n")
            );
        }

        let index = weights
            .choose_weighted(&mut self.rng, |(_, weight)| *weight)
            .ok()?
            .0;
        let progression = progressions.swap_remove(index);
        trace!(
            "[World {}] Chose forced progression: {progression}",
            self.index
        );

        Some(progression)
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
            amount -= batch;
            let command = compile::spirit_light(
                CommandInteger::Constant {
                    value: batch as i32,
                },
                &mut self.rng,
            );
            let node = self.choose_placement_node(true).unwrap();
            self.place(node, command, placement_spoiler);
        }
    }

    fn choose_placement_node(&mut self, is_spirit_light: bool) -> Option<&'graph Node> {
        if is_spirit_light {
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
            if is_spirit_light {
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
                CommonItem::from_command(command)
                    .into_iter()
                    .next()
                    .map_or(MapIcon::QuestItem, |common_item| common_item.map_icon())
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
                warning!("No name specified for custom command: {value}");
                value
            })
    }

    fn simulate_message(&mut self, command: &CommandVoid) -> Option<String> {
        find_message(command).map(|message| self.world.simulate(message, &self.output))
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

        let price = price.unwrap_or_else(|| CommandInteger::Constant {
            value: self.shop_price(command),
        });
        let icon = icon.or_else(|| default_icon(command));

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
        let common_items = CommonItem::from_command(command);

        if matches!(common_items.as_slice(), [CommonItem::Skill(Skill::Blaze)]) {
            return 420;
        }

        let base_price = if common_items.is_empty() {
            200.
        } else {
            common_items
                .into_iter()
                .map(|common_item| common_item.shop_price())
                .sum()
        };

        (base_price * self.price_distribution.sample(&mut self.rng)).round() as i32
    }

    fn fill_remaining(&mut self, placement_spoiler: &mut Vec<SpoilerPlacement>) {
        trace!(
            "[World {}] Filling remaining locations with spirit light",
            self.index
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
                #[cfg(any(feature = "log", test))]
                let index = self.index;
                warning!("[World {index}] Placing more {} than intended to avoid placing Spirit Light in a shop", self.log_name(&command));
                command
            } else {
                compile::spirit_light(
                    CommandInteger::Constant {
                        value: self.spirit_light_provider.take(1 + placements_remaining) as i32,
                    },
                    &mut self.rng,
                )
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
        #[cfg(any(feature = "log", test))]
        let index = self.index;
        trace!(
            "[World {index}] Placing {} at {}",
            self.log_name(&command),
            node.identifier()
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
        self.world.simulate(&command, &self.output);
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
            command: command.clone(),
            item_name: self.log_name(command),
        });
    }
}

fn total_reach_check<'graph>(
    world: &World<'graph, '_>,
    output: &IntermediateOutput,
    item_pool: &ItemPool,
) -> Vec<&'graph Node> {
    let mut world = world.clone();
    for command in item_pool.clone().drain() {
        world.simulate(&command, output);
    }
    world.modify_spirit_light(TOTAL_SPIRIT_LIGHT, output);
    world.reached()
}

pub fn command_name(command: &CommandVoid, item_metadata: &ItemMetadata) -> CommandString {
    item_metadata
        .name(command)
        .map(|value| CommandString::Constant { value })
        .or_else(|| find_message(command).cloned())
        .unwrap_or_else(|| {
            let value = command.to_string();
            warning!("No name specified for custom command: {value}");
            CommandString::Constant {
                value: value.into(),
            }
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

fn default_icon(command: &CommandVoid) -> Option<Icon> {
    CommonItem::from_command(command)
        .into_iter()
        .next()
        .and_then(|common_item| match common_item {
            CommonItem::SpiritLight(_) => {
                Some(Icon::File("assets/icons/game/experience.png".to_string()))
            }
            CommonItem::HealthFragment => Some(Icon::File(
                "assets/icons/game/healthfragment.png".to_string(),
            )),
            CommonItem::EnergyFragment => Some(Icon::File(
                "assets/icons/game/energyfragment.png".to_string(),
            )),
            CommonItem::GorlekOre => {
                Some(Icon::File("assets/icons/game/gorlekore.png".to_string()))
            }
            CommonItem::Keystone => Some(Icon::File("assets/icons/game/keystone.png".to_string())),
            CommonItem::ShardSlot => {
                Some(Icon::File("assets/icons/game/shardslot.png".to_string()))
            }
            CommonItem::WeaponUpgrade(weapon_upgrade) => match weapon_upgrade {
                WeaponUpgrade::ExplodingSpear => Some(Icon::Opher(OpherIcon::ExplodingSpear)),
                WeaponUpgrade::HammerShockwave => Some(Icon::Opher(OpherIcon::HammerShockwave)),
                WeaponUpgrade::StaticShuriken => Some(Icon::Opher(OpherIcon::StaticShuriken)),
                WeaponUpgrade::ChargeBlaze => Some(Icon::Opher(OpherIcon::ChargeBlaze)),
                WeaponUpgrade::RapidSentry => Some(Icon::Opher(OpherIcon::RapidSentry)),
            },
            CommonItem::Shard(shard) => Some(Icon::Shard(shard)),
            CommonItem::Teleporter(_) => {
                Some(Icon::File("assets/icons/game/teleporter.png".to_string()))
            }
            CommonItem::Skill(skill) => match skill {
                Skill::Bash => Some(Icon::Equipment(Equipment::Bash)),
                Skill::DoubleJump => Some(Icon::Equipment(Equipment::Bounce)),
                Skill::Launch => Some(Icon::Equipment(Equipment::Launch)),
                Skill::Glide => Some(Icon::Equipment(Equipment::Glide)),
                Skill::WaterBreath => Some(Icon::Opher(OpherIcon::WaterBreath)),
                Skill::Grenade => Some(Icon::Equipment(Equipment::Grenade)),
                Skill::Grapple => Some(Icon::Equipment(Equipment::Grapple)),
                Skill::Flash => Some(Icon::Equipment(Equipment::Glow)),
                Skill::Spear => Some(Icon::Opher(OpherIcon::Spear)),
                Skill::Regenerate => Some(Icon::Equipment(Equipment::Regenerate)),
                Skill::Bow => Some(Icon::Equipment(Equipment::Bow)),
                Skill::Hammer => Some(Icon::Opher(OpherIcon::Hammer)),
                Skill::Sword => Some(Icon::Equipment(Equipment::Sword)),
                Skill::Burrow => Some(Icon::Equipment(Equipment::Burrow)),
                Skill::Dash => Some(Icon::Equipment(Equipment::Dash)),
                Skill::WaterDash => Some(Icon::Equipment(Equipment::WaterDash)),
                Skill::Shuriken => Some(Icon::Opher(OpherIcon::Shuriken)),
                Skill::Seir => Some(Icon::Equipment(Equipment::Sein)),
                Skill::Blaze => Some(Icon::Opher(OpherIcon::Blaze)),
                Skill::Sentry => Some(Icon::Opher(OpherIcon::Sentry)),
                Skill::Flap => Some(Icon::Equipment(Equipment::Flap)),
                Skill::GladesAncestralLight => Some(Icon::File(
                    "assets/icons/game/ancestrallight1.png".to_string(),
                )),
                Skill::InkwaterAncestralLight => Some(Icon::File(
                    "assets/icons/game/ancestrallight2.png".to_string(),
                )),
                _ => None,
            },
            CommonItem::CleanWater => Some(Icon::File("assets/icons/game/water.png".to_string())),
        })
}

// TODO make a generic contained_commands iterator?
// This catches common cases but would fail to detect edge case commands
fn modifies_uberstate(command: &CommandVoid, output: &IntermediateOutput) -> bool {
    match command {
        CommandVoid::Multi { commands } => commands
            .iter()
            .any(|command| modifies_uberstate(command, output)),
        CommandVoid::Lookup { index } => modifies_uberstate(&output.command_lookup[*index], output),
        CommandVoid::StoreBoolean { .. }
        | CommandVoid::StoreInteger { .. }
        | CommandVoid::StoreFloat { .. } => true,
        _ => false,
    }
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
