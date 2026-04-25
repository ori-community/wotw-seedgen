use itertools::Itertools;
use serde::{Deserialize, Serialize};
use std::fmt::{self, Display, Write};
use wotw_seedgen_data::{assets::LocDataEntry, seed_language::output::CommandVoid, Position, Zone};

/// Complete data to create a logic spoiler for the seed
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SeedSpoiler {
    /// Anchor identifier of all the spawn locations
    pub spawns: Vec<String>,
    /// For each world, all the entrance connections
    ///
    /// If a world's list of entrance connections is empty, the entrances were not randomized
    pub entrances: Vec<Vec<(String, String)>>,
    /// An ordered list describing the preplaced items
    pub preplacements: Vec<SpoilerPlacement>,
    /// Each [`SpoilerGroup`] represents one "step" of placements
    pub groups: Vec<SpoilerGroup>,
}

impl SeedSpoiler {
    pub(super) fn new(spawns: Vec<String>, entrances: Vec<Vec<(String, String)>>) -> Self {
        Self {
            spawns,
            entrances,
            preplacements: vec![],
            groups: vec![],
        }
    }
}

/// One "step" of placements in a [`SeedSpoiler`]
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpoilerGroup {
    /// The new reachables for each world
    pub reachable: Vec<Vec<NodeSummary>>,
    // TODO grouped instead of repeating names?
    /// The set of items that were placed as forced progression, if any
    pub forced_items: Vec<SpoilerItem>,
    /// An ordered list describing the placed items
    pub placements: Vec<SpoilerPlacement>,
}

/// One item placed on one location
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpoilerPlacement {
    /// The "sending" world
    pub origin_world_index: usize,
    /// The "receiving" world
    pub target_world_index: usize,
    /// The placement location
    pub location: NodeSummary,
    /// The placed item
    pub item: SpoilerItem,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SpoilerItem {
    /// The placed command
    pub command: CommandVoid,
    /// The readable name of the placed item, which usually varies from the `command`s [`Display`] implementation
    pub name: String,
}

/// Select data from a [`LocDataEntry`]
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct NodeSummary {
    /// The identifier
    pub identifier: String,
    /// The [`Position`], if applicable
    pub position: Option<Position>,
    /// The [`Zone`]
    pub zone: Zone,
}

impl NodeSummary {
    pub(super) fn new(pickup: &LocDataEntry) -> Self {
        Self {
            identifier: pickup.identifier.clone(),
            position: pickup.position,
            zone: pickup.zone,
        }
    }

    pub(super) fn spawn() -> Self {
        Self {
            identifier: "Spawn".to_string(),
            position: None,
            zone: Zone::Spawn,
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

        writeln!(f)?;

        // TODO display loops of two more efficiently
        for (index, entrances) in self.entrances.iter().enumerate() {
            if !entrances.is_empty() {
                let longest_entrance = entrances.iter().map(|(from, _)| from.len()).max().unwrap();

                writeln!(f, "[{index}] Entrances:")?;

                for (from, to) in entrances {
                    writeln!(f, "  {from:<longest_entrance$} to {to}")?;
                }
            }
        }

        writeln!(f)?;

        type FormattedPlacements<'a> = Vec<(String, String, &'a Option<Position>)>;

        fn format_placements<'a>(
            placements: &'a [SpoilerPlacement],
            multiworld: bool,
            longest_item: &mut usize,
            longest_location: &mut usize,
        ) -> FormattedPlacements<'a> {
            placements
                .iter()
                .map(|placement| {
                    let mut item = String::new();

                    if multiworld {
                        let _ = write!(item, "[{}] ", placement.target_world_index);
                    }
                    let _ = write!(item, "{}", placement.item);

                    if item.len() > *longest_item {
                        *longest_item = item.len();
                    }

                    let mut location = String::new();

                    if multiworld {
                        let _ = write!(location, "[{}] ", placement.origin_world_index);
                    }
                    let _ = write!(location, "{}", placement.location.identifier);

                    if location.len() > *longest_location {
                        *longest_location = location.len();
                    }

                    (item, location, &placement.location.position)
                })
                .collect()
        }

        fn write_placements(
            f: &mut fmt::Formatter,
            placements: FormattedPlacements,
            longest_item: usize,
            longest_location: usize,
        ) -> fmt::Result {
            if !placements.is_empty() {
                for (item, location, position) in placements {
                    write!(f, "    {item:<longest_item$}  ")?;

                    match position {
                        Some(position) => writeln!(f, "{location:<longest_location$}  {position}")?,
                        None => writeln!(f, "{location}")?,
                    };
                }
            }

            Ok(())
        }

        let mut longest_item = 0;
        let mut longest_location = 0;

        let preplacements = format_placements(
            &self.preplacements,
            multiworld,
            &mut longest_item,
            &mut longest_location,
        );

        let spoiler_groups = self
            .groups
            .iter()
            .map(|spoiler_group| {
                let placements = format_placements(
                    &spoiler_group.placements,
                    multiworld,
                    &mut longest_item,
                    &mut longest_location,
                );

                (
                    &spoiler_group.reachable,
                    &spoiler_group.forced_items,
                    placements,
                )
            })
            .collect::<Vec<_>>();

        writeln!(f, "Preplacements")?;
        write_placements(f, preplacements, longest_item, longest_location)?;
        writeln!(f)?;

        for (index, (reachable, forced_items, placements)) in spoiler_groups.into_iter().enumerate()
        {
            writeln!(f, "Step {}", index + 1)?;

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
                writeln!(f, "  Force placed: {}", forced_items.iter().format(", "))?;
            }
            writeln!(f)?;

            write_placements(f, placements, longest_item, longest_location)?;

            writeln!(f)?;
        }

        Ok(())
    }
}

impl Display for SpoilerItem {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.name.fmt(f)
    }
}
