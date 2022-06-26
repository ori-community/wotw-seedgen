use std::fmt::{self, Display};
use prettytable::{format, row, cell, Table};

use serde::{Serialize, Deserialize};

use crate::{util::Position, Item};

#[cfg(doc)]
use super::placement::Placement;

/// Complete data to create a logic spoiler for the seed
#[derive(Serialize, Deserialize, Clone)]
pub struct SeedSpoiler {
    /// Metadata about the number of worlds and basic details about them
    pub worlds: Vec<SpoilerWorld>,
    /// Each [`SpoilerGroup`] represents one "step" of placements
    pub groups: Vec<SpoilerGroup>,
}
/// Basic details about a world
#[derive(Serialize, Deserialize, Clone)]
pub struct SpoilerWorld {
    /// User-given name for this world, or "World" as a default on single-world seeds
    pub name: String,
    /// Anchor identifier of the spawn location
    pub spawn: String,
}
/// One "step" of placements in a [`SeedSpoiler`]
#[derive(Default, Serialize, Deserialize, Clone)]
pub struct SpoilerGroup {
    /// Either contains the reachables for each world, or empty for placement groups before reachables are considered
    pub reachable: Vec<SpoilerWorldReachable>,
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

        writeln!(f)?;
        writeln!(f)?;

        let items_table_format = format::FormatBuilder::new()
            .column_separator(' ')
            .separators(&[], format::LineSeparator::default())
            .padding(2, 1)
            .indent(2)
            .build();

        for (index, spoiler_group) in self.groups.iter().enumerate() {
            write!(f, "Step {index}")?;

            if spoiler_group.reachable.is_empty() {
                writeln!(f, " (priority placements)")?;
            } else {
                writeln!(f)?;

                for (world_index, world_reachable) in spoiler_group.reachable.iter().enumerate() {
                    if multiworld {
                        write!(f, "  World {world_index}: ")?;
                    } else {
                        write!(f, "  ")?;
                    }

                    if world_reachable.locations.is_empty() {
                        writeln!(f, "No new locations reachable")?;
                    } else {
                        let count = world_reachable.locations.len();
                        write!(f, "{count} new location")?;
                        if count > 1 { write!(f, "s")?; }
                        writeln!(f, " reachable")?;
                    }
                }
            }

            let mut items_table = Table::new();
            items_table.set_format(items_table_format);

            let placement_count = spoiler_group.placements.len();
            if placement_count > 0 {
                write!(f, "  {placement_count} item")?;
                if placement_count > 1 { write!(f, "s")?; }
                writeln!(f, " placed")?;
                writeln!(f)?;

                for placement in &spoiler_group.placements {
                    let mut pickup = "".to_owned();
                    let mut location = "".to_owned();
                    let mut position = "".to_owned();

                    if multiworld {
                        let target_world = &self.worlds[placement.target_world_index].name;
                        pickup.push_str(format!("{target_world}'s ").as_str());
                    }

                    let item = &placement.item;
                    pickup.push_str(format!("{item}").as_str());

                    if placement.forced {
                        pickup.push_str(" [forced]")
                    }

                    if multiworld {
                        let origin_world = &self.worlds[placement.origin_world_index].name;
                        location.push_str(format!("{origin_world}'s ").as_str());
                    }

                    let node_identifier = &placement.node_identifier;
                    location.push_str(format!("{node_identifier}").as_str());

                    if let Some(node_position) = &placement.node_position {
                        position.push_str(format!("{node_position}").as_str());
                    }

                    items_table.add_row(row![
                        pickup,
                        location,
                        position,
                    ]);
                }

                writeln!(f, "{}", items_table.to_string())?;
            }

            writeln!(f)?;
        }

        Ok(())
    }
}
