use serde::Deserialize;

use crate::uber_state::{UberIdentifier, UberStateTrigger, UberStateCondition, UberStateComparator};

/// Information about an obtainable world state
#[derive(Debug, Clone, PartialEq)]
pub struct NamedState {
    pub name: String,
    pub trigger: UberStateTrigger,
}
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StateEntry {
    node_identifier: String,
    uber_group: u16,
    uber_id: u16,
    uber_state_value: u32,
}

/// Parses state data from a csv format
/// 
/// # Examples
/// 
/// ```
/// # use wotw_seedgen::logic::{parse_states, NamedState};
/// use wotw_seedgen::uber_state::UberStateTrigger;
/// 
/// let input = "
/// NodeIdentifier, UberGroup, UberId, UberStateValue
/// MarshSpawn.HowlBurnt, 21786, 25095, 1
/// ";
/// let states = parse_states(input).unwrap();
/// 
/// assert_eq!(states, vec![
///     NamedState {
///         name: "MarshSpawn.HowlBurnt".to_string(),
///         trigger: "21786|25095>=1".parse().unwrap(),
///     }
/// ]);
/// ```
pub fn parse_states(input: &str) -> Result<Vec<NamedState>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(input.as_bytes());

    let states = reader.deserialize().map(|record| {
        let StateEntry { node_identifier, uber_group, uber_id, uber_state_value } = record.map_err(|err| err.to_string())?;

        let identifier = UberIdentifier::new(uber_group, uber_id);
        let condition = Some(UberStateCondition { comparator: UberStateComparator::GreaterOrEquals, value: uber_state_value });
        let trigger = UberStateTrigger { identifier, condition };
        Ok(NamedState { name: node_identifier, trigger })
    }).collect::<Result<_, String>>()?;

    Ok(states)
}
