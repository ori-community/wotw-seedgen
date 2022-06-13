use std::fmt::{self, Display};

use serde::{Serialize, Deserialize};

use crate::{util::Position, Item};

#[cfg(doc)]
use super::placement::Placement;

/// Complete data to create a logic spoiler for the seed
#[derive(Serialize, Deserialize)]
pub struct SeedSpoiler {
    /// Metadata about the number of worlds and basic details about them
    pub worlds: Vec<SpoilerWorld>,
    /// Each [`SpoilerGroup`] represents one "step" of placements
    pub groups: Vec<SpoilerGroup>,
}
/// Basic details about a world
#[derive(Serialize, Deserialize)]
pub struct SpoilerWorld {
    /// User-given name for this world, or "World" as a default on single-world seeds
    pub name: String,
    /// Anchor identifier of the spawn location
    pub spawn: String,
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
        let world_count = self.worlds.len();
        let multiworld = world_count > 1;

        if multiworld {
            for SpoilerWorld { name, spawn } in &self.worlds {
                writeln!(f, "{name}'s Spawn: {spawn}")?;
            }
        } else {
            let spawn = &self.worlds[0].spawn;
            writeln!(f, "Spawn: {spawn}")?;
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
                        let world_name = &self.worlds[world_index].name;
                        write!(f, "{world_name}: ")?;
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
                        let target_world = &self.worlds[placement.target_world_index].name;
                        write!(f, "{target_world}'s ")?;
                    }

                    let item = &placement.item;
                    write!(f, "{item} from ")?;

                    if multiworld {
                        let origin_world = &self.worlds[placement.origin_world_index].name;
                        write!(f, "{origin_world}'s ")?;
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
