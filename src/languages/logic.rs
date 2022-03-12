pub mod tokenizer;
pub mod parser;
pub mod emitter;

use std::{path::Path, convert::TryFrom};

use decorum::R32;
use parser::ParseError;
use serde::Deserialize;
use crate::world::graph::Graph;
use crate::Settings;
use crate::util::{self, UberState, Position};

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
    let locations = parse_locations(&input).map_err(|err| format!("Error parsing locations from {}: {}", locations.as_ref().display(), err))?;

    let input = util::read_file(&states, "logic")?;
    let state_map = parse_states(&input).map_err(|err| format!("Error parsing states from {}: {}", states.as_ref().display(), err))?;

    emitter::emit(&areas, &metadata, &locations, &state_map, settings, validate).map_err(|err| format!("Error building the logic: {}", err))
}

#[derive(Debug)]
pub struct Location {
    pub name: String,
    pub zone: String,
    pub uber_state: UberState,
    pub position: Position,
}

#[derive(Debug, Deserialize)]
struct LocationEntry<'a> {
    name: &'a str,
    zone: &'a str,
    _kind: &'a str,
    _variant: &'a str,
    _uber_group_name: &'a str,
    uber_group: &'a str,
    _uber_id_name: &'a str,
    uber_id: &'a str,
    x: f32,
    y: f32,
}

pub fn parse_locations(input: &str) -> Result<Vec<Location>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(input.as_bytes());

    let mut locations = Vec::with_capacity(389);

    let mut record = csv::StringRecord::new();
    while reader.read_record(&mut record).map_err(|err| err.to_string())? {
        let record: LocationEntry = record.deserialize(None).map_err(|err| err.to_string())?;

        let uber_state = UberState::from_parts(record.uber_group, record.uber_id)?;
        let x = R32::try_from(record.x).map_err(|err| format!("Invalid coordinate {}: {}", record.x, err))?;
        let y = R32::try_from(record.y).map_err(|err| format!("Invalid coordinate {}: {}", record.y, err))?;
        let position = Position { x, y };
        let location = Location { name: record.name.to_owned(), zone: record.zone.to_owned(), uber_state, position };

        locations.push(location);
    }

    Ok(locations)
}

#[derive(Debug)]
pub struct NamedState {
    pub name: String,
    pub uber_state: UberState,
}
#[derive(Debug, Deserialize)]
struct StateEntry {
    name: String,
    uber_group: String,
    uber_id: String,
}

pub fn parse_states(input: &str) -> Result<Vec<NamedState>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(input.as_bytes());

    let mut states = Vec::with_capacity(97);

    for result in reader.deserialize() {
        let record: StateEntry = result.map_err(|err| err.to_string())?;

        let uber_state = UberState::from_parts(&record.uber_group, &record.uber_id)?;
        let state = NamedState { name: record.name, uber_state };

        states.push(state);
    }

    Ok(states)
}
