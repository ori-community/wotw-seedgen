use std::fmt::{self, Display, Write};

use serde::{Serialize, Deserialize};

use crate::{util::Position, Item, Inventory};

#[cfg(doc)]
use super::placement::Placement;

/// Complete data to create a logic spoiler for the seed
#[derive(Serialize, Deserialize, Clone)]
pub struct SeedSpoiler {
    /// Anchor identifier of all the spawn locations
    pub spawns: Vec<String>,
    /// Each [`SpoilerGroup`] represents one "step" of placements
    pub groups: Vec<SpoilerGroup>,
}
/// One "step" of placements in a [`SeedSpoiler`]
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SpoilerGroup {
    /// Either contains the reachables for each world, or empty for placement groups before reachables are considered
    pub reachable: Vec<SpoilerWorldReachable>,
    /// The set of items that were placed as forced progression, if any
    pub forced_items: Inventory,
    /// An ordered list detailing the [`Placement`]s
    pub placements: Vec<SpoilerPlacement>,
}
/// Newly reachable locations
#[derive(Serialize, Deserialize, Clone)]
pub struct SpoilerWorldReachable {
    pub locations: Vec<String>,
}
/// One item placed on one location
#[derive(Serialize, Deserialize, Clone)]
pub struct SpoilerPlacement {
    /// The "sending" world
    pub origin_world_index: usize,
    /// The "receiving" world
    pub target_world_index: usize,
    /// The identifier of the placement location
    pub node_identifier: String,
    /// The [`Position`] of the location, if applicable
    pub node_position: Option<Position>,
    /// The placed [`Item`]
    pub item: Item,
}

impl SeedSpoiler {
    /// Serialize into json format
    pub fn to_json(&self) -> String {
        // This is safe because the SeedSpoiler struct is known to serialize successfully
        serde_json::to_string(&self).unwrap()
    }
}

impl Display for SeedSpoiler {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let world_count = self.spawns.len();
        let multiworld = world_count > 1;

        if multiworld {
            for (index, spawn) in self.spawns.iter().enumerate() {
                writeln!(f, "Spawn for World [{index}]: {spawn}")?;
            }
        } else {
            let spawn = &self.spawns[0];
            writeln!(f, "Spawn: {spawn}")?;
        }

        writeln!(f)?;
        writeln!(f)?;

        let mut longest_pickup = 0;
        let mut longest_location = 0;

        let spoiler_groups = self.groups.iter().map(|spoiler_group| {
            let placements = spoiler_group.placements.iter().map(|placement| {
                let mut pickup = String::new();
                if multiworld {
                    write!(pickup, "[{}] ", placement.target_world_index)?;
                }
                write!(pickup, "{}", placement.item)?;
                if pickup.len() > longest_pickup { longest_pickup = pickup.len(); }

                let mut location  = String::new();
                if multiworld {
                    write!(location , "[{}] ", placement.origin_world_index)?;
                }
                write!(location, "{}", placement.node_identifier)?;
                if location.len() > longest_location { longest_location = location.len(); }

                Ok((pickup, location, &placement.node_position))
            }).collect::<Result<Vec<_>, _>>()?;
            Ok((&spoiler_group.reachable, &spoiler_group.forced_items, placements))
        }).collect::<Result<Vec<_>, _>>()?;

        for (index, (reachable, forced_items, placements)) in spoiler_groups.into_iter().enumerate() {
            write!(f, "Step {index}")?;

            if reachable.is_empty() {
                writeln!(f, " (priority placements)")?;
            } else {
                writeln!(f)?;

                for (world_index, world_reachable) in reachable.iter().enumerate() {
                    if multiworld {
                        write!(f, "  [{world_index}]: ")?;
                    } else {
                        write!(f, "  ")?;
                    }

                    if world_reachable.locations.is_empty() {
                        writeln!(f, "No new reachables")?;
                    } else {
                        let locations = world_reachable.locations.join(", ");
                        let count = world_reachable.locations.len();
                        write!(f, "{count} new reachable")?;
                        if count > 1 { write!(f, "s")?; }
                        writeln!(f, ": {locations}")?;
                    }
                }
            }

            if !forced_items.items.is_empty() {
                writeln!(f, "  Force placed: {forced_items}")?;
            }
            writeln!(f)?;

            let placement_count = placements.len();
            if placement_count > 0 {
                for (pickup, location, position) in placements {
                    write!(f, "    {pickup:<longest_pickup$}  ")?;
                    match position {
                        Some(position) => writeln!(f, "{location:<longest_location$}  {position}"),
                        None => writeln!(f, "{location}"),
                    }?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
