mod emitter;
mod locations;
mod parser;
mod states;
mod tokenizer;

pub use emitter::build;
pub use locations::{parse_locations, Location};
pub use parser::Areas;
pub use states::{parse_states, NamedState};

use crate::settings::UniverseSettings;
use crate::world::Graph;

/// Convenience function to perform all steps of parsing and building the logic in one call
///
/// For more details, check the individual steps contained in this module
pub fn parse_logic(
    areas: &str,
    locations: &str,
    states: &str,
    universe_settings: &UniverseSettings,
    validate: bool,
) -> Result<Graph, String> {
    let areas = Areas::parse(areas).map_err(|err| err.verbose_display())?;
    let locations = parse_locations(locations)?;
    let named_states = parse_states(states)?;
    build(areas, locations, named_states, universe_settings, validate)
}
