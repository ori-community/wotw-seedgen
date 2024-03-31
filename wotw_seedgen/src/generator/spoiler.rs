use crate::inventory::Inventory;
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Write};
use wotw_seedgen_data::{Position, Zone};
use wotw_seedgen_logic_language::output::Node;
use wotw_seedgen_seed_language::output::CommandVoid;

/// Complete data to create a logic spoiler for the seed
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct SeedSpoiler {
    /// Anchor identifier of all the spawn locations
    pub spawns: Vec<String>,
    /// An ordered list describing the preplaced items
    pub preplacements: Vec<SpoilerPlacement>,
    /// Each [`SpoilerGroup`] represents one "step" of placements
    pub groups: Vec<SpoilerGroup>,
}
impl SeedSpoiler {
    pub(super) fn new(spawns: Vec<String>) -> Self {
        Self {
            spawns,
            preplacements: vec![],
            groups: vec![],
        }
    }
}
/// One "step" of placements in a [`SeedSpoiler`]
#[derive(Debug, Default, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct SpoilerGroup {
    /// The new reachables for each world
    pub reachable: Vec<Vec<NodeSummary>>,
    /// The set of items that were placed as forced progression, if any
    pub forced_items: Inventory,
    /// An ordered list describing the placed items
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
    /// The placed command
    pub command: CommandVoid,
    /// The readable name of the placed item, which usually varies from the `command`s [`Display`] implementation
    pub item_name: String,
}
/// Select data from a [`Node`](crate::logic_language::output::Node)
#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct NodeSummary {
    /// The identifier
    pub identifier: String,
    /// The [`Position`], if applicable
    pub position: Option<Position>,
    /// The [`Zone`], if applicable
    pub zone: Option<Zone>,
}
impl NodeSummary {
    pub(super) fn new(node: &Node) -> Self {
        Self {
            identifier: node.identifier().to_string(),
            position: node.position().copied(),
            zone: node.zone(),
        }
    }
    pub(super) fn spawn() -> Self {
        Self {
            identifier: "Spawn".to_string(),
            position: None,
            zone: Some(Zone::Spawn),
        }
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

        write!(f, "\n\n")?;

        if !self.preplacements.is_empty() {
            writeln!(f, "Preplacements")?;
            // TODO write preplacements, the code below looks confusing, maybe improve it first
        }

        let mut longest_pickup = 0;
        let mut longest_location = 0;

        let spoiler_groups = self
            .groups
            .iter()
            .map(|spoiler_group| {
                let placements = spoiler_group
                    .placements
                    .iter()
                    .map(|placement| {
                        let mut pickup = String::new();
                        if multiworld {
                            write!(pickup, "[World {}] ", placement.target_world_index)?;
                        }
                        write!(pickup, "{}", placement.item_name)?;
                        if pickup.len() > longest_pickup {
                            longest_pickup = pickup.len();
                        }

                        let mut location = String::new();
                        if multiworld {
                            write!(location, "[World {}] ", placement.origin_world_index)?;
                        }
                        write!(location, "{}", placement.location.identifier)?;
                        if location.len() > longest_location {
                            longest_location = location.len();
                        }

                        Ok((pickup, location, &placement.location.position))
                    })
                    .collect::<Result<Vec<_>, _>>()?;
                Ok((
                    &spoiler_group.reachable,
                    &spoiler_group.forced_items,
                    placements,
                ))
            })
            .collect::<Result<Vec<_>, _>>()?;

        for (index, (reachable, forced_items, placements)) in spoiler_groups.into_iter().enumerate()
        {
            writeln!(f, "Step {index}")?;

            for (world_index, world_reachable) in reachable.iter().enumerate() {
                if multiworld {
                    write!(f, "  [{world_index}]: ")?;
                } else {
                    write!(f, "  ")?;
                }

                if world_reachable.is_empty() {
                    writeln!(f, "No new reachables")?;
                } else {
                    let locations = world_reachable
                        .iter()
                        .map(|node| &node.identifier)
                        .format(", ");
                    let count = world_reachable.len();
                    write!(f, "{count} new reachable")?;
                    if count > 1 {
                        write!(f, "s")?;
                    }
                    writeln!(f, ": {locations}")?;
                }
            }

            if !forced_items.is_empty() {
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
