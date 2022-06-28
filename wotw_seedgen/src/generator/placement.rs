use std::mem;

use rand::{
    Rng,
    seq::{SliceRandom, IteratorRandom},
    distributions::{Distribution, Uniform, Bernoulli},
};

use crate::{
    inventory::Inventory,
    item::{Item, Resource, Skill, Teleporter, Command, ShopCommand, Message},
    settings::{Difficulty, Goal, WorldSettings, Spawn}, util::{
        self,
        UberState, UberType,
        constants::{RELIC_ZONES, KEYSTONE_DOORS, RESERVE_SLOTS, PLACEHOLDER_SLOTS, SHOP_PRICES, DEFAULT_SPAWN, RANDOM_PROGRESSION, RETRIES, GORLEK_SPAWNS, MOKI_SPAWNS},
    }, world::{
        World,
        graph::{self, Node, Graph},
        player::Player,
    }, header::CodeDisplay
};

use super::seed::SeedWorld;
use super::spoiler::{SeedSpoiler, SpoilerGroup, SpoilerWorldReachable, SpoilerPlacement};

#[derive(Debug, Clone)]
/// One [`Item`] placed on one [`UberState`] location
pub struct Placement<'a> {
    /// The [`Node`] from the logic [`Graph`] this was placed on
    pub node: Option<&'a Node>,
    /// The [`UberState`] describing this placement's location
    pub uber_state: UberState,
    /// The [`Item`] to be granted when collecting this placement
    pub item: Item,
}

impl Placement<'_> {
    pub fn code(&self) -> CodeDisplay<Placement> {
        CodeDisplay::new(self, |s, f| { write!(f, "{}|{}", s.uber_state.code(), s.item.code()) })
    }
}

struct WorldContext<'a, 'b> {
    world: World<'a, 'b>,
    spawn: &'a Node,
    placements: Vec<Placement<'a>>,
    placeholders: Vec<&'a Node>,
    collected_preplacements: Vec<usize>,
    spawn_slots: Vec<&'a Node>,
    reachable_locations: Vec<&'a Node>,
    unreachable_locations: Vec<&'a Node>,
    spirit_light_rng: SpiritLightAmounts,  // TODO this can get kinda weird maybe have a shared spirit light rng instead
    random_spirit_light: Bernoulli,
    shop_slots: usize,
    world_tour: Option<usize>,
}

struct GeneratorContext<'a, R, I>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    world_count: usize,
    total_reachable_count: usize,
    multiworld_state_index: I,
    spoiler_groups: Vec<SpoilerGroup>,
    current_spoiler_group: SpoilerGroup,
    price_range: Uniform<f32>,
    random_progression: Bernoulli,
    rng: &'a mut R,
}

impl<R: Rng, I: Iterator<Item=usize>> GeneratorContext<'_, R, I> {
    fn finalize_spoiler_group(&mut self) {
        self.current_spoiler_group.placements.sort_unstable_by(|a, b| a.item.cmp(&b.item));
        self.spoiler_groups.push(mem::take(&mut self.current_spoiler_group));
    }
}

struct ReachContext<'a> {
    reachable: Vec<graph::Reached<'a>>,
    reachable_states: Vec<Vec<&'a Node>>,
    unmet: Vec<graph::Progressions<'a>>,
    reachable_counts: Vec<usize>,
    unreached_count: usize,
}

fn format_identifiers(mut identifiers: Vec<&str>) -> String {
    let length = identifiers.len();
    if length > 20 {
        identifiers.truncate(20);
    }

    let mut identifiers = identifiers.join(", ");

    if length > 20 {
        identifiers.push_str(&format!("... ({} total)", length));
    }

    identifiers
}

fn progression_check<'a, R, I>(world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<ReachContext<'a>, String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let mut reachable = Vec::with_capacity(context.world_count);
    let mut reachable_states = Vec::with_capacity(context.world_count);
    let mut unmet = Vec::with_capacity(context.world_count);

    for world_context in world_contexts {
        let (world_reachable, world_unmet) = world_context.world.graph.reached_and_progressions(&world_context.world.player, world_context.spawn, &world_context.world.uber_states, &world_context.world.sets)?;
        let world_reachable_states = world_reachable.iter().filter(|node| !node.can_place()).copied().collect::<Vec<_>>();
        reachable.push(world_reachable);
        reachable_states.push(world_reachable_states);
        unmet.push(world_unmet);
    }

    let reachable_counts = reachable.iter()
        .map(|world_reachable| world_reachable.iter().filter(|node| node.can_place()).count())
        .collect::<Vec<_>>();
    let unreached_count = context.total_reachable_count - reachable_counts.iter().sum::<usize>();

    Ok(ReachContext {
        reachable,
        reachable_states,
        unmet,
        reachable_counts,
        unreached_count,
    })
}

fn place_item<'a, R, I>(origin_world_index: usize, target_world_index: usize, node: &'a Node, was_placeholder: bool, forced: bool, item: Item, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let origin_world_context = &mut world_contexts[origin_world_index];

    let uber_state = node.uber_state().unwrap();
    let is_shop = uber_state.is_shop();

    if uber_state.is_purchasable() {
        origin_world_context.shop_slots -= 1;

        if is_shop {
            shop_placement(node, &item, origin_world_index, origin_world_context, context)?;
        }
    }

    let origin_details = origin_world_context.world.custom_items.get(&item);
    let custom_name =  origin_details.and_then(|details| details.name.clone());
    let display =  origin_details.and_then(|details| details.display.clone());
    let generic_name = custom_name.clone().unwrap_or_else(|| item.to_string());

    if origin_world_index == target_world_index {
        log::trace!("(World {}): Placed {} at {}", origin_world_index, generic_name, if was_placeholder { format!("placeholder {} ({} left)", node, origin_world_context.placeholders.len()) } else { format!("{}", node) });

        origin_world_context.placements.push(Placement {
            node: Some(node),
            uber_state: uber_state.clone(),
            item: item.clone(),
        });

        if is_shop {
            if let Some(name) = custom_name {
                origin_world_context.placements.push(Placement {
                    node: Some(node),
                    uber_state: uber_state.clone(),
                    item: Item::Message(Message::new(name)),
                });
            }
        } else if let Some(display) = display.or(custom_name) {
            origin_world_context.placements.push(Placement {
                node: Some(node),
                uber_state: uber_state.clone(),
                item: Item::Message(Message::new(display)),
            });
        }
    } else {
        log::trace!("(World {}): Placed World {}'s {} at {}", origin_world_index, target_world_index, generic_name, if was_placeholder { format!("placeholder {} ({} left)", node, origin_world_context.placeholders.len()) } else { format!("{}", node) });

        let target_world_context = &mut world_contexts[target_world_index];
        let target_details = target_world_context.world.custom_items.get(&item);
        let target_display = target_details.and_then(|details| details.display.clone());

        let state_index = context.multiworld_state_index.next().unwrap();

        let custom_name = custom_name.unwrap_or_else(|| format!("$[{}]", item.code()));
        let origin_message = Item::Message(Message::new(format!("$[15|5|{}]'s {}", target_world_index, custom_name)));
        let send_item = UberState::from_parts("12" , &state_index.to_string())?.to_item(UberType::Bool);
        let target_message = Item::Message(Message::new(format!("{} from $[15|5|{}]|mute", target_display.unwrap_or(custom_name), origin_world_index)));
        let target_uber_state = UberState::from_parts("12", &state_index.to_string())?;

        target_world_context.placements.push(Placement {
            node: None,
            uber_state: target_uber_state.clone(),
            item: item.clone(),
        });
        target_world_context.placements.push(Placement {
            node: None,
            uber_state: target_uber_state,
            item: target_message,
        });
        let origin_world_context = &mut world_contexts[origin_world_index];
        origin_world_context.placements.push(Placement {
            node: Some(node),
            uber_state: node.uber_state().unwrap().clone(),
            item: send_item,
        });
        origin_world_context.placements.push(Placement {
            node: Some(node),
            uber_state: node.uber_state().unwrap().clone(),
            item: origin_message,
        });
    }

    let node_identifier = node.identifier().to_string();
    let node_position = node.position().cloned();
    context.current_spoiler_group.placements.push(SpoilerPlacement { forced, origin_world_index, target_world_index, node_identifier, node_position, item });

    Ok(())
}

fn shop_placement<R, I>(node: &Node, item: &Item, world_index: usize, world_context: &mut WorldContext, context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let details = world_context.world.custom_items.get(item);
    let uber_state = node.uber_state().unwrap();

    let (_, _, price_uber_state) = SHOP_PRICES.iter()
        .find(|(_, location, _)| &uber_state.identifier == location)
        .ok_or_else(|| format!("(World {}): {} claims to be a shop location, but doesn't have an entry in the shop prices table!", world_index, node))?;

    let mut price = details.and_then(|details| details.price).unwrap_or_else(|| item.shop_price());

    if item.random_shop_price() {
        let modified_price = price as f32 * context.price_range.sample(context.rng);
        price = util::float_to_int(modified_price).map_err(|_| format!("(World {}): Overflowed shop price for {} after adding a random amount to it", world_index, item))?;
    }

    let price_setter = UberState {
        identifier: price_uber_state.clone(),
        value: price.to_string(),
    }.to_item(UberType::Int);

    log::trace!("(World {}): Placing {} at Spawn as price for the item below", world_index, price_setter);

    world_context.placements.push(Placement {
        node: None,
        uber_state: UberState::load(),
        item: price_setter,
    });

    let description = details.and_then(|details| details.description.clone()).or_else(|| item.description());
    if description.is_some() {
        let description_setter = Item::ShopCommand(ShopCommand::SetDescription {
            uber_identifier: uber_state.identifier.clone(),
            description,
        });

        world_context.placements.push(Placement {
            node: None,
            uber_state: UberState::load(),
            item: description_setter,
        });
    }

    if let Some(icon) = details.and_then(|details| details.icon.clone()).or_else(|| item.icon()) {
        let icon_setter = Item::ShopCommand(ShopCommand::SetIcon {
            uber_identifier: uber_state.identifier.clone(),
            icon,
        });

        world_context.placements.push(Placement {
            node: None,
            uber_state: UberState::load(),
            item: icon_setter,
        });
    }

    Ok(())
}

fn place_relics<'a, R, I>(amount: usize, world_index: usize, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let world_context = &mut world_contexts[world_index];

    let mut relic_zones = RELIC_ZONES.to_vec();
    relic_zones.shuffle(context.rng);
    relic_zones.truncate(amount);

    let mut relic_locations = relic_zones.into_iter().map(|zone| (zone, Vec::with_capacity(60))).collect::<Vec<_>>();

    for &node in &world_context.reachable_locations {
        if let Some(zone) = node.zone() {
            if let Some((_, zone_relic_locations)) = relic_locations.iter_mut().find(|(relic_zone, _)| zone == *relic_zone) {
                let uber_state = node.uber_state().unwrap();

                if !world_context.world.preplacements.contains_key(uber_state)
                && !world_context.placements.iter().any(|placement| &placement.uber_state == uber_state) {
                    zone_relic_locations.push(node);
                }
            }
        }
    }

    for (zone, relic_locations) in &mut relic_locations {
        log::trace!("(World {}): Placing Relic in {}", world_index, zone);

        if let Some(&location) = relic_locations.choose(context.rng) {
            place_item(world_index, world_index, location, false, false, Item::Relic(*zone), world_contexts, context)?;
        }
    }

    Ok(())
}

#[inline]
fn force_keystones<'a, R, I>(reachable_states: &[Vec<&Node>], reserved_slots: &mut Vec<(usize, &'a Node)>, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    for target_world_index in 0..context.world_count {
        let mut missing_keystones = 0;

        let world_context = &mut world_contexts[target_world_index];

        let placed_keystones = world_context.world.player.inventory.get(&Item::Resource(Resource::Keystone));
        if placed_keystones < 2 { return Ok(()); }

        let required_keystones: u32 = reachable_states[target_world_index].iter()
            .filter_map(|&node| {
                if let Some((_, keystones)) = KEYSTONE_DOORS.iter().find(|&&(identifier, _)| identifier == node.identifier()) {
                    return Some(*keystones);
                }
                None
            })
            .sum();
        if required_keystones <= placed_keystones { return Ok(()); }

        missing_keystones += required_keystones - placed_keystones;

        log::trace!("(World {}): Force placing {} keystones to avoid keylocks", target_world_index, missing_keystones);

        for _ in 0..missing_keystones {
            forced_placement(target_world_index, Item::Resource(Resource::Keystone), reserved_slots, world_contexts, context)?;
        }
    }

    Ok(())
}

fn forced_placement<'a, R, I>(target_world_index: usize, item: Item, reserved_slots: &mut Vec<(usize, &'a Node)>, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let is_multiworld_spread = item.is_multiworld_spread();

    let mut choose_node = || {
        if is_multiworld_spread {
            let mut world_indices = (0..context.world_count).collect::<Vec<_>>();
            world_indices.shuffle(context.rng);

            if let Some((origin_world_index, node)) = reserved_slots.pop() {
                return Ok((origin_world_index, node, false));
            }
            for origin_world_index in world_indices {
                let placeholders = &mut world_contexts[origin_world_index].placeholders;
                if !placeholders.is_empty() {
                    let index = context.rng.gen_range(0..placeholders.len());
                    let node = placeholders.remove(index);
                    return Ok((origin_world_index, node, true));
                }
            }
        } else {
            if let Some((index, _)) = reserved_slots.iter().enumerate().find(|(_, (world_index, _))| world_index == &target_world_index) {
                let (_, node) = reserved_slots.remove(index);
                return Ok((target_world_index, node, false));
            }

            let placeholders = &mut world_contexts[target_world_index].placeholders;
            if !placeholders.is_empty() {
                let index = context.rng.gen_range(0..placeholders.len());
                let node = placeholders.remove(index);
                return Ok((target_world_index, node, true));
            }
        }
        return Err(format!("(World {}): Not enough slots to place forced progression {}", target_world_index, item))  // due to the slot checks in missing_items this should only ever happen for forced keystone placements
    };

    let mut node = choose_node()?;

    // Don't place Spirit Light in shops
    if matches!(item, Item::SpiritLight(_)) {
        let mut skipped_slots = Vec::new();

        while node.1.uber_state().unwrap().is_purchasable() {
            skipped_slots.push((node.0, node.1));

            node = choose_node()?;
        }

        for skipped_slot in skipped_slots {
            world_contexts[skipped_slot.0].placeholders.push(skipped_slot.1);
        }
    }

    let world_context = &mut world_contexts[target_world_index];
    world_context.world.pool.remove(&item, 1);
    world_context.world.grant_player(item.clone(), 1).unwrap_or_else(|err| log::error!("(World {}): {}", target_world_index, err));
    place_item(node.0, target_world_index, node.1, node.2, true, item, world_contexts, context)?;

    Ok(())
}

fn determine_progressions<'a>(world_index: usize, slots: usize, world_slots: usize, reach_context: &ReachContext, world_context: &WorldContext<'a, '_>) -> Result<Vec<Inventory>, String> {
    let mut itemsets = Vec::new();

    let owned_states = reach_context.reachable_states[world_index].iter().map(|&node| node.index()).collect::<Vec<_>>();

    for (requirement, best_orbs) in &reach_context.unmet[world_index] {
        let items = requirement.items_needed(&world_context.world.player, &owned_states);
        // TODO this is a giant mess of redundancies
        // log::trace!("requirement: {:?}", requirement);

        for (mut needed, orb_cost) in items {
            world_context.world.player.missing_items(&mut needed);

            for orbs in best_orbs {
                let orb_variants = world_context.world.player.missing_for_orbs(&needed, orb_cost, *orbs);

                for missing in orb_variants {
                    // log::trace!("missing items: {}", missing);

                    if missing.items.is_empty() {  // sanity check
                        log::trace!("(World {}): Failed to determine which items were needed for progression to meet {:?} (had {})", world_index, requirement, world_context.world.player.inventory);
                        return Err(String::from("Failed to determine which items were needed for progression"));
                    }
                    if missing.item_count() > slots
                    || missing.world_item_count() > world_slots
                    || !world_context.world.pool.contains(&missing) { continue; }

                    itemsets.push(missing);
                }
            }
        }
    }

    Ok(itemsets)
}

fn filter_itemsets(itemsets: &mut Vec<Inventory>) {
    itemsets.sort_unstable_by_key(Inventory::item_count);
    itemsets.reverse();
    let mut index = 0;
    for _ in 0..itemsets.len() {
        let current = &itemsets[index];
        if itemsets[index + 1..].iter().any(|other| current.contains(other)) {
            itemsets.remove(index);
        } else {
            index += 1;
        }
    }
}

fn pick_progression<'a, 'b, R, I>(target_world_index: usize, itemsets: &'b [Inventory], slots: usize, reach_context: &ReachContext, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<&'b Inventory, String>where
R: Rng,
I: Iterator<Item=usize>,
{
    log::trace!("(World {}): {} options for forced progression:", target_world_index, itemsets.len());

    let weight = |inventory: &Inventory| -> Result<f32, String> {
        let mut newly_reached = 0;

        let target_world_context = &world_contexts[target_world_index];

        let lookahead_player = Player {
            inventory: target_world_context.world.player.inventory.merge(inventory),
            ..target_world_context.world.player.clone()
        };
        let mut lookahead_reachable = target_world_context.world.graph.reached_locations(&lookahead_player, target_world_context.spawn, &target_world_context.world.uber_states, &target_world_context.world.sets)?;
        lookahead_reachable.retain(|&node| node.can_place());

        newly_reached += lookahead_reachable.len().saturating_sub(reach_context.reachable_counts[target_world_index]);
        // Resource tracking can result in reaching less locations with an added teleporter, so prevent any overflows.
        // This is very rare and usually means the granted teleporter doesn't actually lead anywhere new, so 0 newly reached is accurate enough.

        lookahead_reachable.retain(|&node| node.uber_state().map_or(false, |reached| target_world_context.world.preplacements.keys().any(|preplaced| reached == preplaced)));
        let preplaced_reached = lookahead_reachable.len();

        if slots - inventory.item_count() < 3 && newly_reached <= preplaced_reached {
            return Ok(0.000_001);
        }

        let base_weight = 1.0 / inventory.cost();

        Ok(base_weight * (newly_reached + 1) as f32)
    };
    let with_weights = itemsets.iter()
        .map::<Result<(&Inventory, f32), String>, _>(|inventory| Ok((inventory, weight(inventory)?)))
        .collect::<Result<Vec<_>, _>>()?;
    let weight_sum: f32 = with_weights.iter().map(|(_, weight)| weight).sum();

    let (progression, _) = with_weights
        .choose_weighted(context.rng, |&(inventory, weight)| {
            let mut inventory = format!("{}", inventory);
            util::add_trailing_spaces(&mut inventory, 20);
            log::trace!("-> {}  ({}%)", inventory, (weight / weight_sum * 1000.0).round() / 10.0);

            weight
        })
        .map_err(|err| format!("(World {}): Error choosing progression: {}", target_world_index, err))?;

    log::trace!("(World {}): Chosen progression: {}", target_world_index, progression);

    Ok(progression)
}

fn split_progression_item<'a, R, I>(world_index: usize, item: &Item, amount: u32, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Vec<Item>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    if let Item::SpiritLight(1) = item {
        let mut spirit_light_items = Vec::with_capacity(1);
        let mut amount_placed = 0;

        while amount_placed < amount {
            let stacked_amount = world_contexts[world_index].spirit_light_rng.sample(context.rng);
            amount_placed += stacked_amount;
            spirit_light_items.push(Item::SpiritLight(stacked_amount));
        }

        spirit_light_items
    } else {
        vec![item.clone(); amount as usize]
    }
}

#[inline]
fn spawn_progressions<'a, R, I>(world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    'outer: for world_index in 0..context.world_count {
        let mut random_slots = 2;

        loop {
            let available_spawn_slots = world_contexts[world_index].spawn_slots.len();
            if available_spawn_slots <= random_slots {
                continue 'outer;
            }

            context.finalize_spoiler_group();

            log::trace!("(World {}): Placing spawn progression", world_index);

            let reach_context = progression_check(world_contexts, context)?;  // TODO This is inefficient! The problem here is that the ReachContext always holds all worlds at once. Maybe it should be a Vec of per-world Reach contexts?
            let world_context = &world_contexts[world_index];

            let mut itemsets = determine_progressions(world_index, available_spawn_slots, available_spawn_slots, &reach_context, world_context)?;

            if itemsets.is_empty() {
                log::trace!("(World {}): No progressions found", world_index);
                continue 'outer;
            }

            filter_itemsets(&mut itemsets);
            let progression = pick_progression(world_index, &itemsets, available_spawn_slots, &reach_context, world_contexts, context)?;

            for (item, amount) in &progression.items {
                let items = split_progression_item(world_index, item, *amount, world_contexts, context);

                for item in items {
                    let world_context = &mut world_contexts[world_index];
                    world_context.world.pool.remove(&item, 1);
                    world_context.world.grant_player(item.clone(), 1).unwrap_or_else(|err| log::error!("(World {}): {}", world_index, err));

                    log::trace!("(World {}): Placed {} as spawn progression", world_index, item);

                    if matches!(item, Item::Skill(Skill::Regenerate)) {
                        random_slots -= 1;
                    }

                    let world_context = &mut world_contexts[world_index];
                    let node = world_context.spawn_slots.pop();

                    context.current_spoiler_group.placements.push(SpoilerPlacement {
                        forced: true,
                        origin_world_index: world_index,
                        target_world_index: world_index,
                        node_identifier: "Spawn".to_string(),
                        node_position: None,
                        item: item.clone(),
                    });
                    world_context.placements.push(Placement {
                        node,
                        uber_state: UberState::spawn(),
                        item,
                    });
                }
            }
        }
    }

    Ok(())
}

fn force_progression<'a, R, I>(reserved_slots: &mut Vec<(usize, &'a Node)>, reach_context: &mut ReachContext,world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let slots = reserved_slots.len() + world_contexts.iter().map(|world_context| world_context.placeholders.len()).sum::<usize>();

    let mut world_indices = (0..context.world_count).collect::<Vec<_>>();
    world_indices.shuffle(context.rng);

    let (target_world_index, mut itemsets) = loop {
        if let Some(chosen_world_index) = world_indices.pop() {
            let world_context = &mut world_contexts[chosen_world_index];
            let world_slots = reserved_slots.iter().filter(|(world_index, _)| *world_index == chosen_world_index).count() + world_context.placeholders.len();

            let itemsets = determine_progressions(chosen_world_index, slots, world_slots, reach_context, world_context)?;

            if itemsets.is_empty() {
                log::trace!("(World {}): No progressions found", chosen_world_index);
            } else {
                break (chosen_world_index, itemsets);
            }
        } else {
            if world_contexts.iter().any(|world_context| !world_context.placeholders.is_empty()) &&
            world_contexts.iter().any(|world_context| world_context.world.pool.inventory.items.iter().any(|(item, _)| matches!(item, Item::UberState(_)))) {
                flush_item_pool(world_contexts, context)?;
                return Ok(());
            }

            if world_contexts.iter().all(|world_context| world_context.placements.is_empty()) {
                for (world_index, world_context) in world_contexts.iter().enumerate() {
                    log::trace!("(World {}): Failed to reach anything from spawn location {}", world_index, world_context.spawn);
                }
                return Err(String::from("Failed to reach anything from spawn location"));
            }

            for (world_index, world_context) in world_contexts.iter().enumerate() {
                let identifiers: Vec<_> = world_context.reachable_locations.iter()
                    .filter_map(|&node| {
                        let node_index = node.index();

                        node.uber_state().and_then(|uber_state|
                            if !world_context.placements.iter().any(|placement| &placement.uber_state == uber_state) &&
                            !world_context.placeholders.iter().any(|&placeholder| placeholder.index() == node_index) &&
                            !world_context.collected_preplacements.iter().any(|&collected| collected == node_index)
                            {
                                Some(node.identifier())
                            } else { None }
                        )
                    })
                    .collect();

                log::trace!("(World {}): Failed to reach all locations with inventory: {}", world_index, world_context.world.player.inventory);
                log::error!("(World {}): Couldn't reach locations {}", world_index, format_identifiers(identifiers));
            }

            return Err(String::from("Failed to reach all locations"));
        }
    };

    filter_itemsets(&mut itemsets);
    let progression = pick_progression(target_world_index, &itemsets, slots, reach_context, world_contexts, context)?;

    for (item, amount) in &progression.items {
        let items = split_progression_item(target_world_index, item, *amount, world_contexts, context);

        for item in items {
            forced_placement(target_world_index, item, reserved_slots, world_contexts, context)?;
        }
    }

    Ok(())
}

#[inline]
fn random_item_placement<'a, R, I>(origin_world_index: usize, node: &'a Node, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<bool, String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let origin_world_context = &mut world_contexts[origin_world_index];

    let is_purchasable = node.uber_state().map_or(false, UberState::is_purchasable);

    if is_purchasable || !origin_world_context.random_spirit_light.sample(context.rng) {
        let target_world_index = context.rng.gen_range(0..context.world_count);

        if is_purchasable || origin_world_context.shop_slots < world_contexts[target_world_index].world.pool.inventory.item_count() {
            let target_world_context = &mut world_contexts[target_world_index];

            if let Some(item) = target_world_context.world.pool.choose_random(origin_world_index != target_world_index, context.rng) {
                let item = item.clone();
                let is_progression = item.is_progression(target_world_context.world.player.settings.difficulty);
                target_world_context.world.pool.remove(&item, 1);
                target_world_context.world.grant_player(item.clone(), 1).unwrap_or_else(|err| log::error!("(World {}): {}", target_world_index, err));
                place_item(origin_world_index, target_world_index, node, false, false, item, world_contexts, context)?;

                return Ok(is_progression);
            }
        } else {
            log::trace!("(World {}) Forcing spirit light placement to preserve items for shop slots", origin_world_index);
        }
    }

    let origin_world_context = &mut world_contexts[origin_world_index];

    let amount = origin_world_context.spirit_light_rng.sample(context.rng);
    let item = Item::SpiritLight(amount);

    origin_world_context.world.pool.remove(&item, 1);
    origin_world_context.world.grant_player(item.clone(), 1).unwrap_or_else(|err| log::error!("(World {}): {}", origin_world_index, err));
    place_item(origin_world_index, origin_world_index, node, false, false, item, world_contexts, context)?;

    Ok(true)
}

#[inline]
/// May randomly place an item, or add a placeholder
/// Returns `true` if a progression item was placed, `false` if a placeholder was added or the placed item cannot be progression
fn random_placement<'a, R, I>(origin_world_index: usize, node: &'a Node, allow_placeholder: bool, world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<bool, String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let origin_world_context = &mut world_contexts[origin_world_index];

    // force a couple placeholders at the start
    let mut force = false;
    if origin_world_context.placeholders.len() < 4 {
        force = true;
    } else if context.random_progression.sample(context.rng) {
        return random_item_placement(origin_world_index, node, world_contexts, context);
    }

    let origin_world_context = &mut world_contexts[origin_world_index];
    log::trace!("(World {}): Reserving {} as {}placeholder", origin_world_index, node, if force { "forced " } else { "" });

    origin_world_context.placeholders.push(node);
    if !allow_placeholder {
        let placeholder_index = context.rng.gen_range(0..origin_world_context.placeholders.len());
        let placeholder = origin_world_context.placeholders.remove(placeholder_index);
        return random_item_placement(origin_world_index, placeholder, world_contexts, context);
    }

    Ok(false)
}

#[inline]
fn one_xp<'a, R, I>(world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    for world_index in 0..context.world_count {
        let world_context = &world_contexts[world_index];

        if let Some(node) = world_context.world.graph.nodes.iter()
            .filter(|&node|
                node.can_place() &&
                node.uber_state().map_or(true, |uber_state| !world_context.world.preplacements.contains_key(uber_state)))
            .choose(context.rng)
        {
            place_item(world_index, world_index, node, false, false, Item::SpiritLight(1), world_contexts, context)?;
        }
    }

    Ok(())
}

/* proposed per-pickup exp formula:
 * exp = M * (n^2) + base*roll
 * where:
 * n = the number of exp pickups placed so far
 * base = the minimum starting value of an ex pickup
 * roll = a float multiplier to provide some randomness
 * M = a multplier calculated such that the sum of every exp value (before randomness) is equal to a total (see factor for the math)
 * this gives us a nice shallow parabola with some randomness but not so much that you can't tell approximately when a pickup was placed
 */
struct SpiritLightAmounts {
    factor: f32,
    noise: Uniform<f32>,
    index: usize,
}
impl SpiritLightAmounts {
    fn new(spirit_light_pool: f32, spirit_light_slots: f32, random_low: f32, random_high: f32) -> SpiritLightAmounts {
        let factor = (spirit_light_pool as f32 - spirit_light_slots * 50.0) / (spirit_light_slots.powi(3) / 3.0 + spirit_light_slots.powi(2) / 2.0 + spirit_light_slots / 6.0);
        let noise = Uniform::new_inclusive(random_low, random_high);

        SpiritLightAmounts {
            factor,
            noise,
            index: 0,
        }
    }
    fn sample(&mut self, rng: &mut impl Rng) -> u32 {
        #[allow(clippy::cast_precision_loss)]
        let amount = (self.factor * self.index.pow(2) as f32 + 50.0 * self.noise.sample(rng)).round();
        self.index += 1;

        #[allow(clippy::cast_possible_truncation)]
        util::float_to_int(amount).unwrap_or(u32::MAX)
    }
}

fn place_remaining<'a, R, I>(world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    let mut shop_placeholders = vec![Vec::new(); context.world_count];

    for world_index in 0..context.world_count {
        let world_context = &mut world_contexts[world_index];

        world_context.placeholders.retain(|&node| {
            if node.uber_state().unwrap().is_purchasable() {
                shop_placeholders[world_index].push(node);
                false
            } else { true }
        });

        shop_placeholders[world_index].shuffle(context.rng);
        world_context.placeholders.shuffle(context.rng);
        world_context.unreachable_locations.shuffle(context.rng);
    }

    for target_world_index in 0..context.world_count {
        let mut remaining = world_contexts[target_world_index].world.pool.inventory.items.drain()
            .flat_map(|(item, amount)| vec![item; amount as usize])
            .collect::<Vec<_>>();
        log::trace!("(World {}): Placing the remaining {} items randomly", target_world_index, remaining.len());

        remaining.shuffle(context.rng);

        let mut space_remaining = true;
        'outer: for item in remaining {
            if space_remaining {
                let origin_world_indices = if item.is_multiworld_spread() {
                    let mut indices = (0..context.world_count).collect::<Vec<_>>();
                    indices.shuffle(context.rng);
                    indices
                } else {
                    vec![target_world_index]
                };

                for origin_world_index in origin_world_indices {
                    if let Some(node) = shop_placeholders[origin_world_index].pop().or_else(|| world_contexts[origin_world_index].placeholders.pop()) {
                        place_item(origin_world_index, target_world_index, node, true, false, item, world_contexts, context)?;
                        continue 'outer;
                    }
                }

                space_remaining = false;

                log::warn!("(World {}): Not enough space to place all items from the item pool!", target_world_index);
                log::trace!("Unable to place {}", item);
            } else {
                log::trace!("Unable to place {}", item);
            }
        }
    }

    for (world_index, world_shop_placeholders) in shop_placeholders.iter().enumerate() {
        if !world_shop_placeholders.is_empty() {
            log::warn!("(World {}): Not enough items in the pool to fill all shops! Filling with extra Gorlek Ore", world_index);

            for &world_shop_placeholder in world_shop_placeholders {
                place_item(world_index, world_index, world_shop_placeholder, true, false, Item::Resource(Resource::Ore), world_contexts, context)?;
            }
        }
    }

    for world_index in 0..context.world_count {
        log::trace!("(World {}): Placed all items from the pool, placing Spirit Light", world_index);

        while let Some(placeholder) = world_contexts[world_index].placeholders.pop() {
            let amount = world_contexts[world_index].spirit_light_rng.sample(context.rng);
            let item = Item::SpiritLight(amount);

            place_item(world_index, world_index, placeholder, true, false, item, world_contexts, context)?;
        }

        if !world_contexts[world_index].unreachable_locations.is_empty() {
            log::trace!("(World {}): Filling unreachable locations", world_index);
        }
        while let Some(unreachable) = world_contexts[world_index].unreachable_locations.pop() {
            let item = if unreachable.uber_state().map_or(false, UberState::is_purchasable) {
                Item::Resource(Resource::Ore)
            } else {
                let amount = world_contexts[world_index].spirit_light_rng.sample(context.rng);
                Item::SpiritLight(amount)
            };

            place_item(world_index, world_index, unreachable, false, false, item, world_contexts, context)?;
        }

        world_contexts[world_index].placements.shrink_to_fit();
    }

    Ok(())
}

#[inline]
fn total_reach_check<'a>(world: &World<'a, '_>) -> Result<Vec<&'a Node>, String> {
    let mut finished_world = world.clone();
    for (item, amount) in &world.pool.inventory.items {
        if item.is_progression(world.player.settings.difficulty) {
            finished_world.grant_player(item.clone(), *amount)?;
        }
    }
    finished_world.grant_player(Item::SpiritLight(1), world.pool.spirit_light)?;

    let mut collected_preplacements = Vec::new();
    let mut total_reachable_count = 0;

    let spawn = finished_world.graph.find_spawn(DEFAULT_SPAWN)?;

    loop {
        let mut reachable_locations = finished_world.graph.reached_locations(&finished_world.player, spawn, &finished_world.uber_states, &finished_world.sets)?;
        let new_reachable_count = reachable_locations.len();

        if new_reachable_count > total_reachable_count {
            total_reachable_count = new_reachable_count;
        } else {
            reachable_locations.retain(|&node| node.can_place());
            return Ok(reachable_locations);
        }

        reachable_locations.retain(|&node| {
            node.uber_state().is_some() &&
            !collected_preplacements.iter().any(|&index| index == node.index())
        });

        for node in reachable_locations {
            let preplaced = finished_world.collect_preplacements(node.uber_state().unwrap());
            if preplaced {
                collected_preplacements.push(node.index());
            }
        }
    };
}

fn flush_item_pool<'a, R, I>(world_contexts: &mut [WorldContext<'a, '_>], context: &mut GeneratorContext<'_, R, I>) -> Result<(), String>
where
    R: Rng,
    I: Iterator<Item=usize>,
{
    log::trace!("Got stuck. Trying to flush uberState items from the item pool to recover...");

    for target_world_index in 0..context.world_count {
        let uber_state_items = world_contexts[target_world_index].world.pool.inventory.items.iter()
            .filter(|(item, _)| matches!(item, Item::UberState(_)))
            .flat_map(|(item, amount)| vec![item.clone(); *amount as usize])
            .collect::<Vec<_>>();

        'outer: for item in uber_state_items {
            let mut origin_world_indices = (0..context.world_count).collect::<Vec<_>>();
            origin_world_indices.shuffle(context.rng);

            for origin_world_index in origin_world_indices {
                if let Some(node) = world_contexts[origin_world_index].placeholders.pop() {
                    let target_world_context = &mut world_contexts[target_world_index];

                    target_world_context.world.pool.remove(&item, 1);
                    target_world_context.world.grant_player(item.clone(), 1).unwrap_or_else(|err| log::error!("(World {}): {}", target_world_index, err));
                    place_item(origin_world_index, target_world_index, node, true, false, item, world_contexts, context)?;

                    continue 'outer;
                }
            }
        }
    }

    Ok(())
}

fn generate_placements_from_spawn<'a, 'b>(
    worlds: Vec<World<'a, 'b>>,
    spawns: Vec<&'a Node>,
    rng: &mut impl Rng
) -> Result<(Vec<SeedWorld<'a>>, SeedSpoiler), String> {
    // TODO enforce a max total price for shops
    let price_range = Uniform::new_inclusive(0.75, 1.25);

    let mut world_contexts = build_world_contexts(worlds, &spawns, rng)?;

    let total_reachable_count: usize = world_contexts.iter().map(|world_context| world_context.reachable_locations.len()).sum();

    let mut context = GeneratorContext {
        world_count: world_contexts.len(),
        total_reachable_count,
        multiworld_state_index: 0..,
        spoiler_groups: Vec::new(),
        current_spoiler_group: SpoilerGroup::default(),
        price_range,
        random_progression: Bernoulli::new(RANDOM_PROGRESSION).unwrap(),
        rng,
    };

    one_xp(&mut world_contexts, &mut context)?;
    for world_index in 0..context.world_count {
        if let Some(amount) = world_contexts[world_index].world_tour {
            place_relics(amount, world_index, &mut world_contexts, &mut context)?;
        }
    }

    spawn_progressions(&mut world_contexts, &mut context)?;

    let mut reserved_slots = Vec::<(usize, &Node)>::with_capacity(RESERVE_SLOTS);

    loop {
        context.finalize_spoiler_group();

        let mut reach_context = progression_check(&mut world_contexts, &mut context)?;

        force_keystones(&reach_context.reachable_states, &mut reserved_slots, &mut world_contexts, &mut context)?;

        let mut needs_placement = (0..context.world_count).map(|world_index| {
            let world_reachable = &mut reach_context.reachable[world_index];
            let world_context = &mut world_contexts[world_index];

            world_reachable.retain(|&node| {
                let node_index = node.index();

                node.uber_state().map_or(false, |uber_state|
                    !world_context.placements.iter().any(|placement| &placement.uber_state == uber_state) &&
                    !world_context.placeholders.iter().any(|&placeholder| placeholder.index() == node_index) &&
                    !world_context.collected_preplacements.iter().any(|&collected| collected == node_index) &&
                    !reserved_slots.iter().any(|&(reserved_world, node)| reserved_world == world_index && node.index() == node_index)
                )
            });

            let identifiers: Vec<_> = world_reachable.iter()
                .filter_map(|&node| 
                    if node.can_place() {
                        Some(node.identifier())
                    } else { None })
                .collect();

            log::trace!("(World {}): {} Reachable free locations: {}", world_index, identifiers.len(), format_identifiers(identifiers));

            let mut world_needs_placement = Vec::with_capacity(world_reachable.len());

            for node in world_reachable {
                let preplaced = world_context.world.collect_preplacements(node.uber_state().unwrap());
                if preplaced {
                    world_context.collected_preplacements.push(node.index());
                } else if node.can_place() {
                    world_needs_placement.push(*node);
                }
            }

            let locations = world_needs_placement.iter().map(|node| node.identifier().to_owned()).collect::<Vec<_>>();
            context.current_spoiler_group.reachable.push(SpoilerWorldReachable { locations });

            world_needs_placement.append(&mut world_context.spawn_slots);
            world_needs_placement.shrink_to_fit();
            world_needs_placement.shuffle(context.rng);

            world_needs_placement
        }).collect::<Vec<_>>();

        if reach_context.unreached_count == 0 {
            log::trace!("All locations reached");

            for (world_index, reserved) in reserved_slots {
                world_contexts[world_index].placeholders.push(reserved);
            }

            place_remaining(&mut world_contexts, &mut context)?;

            context.finalize_spoiler_group();

            let (seed_worlds, spawns) = world_contexts.into_iter().zip(spawns)
                .map(|(world_context, spawn)| (
                    SeedWorld {
                        flags: Vec::new(),  // filled later
                        spawn,
                        placements: world_context.placements,
                        headers: String::new(),  // Filled later
                    },
                    world_context.spawn.identifier().to_string()))
                .unzip();
            let groups = context.spoiler_groups;

            let spoiler = SeedSpoiler { spawns, groups };
            return Ok((seed_worlds, spoiler));
        }

        if reserved_slots.len() < RESERVE_SLOTS {
            loop {
                let world_index = context.rng.gen_range(0..context.world_count);

                if let Some(node) = needs_placement[world_index].pop() {
                    reserved_slots.push((world_index, node));

                    if reserved_slots.len() == RESERVE_SLOTS {
                        break;
                    }
                } else {
                    break;
                }
            }

            reserved_slots.shuffle(context.rng);
        }

        let placement_count: usize = needs_placement.iter().map(Vec::len).sum();
        if placement_count > 0 {
            log::trace!("Placing {} items randomly, reserved {} for the next placement group", placement_count, reserved_slots.len());

            let mut total_placeholders = world_contexts.iter().map(|world_context| world_context.placeholders.len()).sum::<usize>();
            let mut any_random_placements = false;

            for (origin_world_index, world_needs_placement) in needs_placement.iter().enumerate() {
                for &node in world_needs_placement {
                    let allow_placeholder = total_placeholders < PLACEHOLDER_SLOTS;
                    if random_placement(origin_world_index, node, allow_placeholder, &mut world_contexts, &mut context)? {
                        any_random_placements |= true;
                    } else {
                        total_placeholders += 1;
                    }
                }
            }

            if any_random_placements { continue }
        }

        force_progression(&mut reserved_slots, &mut reach_context, &mut world_contexts, &mut context)?;
    }
}

fn build_world_contexts<'a, 'b>(worlds: Vec<World<'a, 'b>>, spawns: &[&'a Node], rng: &mut impl Rng) -> Result<Vec<WorldContext<'a, 'b>>, String> {
    let mut has_warned_about_tp_refill = false;

    worlds.into_iter().enumerate().map(|(world_index, mut world)| {
        world.collect_preplacements(&UberState::spawn());

        let mut placements = Vec::with_capacity(450);
        let mut spawn_slots = Vec::new();

        let spawn = spawns[world_index];
        let spawn_identifier = spawn.identifier();
        if spawn_identifier != DEFAULT_SPAWN {
            for _ in 0..3 {
                spawn_slots.push(&world.graph.spawn_pickup_node);
            }
            let mut message = Message::new(String::new());
            message.frames = Some(420);
            message.instant = true;
            placements.push(Placement {
                node: Some(&world.graph.spawn_pickup_node),
                uber_state: UberState::spawn(),
                item: Item::Message(message),
            });
        }

        let mut spawn_is_tp = false;
        // Remove spawn tp from the pool
        if let Some(spawn_tp) = match spawn_identifier {
            "MarshSpawn.Main" => Some(Item::Teleporter(Teleporter::Marsh)),
            "HowlsDen.Teleporter" => Some(Item::Teleporter(Teleporter::Den)),
            "EastHollow.Teleporter" => Some(Item::Teleporter(Teleporter::Hollow)),
            "GladesTown.Teleporter" => Some(Item::Teleporter(Teleporter::Glades)),
            "InnerWellspring.Teleporter" => Some(Item::Teleporter(Teleporter::Wellspring)),
            "MidnightBurrows.Teleporter" => Some(Item::Teleporter(Teleporter::Burrows)),
            "WoodsEntry.Teleporter" => Some(Item::Teleporter(Teleporter::WestWoods)),
            "WoodsMain.Teleporter" => Some(Item::Teleporter(Teleporter::EastWoods)),
            "LowerReach.Teleporter" => Some(Item::Teleporter(Teleporter::Reach)),
            "UpperDepths.Teleporter" => Some(Item::Teleporter(Teleporter::Depths)),
            "EastPools.Teleporter" => Some(Item::Teleporter(Teleporter::EastLuma)),
            "WestPools" => Some(Item::Teleporter(Teleporter::WestLuma)),
            "LowerWastes.WestTP" => Some(Item::Teleporter(Teleporter::WestWastes)),
            "LowerWastes.EastTP" => Some(Item::Teleporter(Teleporter::EastWastes)),
            "UpperWastes.NorthTP" => Some(Item::Teleporter(Teleporter::OuterRuins)),
            "WindtornRuins.RuinsTP" => Some(Item::Teleporter(Teleporter::InnerRuins)),
            "WillowsEnd.InnerTP" => Some(Item::Teleporter(Teleporter::Willow)),
            _ => None,
        } {
            spawn_is_tp = true;
            world.pool.inventory.remove(&spawn_tp, 1);
        }

        let spawn = spawns[world_index];
        // Add a teleport icon for fully random spawn
        if !spawn_is_tp {
            if let Some(position) = spawn.position().cloned() {
                if !has_warned_about_tp_refill && !world.player.settings.headers.iter().any(|header| header == "tp_refill") {
                    log::warn!("Spawning on non-teleporter locations without the tp_refill header is not recommended!");
                    has_warned_about_tp_refill = true;
                }

                let label = Some("Warp to Spawn".to_string());
                let item = Item::Command(Command::CreateWarp { id: 0, position, label });

                placements.push(Placement {
                    node: None,
                    uber_state: UberState::load(),
                    item,
                });
            } else {
                return Err(format!("Cannot spawn on {} which has no specified coordinates", spawn.identifier()));
            }
        }

        let world_tour = world.player.settings.goals.iter().find_map(|goal|
            match *goal {
                Goal::Relics(amount) => Some(amount),
                Goal::RelicChance(chance) => {
                    if chance == 0.0 { return Some(0); }
                    loop {
                        let amount = (0..11).filter(|_| rng.gen_bool(chance)).count();
                        if amount > 0 {
                            return Some(amount);
                        }
                    }
                },
                _ => None,
            }
        );

        let reachable_locations = total_reach_check(&world)?;

        let unreachable_locations = world.graph.nodes.iter()
            .filter(|&node|
                node.can_place() &&
                !reachable_locations.iter().any(|&reachable| reachable.index() == node.index()) &&
                !world.preplacements.iter().any(|(uber_state, _)| uber_state == node.uber_state().unwrap())
            ).collect::<Vec<_>>();
        if !unreachable_locations.is_empty() {
            let identifiers = unreachable_locations.iter().map(|&node| node.identifier()).collect::<Vec<_>>();
            if !(unreachable_locations.len() == 1 && world.player.settings.difficulty == Difficulty::Moki) {  // moki always has one unreachable pickup
                log::warn!("(World {}): {} locations are unreachable on these settings! These will only hold Spirit Light.", world_index, identifiers.len());
            }
            log::trace!("(World {}): Unreachable locations on these settings: {}", world_index, format_identifiers(identifiers));
        }

        let world_slots = world.graph.nodes.iter()
            .filter(|&node| {
                node.can_place() &&
                !world.preplacements.contains_key(node.uber_state().unwrap())
            })
            .count() - 1;  // 1 will be 1xp
        let mut spirit_light_slots = world_slots.saturating_sub(world.pool.inventory.item_count());
        if let Some(amount) = world_tour { spirit_light_slots -= amount; }
        log::trace!("(World {}): Estimated {}/{} slots for Spirit Light", world_index, spirit_light_slots, world_slots);

        let spirit_light_rng = SpiritLightAmounts::new(world.pool.spirit_light as f32, spirit_light_slots as f32, 0.75, 1.25);
        let random_spirit_light = Bernoulli::new(spirit_light_slots as f64 / world_slots as f64).unwrap();

        let shop_slots = world.graph.nodes.iter().filter(|&node|
            node.uber_state().map_or(false, |uber_state|
                uber_state.is_purchasable()
                && !world.preplacements.contains_key(uber_state)
        )).count();

        Ok(WorldContext {
            world,
            spawn,
            placements,
            placeholders: Vec::with_capacity(300),
            collected_preplacements: Vec::new(),
            spawn_slots,
            reachable_locations,
            unreachable_locations,
            spirit_light_rng,
            random_spirit_light,
            shop_slots,
            world_tour,
        })
    }).collect::<Result<Vec<_>, String>>()
}

fn pick_spawn<'a>(graph: &'a Graph, world_settings: &WorldSettings, rng: &mut impl Rng) -> Result<&'a Node, String> {
    let mut valid = graph.nodes.iter().filter(|node| node.can_spawn());
    let spawn = match &world_settings.spawn {
        Spawn::Random => valid
            .filter(|&node| {
                let identifier = node.identifier();
                if world_settings.difficulty >= Difficulty::Gorlek {
                    GORLEK_SPAWNS.contains(&identifier)
                } else {
                    MOKI_SPAWNS.contains(&identifier)
                }
            })
            .choose(rng)
            .ok_or_else(|| String::from("No valid spawn locations available"))?,
        Spawn::FullyRandom => valid
            .choose(rng)
            .ok_or_else(|| String::from("No valid spawn locations available"))?,
        Spawn::Set(spawn_loc) => valid
            .find(|&node| node.identifier() == spawn_loc)
            .ok_or_else(|| format!("Spawn {} not found", spawn_loc))?
    };
    Ok(spawn)
}

pub(super) fn generate_placements<'a, 'b>(
    graph: &'a Graph,
    worlds: &[World<'a, 'b>],
    rng: &mut impl Rng
) -> Result<(Vec<SeedWorld<'a>>, SeedSpoiler), String> {
    let mut index = 0;
    loop {
        let spawn_locs = worlds.iter()
            .map(|world| pick_spawn(graph, world.player.settings, rng))
            .collect::<Result<Vec<_>, String>>()?;
        let identifiers = spawn_locs.iter().map(|spawn_loc| spawn_loc.identifier()).collect::<Vec<_>>();
        log::trace!("Spawning on {}", identifiers.join(", "));

        match generate_placements_from_spawn(worlds.to_owned(), spawn_locs, rng) {
            Ok(seed) => {
                if index > 0 {
                    log::info!("Generated seed after {} tries{}", index + 1, if index < RETRIES / 2 { "" } else { " (phew)" });
                }

                return Ok(seed);
            },
            Err(err) => log::error!("{}\nRetrying...", err),
        }

        index += 1;
        if index == RETRIES {
            return Err(format!("All {} attempts to generate a seed failed :(", RETRIES));
        }
    };
}
