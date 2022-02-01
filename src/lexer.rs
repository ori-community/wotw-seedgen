pub mod tokenizer;
pub mod parser;
pub mod emitter;

use std::path::Path;

use parser::ParseError;
use crate::world::graph::Graph;
use crate::settings::Settings;
use crate::util;

pub fn parse_logic<P1, P2, P3>(areas: P1, locations: P2, states: P3, settings: &Settings, validate: bool) -> Result<Graph, String>
where
    P1: AsRef<Path>,
    P2: AsRef<Path>,
    P3: AsRef<Path>,
{
    let input = util::read_file(&areas, "logic")?;
    let (tokens, metadata) = tokenizer::tokenize(&input).map_err(|err| format!("Error parsing areas from {}: {}", areas.as_ref().display(), err))?;

    let areas = parser::parse_areas(tokens, &metadata).map_err(|err| {
        let ParseError { description, position } = err;
        let line = if position == usize::MAX {
            input.lines().last()
        } else {
            input[position..].lines().next()
        }.unwrap_or("");
        format!("Error parsing areas.wotw: {}: {}", description, line)
    })?;

    let input = util::read_file(&locations, "logic")?;
    let locations = parser::parse_locations(&input).map_err(|err| format!("Error parsing locations from {}: {}", locations.as_ref().display(), err))?;

    let input = util::read_file(&states, "logic")?;
    let state_map = parser::parse_states(&input).map_err(|err| format!("Error parsing states from {}: {}", states.as_ref().display(), err))?;

    emitter::emit(&areas, &metadata, &locations, &state_map, settings, validate).map_err(|err| format!("Error building the logic: {}", err))
}
