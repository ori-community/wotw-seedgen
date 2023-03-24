use std::fmt::{self, Display, Write};

use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    util::{Position, Zone},
    Inventory, Item,
};

/// Complete data to create a logic spoiler for the seed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeedSpoiler {
    /// Anchor identifier of all the spawn locations
    pub spawns: Vec<String>,
    /// Each [`SpoilerGroup`] represents one "step" of placements
    pub groups: Vec<SpoilerGroup>,
}
/// One "step" of placements in a [`SeedSpoiler`]
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpoilerGroup {
    /// Either contains the new reachables for each world, or empty for placement groups before reachables are considered
    pub reachable: Vec<Vec<NodeSummary>>,
    /// The set of items that were placed as forced progression, if any
    pub forced_items: Inventory,
    /// An ordered list detailing the [`Placement`]s
    /// 
    /// [`Placement`]: super::Placement
    pub placements: Vec<SpoilerPlacement>,
}
/// One item placed on one location
#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpoilerPlacement {
    /// The "sending" world
    pub origin_world_index: usize,
    /// The "receiving" world
    pub target_world_index: usize,
    /// The placement location
    pub location: NodeSummary,
    /// The placed [`Item`]
    pub item: Item,
    /// The name of the [`Item`], which may vary from the [`Item`]s [`Display`] implementation if a custom name for item was provided by headers
    pub item_name: String,
}
/// Select data from a [`Node`](crate::world::graph::Node)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeSummary {
    /// The identifier
    pub identifier: String,
    /// The [`Position`], if applicable
    pub position: Option<Position>,
    /// The [`Zone`], if applicable
    pub zone: Option<Zone>,
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
                write!(pickup, "{}", placement.item_name)?;
                if pickup.len() > longest_pickup { longest_pickup = pickup.len(); }

                let mut location  = String::new();
                if multiworld {
                    write!(location , "[{}] ", placement.origin_world_index)?;
                }
                write!(location, "{}", placement.location.identifier)?;
                if location.len() > longest_location { longest_location = location.len(); }

                Ok((pickup, location, &placement.location.position))
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

                    if world_reachable.is_empty() {
                        writeln!(f, "No new reachables")?;
                    } else {
                        let locations = world_reachable.iter().map(|node| &node.identifier).join(", ");
                        let count = world_reachable.len();
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
