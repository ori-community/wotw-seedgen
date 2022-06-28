use std::fmt::{self, Display};

use serde::{Serialize, Deserialize};

use crate::{util::Position, Item};

#[cfg(doc)]
use super::placement::Placement;

/// Complete data to create a logic spoiler for the seed
#[derive(Serialize, Deserialize)]
pub struct SeedSpoiler {
    /// Anchor identifier of all the spawn locations
    pub spawns: Vec<String>,
    /// Each [`SpoilerGroup`] represents one "step" of placements
    pub groups: Vec<SpoilerGroup>,
}
/// One "step" of placements in a [`SeedSpoiler`]
#[derive(Default, Serialize, Deserialize)]
pub struct SpoilerGroup {
    /// Either contains the reachables for each world, or empty for placement groups before reachables are considered
    pub reachable: Vec<SpoilerWorldReachable>,
    /// An ordered list detailing the [`Placement`]s
    pub placements: Vec<SpoilerPlacement>,
}
/// Newly reachable locations
#[derive(Serialize, Deserialize)]
pub struct SpoilerWorldReachable {
    pub locations: Vec<String>,
}
/// One item placed on one location
#[derive(Serialize, Deserialize)]
pub struct SpoilerPlacement {
    /// Whether this placement happened as a part of forced progression (as opposed to random placement)
    pub forced: bool,
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
                write!(f, "World {index}'s Spawn: {spawn}\n")?;
            }
        } else {
            let spawn = &self.spawns[0];
            write!(f, "Spawn: {spawn}\n")?;
        }

        for (index, spoiler_group) in self.groups.iter().enumerate() {
            write!(f, "Step {index}")?;

            if spoiler_group.reachable.is_empty() {
                writeln!(f, " - Priority placements")?;
            } else {
                if multiworld { writeln!(f)?; }
                else { write!(f, " - ")?; }
                for (world_index, world_reachable) in spoiler_group.reachable.iter().enumerate() {
                    if multiworld {
                        write!(f, "World {world_index}: ")?;
                    }
                    if world_reachable.locations.is_empty() {
                        writeln!(f, "No new locations reachable")?;
                    } else {
                        let locations = world_reachable.locations.join(", ");
                        let count = world_reachable.locations.len();
                        write!(f, "{count} new location")?;
                        if count > 1 { write!(f, "s")?; }
                        writeln!(f, " reachable: {locations}")?;
                    }
                }
            }

            let placement_count = spoiler_group.placements.len();
            if placement_count > 0 {
                write!(f, "{placement_count} item")?;
                if placement_count > 1 { write!(f, "s")?; }
                writeln!(f, " placed:")?;

                for placement in &spoiler_group.placements {
                    if placement.forced { write!(f, "(forced) ")?; }

                    if multiworld {
                        let target_world_index = &placement.target_world_index;
                        write!(f, "World {target_world_index}'s ")?;
                    }

                    let item = &placement.item;
                    write!(f, "{item} from ")?;

                    if multiworld {
                        let origin_world_index = &placement.origin_world_index;
                        write!(f, "World {origin_world_index}'s ")?;
                    }

                    let node_identifier = &placement.node_identifier;
                    write!(f, "{node_identifier}")?;
                    if let Some(node_position) = &placement.node_position {
                        write!(f, " at {node_position}")?;
                    }
                    writeln!(f)?;
                }
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
