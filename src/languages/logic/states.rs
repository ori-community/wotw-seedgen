use serde::Deserialize;
use crate::util::UberState;

/// Information about an obtainable world state
#[derive(Debug, Clone, PartialEq)]
pub struct NamedState {
    pub name: String,
    pub uber_state: UberState,
}
#[derive(Deserialize)]
struct StateEntry<'a> {
    name: String,
    uber_group: &'a str,
    uber_id: &'a str,
}

/// Parses state data from a csv format
/// 
/// # Examples
/// 
/// ```
/// # use seedgen::logic::{parse_states, NamedState};
/// use seedgen::util::UberState;
/// 
/// let input = "MarshSpawn.HowlBurnt, 21786, 25095";
/// let states = parse_states(input).unwrap();
/// 
/// assert_eq!(states, vec![
///     NamedState {
///         name: "MarshSpawn.HowlBurnt".to_string(),
///         uber_state: UberState::from_parts("21786", "25095").unwrap(),
///     }
/// ]);
/// ```
pub fn parse_states(input: &str) -> Result<Vec<NamedState>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .has_headers(false)
        .trim(csv::Trim::All)
        .from_reader(input.as_bytes());

    let mut states = Vec::with_capacity(97);

    let mut record = csv::StringRecord::new();
    while reader.read_record(&mut record).map_err(|err| err.to_string())? {
        let record = record.deserialize(None).map_err(|err| err.to_string())?;
        let StateEntry { name, uber_group, uber_id } = record;

        let uber_state = UberState::from_parts(uber_group, uber_id)?;
        let state = NamedState { name, uber_state };

        states.push(state);
    }

    Ok(states)
}
