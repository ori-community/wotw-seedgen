mod seed;
mod spoiler;
mod placement;

pub use seed::{Seed, SeedWorld};
pub use spoiler::{SeedSpoiler, SpoilerGroup, SpoilerWorldReachable, SpoilerPlacement};
pub use placement::Placement;

use std::{fmt::Write, cmp::Ordering};

use rand::{Rng, prelude::StdRng};
use rand_seeder::Seeder;
use rustc_hash::{FxHashMap, FxHashSet};

use crate::item::{Item, Message, UberStateOperator};
use crate::settings::{UniverseSettings, InlineHeader, HeaderConfig, Goal};
use crate::uber_state::UberStateTrigger;
use crate::world::{World, Graph, Pool};
use crate::header::{self, Header, HeaderBuild};
use crate::files::FileAccess;

use placement::generate_placements;

pub fn generate_seed<'graph, 'settings>(graph: &'graph Graph, file_access: &impl FileAccess, settings: &'settings UniverseSettings) -> Result<Seed<'graph, 'settings>, String> {
    let mut rng: StdRng = Seeder::from(&settings.seed).make_rng();
    log::trace!("Seeded RNG with {}", settings.seed);

    let (worlds, (flags, headers)): (Vec<_>, (Vec<_>, Vec<_>)) = settings.world_settings.iter().map(|world_settings| {
        let mut world = World::new_spawn(graph, world_settings);
        world.pool = Pool::preset();

        let (goals, flags, headers) = parse_headers(&mut world, file_access, &mut rng)?;
        world.goals = goals;

        Ok((world, (flags, headers)))
    }).collect::<Result<Vec<_>, String>>()?.into_iter().unzip();

    let (mut worlds, spoiler) = generate_placements(graph, &worlds, &mut rng)?;

    for ((world, flags), headers) in worlds.iter_mut().zip(flags).zip(headers) {
        world.flags = flags;
        world.headers = headers;
    }

    Ok(Seed { worlds, graph, settings, spoiler })
}

fn parse_headers(world: &mut World, file_access: &impl FileAccess, rng: &mut impl Rng) -> Result<(Vec<Goal>, Vec<String>, String), String> {
    validate_header_names(&world.player.settings.headers, &world.player.settings.inline_headers)?;

    let mut config_map = build_config_map(&world.player.settings.header_config)?;

    let mut headers = vec![];
    let mut includes = FxHashSet::default();
    includes.extend(world.player.settings.headers.iter().cloned());

    for header_name in &world.player.settings.headers {
        let header = file_access.read_header(header_name)?;
        parse_header(header_name.clone(), header, &mut headers, &mut includes, &mut config_map, file_access, rng)?;
    }

    for inline_header in &world.player.settings.inline_headers {
        let header_name = inline_header.name.as_ref().cloned().unwrap_or_else(|| "Anonymous Header".to_string());
        let header = inline_header.content.clone();
        parse_header(header_name, header, &mut headers, &mut includes, &mut config_map, file_access, rng)?;
    }

    let mut excludes = FxHashMap::default();
    let mut seed_contents = String::new();
    let mut flags = vec![];
    let mut goals = vec![];
    let mut state_sets = vec![];

    flags.push(world.player.settings.difficulty.to_string());
    if !world.player.settings.tricks.is_empty() { flags.push("Glitches".to_string()); }
    if world.player.settings.is_random_spawn() { flags.push("Random Spawn".to_string()); }
    if world.player.settings.hard { flags.push("Hard".to_string()); }

    let header_names = headers.into_iter().map(|(header_name, mut header)| {
        for exclude in header.excludes {
            excludes.insert(exclude, header_name.clone());
        }

        if !header.seed_content.is_empty() {
            seed_contents.push_str(&header.seed_content);
            seed_contents.push('\n');
        }

        flags.append(&mut header.flags);
        goals.append(&mut header.goals);

        for preplacement in header.preplacements {
            block_spawn_sets(&preplacement, world);
            header.item_pool_changes.entry(preplacement.item.clone()).and_modify(|prior| *prior -= 1).or_insert(-1);
            world.preplace(preplacement.trigger, preplacement.item);
        }

        for (item, amount) in header.item_pool_changes {
            #[allow(clippy::cast_sign_loss)]
            match amount.cmp(&0) {
                Ordering::Less => world.pool.remove(&item, (-amount) as u32),
                Ordering::Equal => {},
                Ordering::Greater => world.pool.grant(item, amount as u32),
            }
        }

        for (item, details) in header.item_details {
            let display = item.to_string();
            if world.custom_items.insert(item, details).is_some() {
                return Err(format!("multiple headers tried to customize the item {display}"));
            }
        }

        state_sets.append(&mut header.state_sets);

        Ok(header_name)
    }).collect::<Result<Vec<_>, _>>()?;

    for header_name in &header_names {
        if let Some(other) = excludes.get(header_name) {
            return Err(format!("headers {other} and {header_name} are incompatible"));
        }
    }
    for header_with_parameters in config_map.keys() {
        if !header_names.iter().any(|header| header == header_with_parameters) {
            log::warn!("The header {header_with_parameters} referenced in a header argument isn't active");
        }
    }

    goals.extend(world.player.settings.goals.iter().cloned());
    
    for flag in goals.iter().map(Goal::flag_name) {
        flags.push(flag.to_string());
    }

    let mut header_block = String::new();

    write!(header_block, "{seed_contents}").unwrap();
    if !state_sets.is_empty() { writeln!(header_block, "// Sets: {}", state_sets.join(", ")).unwrap(); }

    Ok((goals, flags, header_block))
}

fn parse_header(
    header_name: String,
    header: String,
    headers: &mut Vec<(String, HeaderBuild)>,
    includes: &mut FxHashSet<String>,
    config_map: &mut FxHashMap::<String, FxHashMap<String, String>>,
    file_access: &impl FileAccess,
    rng: &mut impl Rng
) -> Result<(), String> {
    log::trace!("Parsing header {header_name}");

    let header_config = config_map.remove(&header_name).unwrap_or_default();

    let header = Header::parse(header, rng)
        .map_err(|err| format!("Error in header {}:\n{}", header_name, err.verbose_display()))?
        .build(header_config)?;

    for include in &header.includes {
        if includes.insert(include.clone()) {
            let header = file_access.read_header(include)?;
            parse_header(include.clone(), header, headers, includes, config_map, file_access, rng)?;
        }
    }

    headers.push((header_name, header));

    Ok(())
}

/// verifies that inline headers don't claim names already in use
fn validate_header_names(headers: &FxHashSet<String>, inline_headers: &[InlineHeader]) -> Result<(), String> {
    for inline_header in inline_headers {
        if let Some(name) = &inline_header.name {
            if headers.contains(name) {
                return Err(format!("Ambiguous name: {name} used both as a file header and an inline header"));
            }
        }
    }

    Ok(())
}

fn build_config_map(header_config: &[HeaderConfig]) -> Result<FxHashMap::<String, FxHashMap<String, String>>, String> {
    let mut config_map = FxHashMap::<String, FxHashMap<_, _>>::default();

    for config in header_config {
        if let Some(prior) = config_map.entry(config.header_name.clone())
            .or_default()
            .insert(config.config_name.clone(), config.config_value.clone())
        {
            if prior != config.config_value {
                return Err(format!("provided multiple values for configuration parameter {} for header {} ({} and {})", config.config_name, config.header_name, prior, config.config_value));
            }
        }
    }

    Ok(config_map)
}

fn block_spawn_sets(preplacement: &header::Pickup, world: &mut World) {
    if let Item::UberState(uber_state_item) = &preplacement.item {
        if preplacement.trigger != UberStateTrigger::spawn() { return }
        if let UberStateOperator::Value(value) = &uber_state_item.operator {
            for trigger in world.graph.nodes.iter()
                .filter(|node| node.can_place())
                .filter_map(|node| node.trigger())
                .filter(|trigger| trigger.check(uber_state_item.identifier, value.to_f32()))
            {
                log::trace!("adding an empty pickup at {} to prevent placements", trigger.code());
                let mut message = Message::new(String::new());
                message.frames = Some(0);
                message.quiet = true;
                message.noclear = true;
                let null_item = Item::Message(message);
                world.preplace(trigger.clone(), null_item);
            }
        }
    }
}
