use regex::Regex;

use crate::uber_state::UberStateTrigger;
use crate::{util::Zone, world::Graph};

fn read_args(seed: &str, start_index: usize) -> Option<usize> {
    let mut depth: u8 = 1;
    for (index, byte) in seed[start_index..].bytes().enumerate() {
        if byte == b'(' {
            depth += 1;
        } else if byte == b')' {
            depth -= 1;
        }
        if depth == 0 {
            return Some(start_index + index);
        }
    }

    None
}

/// Create a new item matcher regex from the given pattern
///
/// This will require the pattern to match the entire item, excluding the optional pickup flag `|mute` at the end which will always be accepted
fn create_regex(pattern: &str) -> Result<Regex, String> {
    Regex::new(&format!(r"^({}(?:\|mute)?)$", pattern))
        .map_err(|err| format!("Invalid regex {}: {}", pattern, err))
}

fn where_is(
    pattern: &str,
    world_index: usize,
    seeds: &[String],
    graph: &Graph,
) -> Result<String, String> {
    let re = create_regex(pattern)?;

    for mut line in seeds[world_index].lines() {
        if let Some(index) = line.find("//") {
            line = &line[..index];
        }
        line = line.trim();

        if line.is_empty()
            || line.starts_with("Spawn:")
            || line.starts_with("Flags:")
            || line.starts_with("timer:")
        {
            continue;
        }

        let mut parts = line.splitn(3, '|');
        let uber_group = parts.next().unwrap();
        let uber_id = parts
            .next()
            .ok_or_else(|| format!("failed to read line {} in seed", line))?;
        let item = parts
            .next()
            .ok_or_else(|| format!("failed to read line {} in seed", line))?;

        if re.is_match(item) {
            if uber_group == "12" {
                // if multiworld shared
                let actual_item = format!(r"8\|12\|{}\|bool\|true", uber_id);

                let mut other_worlds = (0..seeds.len()).collect::<Vec<_>>();
                other_worlds.remove(world_index);

                for other_world_index in other_worlds {
                    let actual_zone = where_is(&actual_item, other_world_index, seeds, graph)?;
                    if &actual_zone != "Unknown" {
                        return Ok(format!("$[15|5|{}]'s {}", other_world_index, actual_zone));
                    }
                }
            } else if uber_group == "3" && (uber_id == "0" || uber_id == "1") {
                return Ok(String::from("Spawn"));
            } else if let Some(node) = graph.nodes.iter().find(|&node| {
                node.trigger().map_or(false, |trigger| {
                    trigger.code().to_string() == format!("{uber_group}|{uber_id}")
                })
            }) {
                if let Some(zone) = node.zone() {
                    return Ok(zone.to_string());
                }
            }
        }
    }

    Ok(String::from("Unknown"))
}

fn how_many(
    pattern: &str,
    zone: Zone,
    world_index: usize,
    seeds: &[String],
    graph: &Graph,
) -> Result<Vec<UberStateTrigger>, String> {
    let mut locations = Vec::new();
    let re = create_regex(pattern)?;

    for mut line in seeds[world_index].lines() {
        if let Some(index) = line.find("//") {
            line = &line[..index];
        }
        line = line.trim();

        if line.is_empty()
            || line.starts_with("Spawn:")
            || line.starts_with("Flags:")
            || line.starts_with("timer:")
        {
            continue;
        }

        let mut parts = line.splitn(3, '|');
        let uber_group = parts.next().unwrap();
        let uber_id = parts
            .next()
            .ok_or_else(|| format!("failed to read line {} in seed", line))?;
        let item = parts
            .next()
            .ok_or_else(|| format!("failed to read line {} in seed", line))?;

        if let Some(trigger) = graph.nodes.iter().find_map(|node| {
            if node.zone() == Some(zone) {
                let trigger = node.trigger();
                if trigger.map_or(false, |trigger| {
                    trigger.code().to_string() == format!("{uber_group}|{uber_id}")
                }) {
                    return trigger;
                }
            }
            None
        }) {
            if re.is_match(item) {
                locations.push(trigger.clone());
            } else {
                // if multiworld shared
                let mut item_parts = item.split('|');
                if item_parts.next() != Some("8") {
                    continue;
                }
                if item_parts.next() != Some("12") {
                    continue;
                }
                let share_id = item_parts.next().unwrap();
                let share_state = format!("12|{}|", share_id);

                let mut other_worlds = (0..seeds.len()).collect::<Vec<_>>();
                other_worlds.remove(world_index);

                'outer: for other_world_index in other_worlds {
                    let other_seed = &seeds[other_world_index];

                    for other_seed_line in other_seed.lines() {
                        if let Some(mut actual_item) = other_seed_line.strip_prefix(&share_state) {
                            if let Some(index) = actual_item.find("//") {
                                actual_item = &actual_item[..index];
                            }
                            actual_item = actual_item.trim();

                            if re.is_match(actual_item) {
                                locations.push(trigger.clone());
                                break 'outer;
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(locations)
}

pub fn postprocess(seeds: &mut [String], graph: &Graph) -> Result<(), String> {
    let clone = seeds.to_vec();

    for (world_index, seed) in seeds.iter_mut().enumerate() {
        let mut last_index = 0;
        loop {
            if let Some(mut start_index) = seed[last_index..].find("$WHEREIS(") {
                start_index += last_index;
                last_index = start_index;

                let after_bracket = start_index + 9;

                if let Some(end_index) = read_args(seed, after_bracket) {
                    let pattern = seed[after_bracket..end_index].trim();

                    let zone = where_is(pattern, world_index, &clone, graph)?;
                    seed.replace_range(start_index..=end_index, &zone);

                    continue;
                }
            }
            break;
        }

        last_index = 0;
        loop {
            if let Some(mut start_index) = seed[last_index..].find("$HOWMANY(") {
                start_index += last_index;
                last_index = start_index;

                let after_bracket = start_index + 9;

                if let Some(end_index) = read_args(seed, after_bracket) {
                    let mut args = seed[after_bracket..end_index].splitn(2, ',');
                    let zone = args.next().unwrap().trim();
                    let zone: u8 = zone
                        .parse()
                        .map_err(|_| format!("expected numeric zone, got {}", zone))?;
                    let zone =
                        Zone::try_from(zone).map_err(|_| format!("invalid zone {}", zone))?;
                    let pattern = args.next().unwrap_or("").trim();

                    let locations = how_many(pattern, zone, world_index, &clone, graph)?;
                    let locations = locations
                        .into_iter()
                        .map(|uber_state| uber_state.code().to_string())
                        .collect::<Vec<_>>();
                    let locations = locations.join(",").replace('|', ",");

                    let sysmessage = format!("$[15|4|{}]", locations);

                    seed.replace_range(start_index..=end_index, &sysmessage);

                    continue;
                }
            }
            break;
        }
    }

    Ok(())
}
