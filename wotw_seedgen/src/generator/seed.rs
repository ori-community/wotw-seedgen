use std::fmt::{self, Display};

use crate::uber_state::UberStateTrigger;
use crate::{
    header,
    settings::{UniverseSettings, WorldSettings},
    util::constants::{DEFAULT_SPAWN, SPAWN_GRANTS},
    world::{graph::Node, Graph},
};

use super::{spoiler::SeedSpoiler, Placement};

/// End Result of seed generation
pub struct Seed<'graph, 'settings> {
    /// Seed data per world
    pub worlds: Vec<SeedWorld<'graph, 'settings>>,
    /// The logic [`Graph`] used to generate the seed
    pub graph: &'graph Graph,
    /// The [`UniverseSettings`] used to generate the seed
    pub settings: &'settings UniverseSettings,
    /// Spoiler data for the generation process
    pub spoiler: SeedSpoiler,
}
/// World-specific data related to a [`Seed`]
pub struct SeedWorld<'graph, 'settings> {
    /// Flags to summarize the seed configuration
    pub flags: Vec<String>,
    /// Starting location
    pub spawn: &'graph Node,
    /// Generated [`Placement`]s
    pub placements: Vec<Placement<'graph>>,
    /// Section that should be added as a result of headers
    pub headers: String,
    /// Portion of the seed settings that belong to this world
    pub world_settings: &'settings WorldSettings,
}

impl Seed<'_, '_> {
    /// Returns the seed files for each world to be used by the randomizer client
    ///
    /// May error if postprocessing commands (such as `$WHEREIS`) contain invalid regexes
    pub fn seed_files(&self) -> Result<Vec<String>, String> {
        let mut seeds = self
            .worlds
            .iter()
            .enumerate()
            .map(|(index, world)| {
                let version = env!("CARGO_PKG_VERSION");
                let slug = &self.settings.slugify();
                let config = &self.settings.to_json();

                format!(
                    "\
                {world}\
                // This World: {index}\n\
                // Target: ^2.0\n\
                // Generator Version: {version}\n\
                // Slug: {slug}\n\
                // Config: {config}\n\
            "
                )
            })
            .collect::<Vec<_>>();

        header::parser::postprocess(&mut seeds, self.graph)?;

        Ok(seeds)
    }
}

impl Display for SeedWorld<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.flags.is_empty() {
            writeln!(f, "Flags: {}", self.flags.join(", "))?;
        }

        let spawn_identifier = self.spawn.identifier();
        if spawn_identifier != DEFAULT_SPAWN {
            let position = self
                .spawn
                .position()
                .expect("Seed Spawn had no coordinates");
            writeln!(
                f,
                "Spawn: {}, {}  // {}",
                position.x, position.y, spawn_identifier
            )?;

            if let Some(spawn_item) = SPAWN_GRANTS.iter().find_map(|(spawn, item)| {
                if *spawn == spawn_identifier {
                    Some(item)
                } else {
                    None
                }
            }) {
                writeln!(
                    f,
                    "{}|{}|mute",
                    UberStateTrigger::spawn().code(),
                    spawn_item.code()
                )?;
            }
        }

        for placement in &self.placements {
            writeln!(f, "{}", placement.code())?;
        }

        write!(f, "{}", self.headers)?;

        if !self.headers.ends_with('\n') {
            writeln!(f,)?;
        }

        Ok(())
    }
}
