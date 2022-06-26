use std::fmt::{self, Display};

use crate::{world::{graph::Node, Graph}, util::{constants::{DEFAULT_SPAWN, SPAWN_GRANTS}, UberState}, header, Settings};

use super::{spoiler::SeedSpoiler, Placement};

/// End Result of seed generation
pub struct Seed<'a> {
    /// Seed data per world
    pub worlds: Vec<SeedWorld<'a>>,
    /// The logic [`Graph`] used to generate the seed
    pub graph: &'a Graph,
    /// The [`Settings`] used to generate the seed
    pub settings: Settings,
    /// Spoiler data for the generation process
    pub spoiler: SeedSpoiler,
}
/// World-specific data related to a [`Seed`]
pub struct SeedWorld<'a> {
    /// Flags to summarize the seed configuration
    pub flags: Vec<String>,
    /// Starting location
    pub spawn: &'a Node,
    /// Generated [`Placement`]s
    pub placements: Vec<Placement<'a>>,
    /// Section that should be added as a result of headers
    pub headers: String,
}

impl Seed<'_> {
    /// Returns the seed files for each world to be used by the randomizer client
    /// 
    /// May error if postprocessing commands (such as `$WHEREIS`) contain invalid regexes
    pub fn seed_files(&self) -> Result<Vec<String>, String> {
        let mut seeds = self.worlds.iter().enumerate().map(|(index, world)| {
            let version = env!("CARGO_PKG_VERSION");
            let slug = &self.settings.slugify();
            let config = &self.settings.to_json();

            format!("{world}\
                #world-index: {index}\n\
                #target: ^2.0\n\
                #generator-version: {version}\n\
                #slug: {slug}\n\
                #config: {config}\n")
        }).collect::<Vec<_>>();

        header::parser::postprocess(&mut seeds, self.graph, &self.settings)?;

        Ok(seeds)
    }
}

impl Display for SeedWorld<'_> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if !self.flags.is_empty() {
            writeln!(f, "#flags: {}", self.flags.join(", "))?;
        }

        let spawn_identifier = self.spawn.identifier();
        if spawn_identifier != DEFAULT_SPAWN {
            let position = self.spawn.position().expect("Seed Spawn had no coordinates");
            writeln!(f, "setup 1|{}|{}  // Spawn at {spawn_identifier}", position.x, position.y)?;

            if let Some(spawn_item) = SPAWN_GRANTS.iter().find_map(|(spawn, item)| if *spawn == spawn_identifier { Some(item) } else { None }) {
                writeln!(f, "{}|{}|mute", UberState::spawn().code(), spawn_item.code())?;
            }
        }

        for placement in &self.placements {
            writeln!(f, "{}", placement.code())?;
        }

        write!(f, "{}", self.headers)?;

        if !self.headers.ends_with('\n') { writeln!(f,)?; }

        Ok(())
    }
}
