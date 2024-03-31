//! Parses state data from a csv format
//!
//! # Examples
//!
//! ```
//! use wotw_seedgen_assets::{StateData, StateDataEntry};
//! use wotw_seedgen_assets::data::UberIdentifier;
//!
//! let input = "
//! NodeIdentifier, UberGroup, UberId, UberStateValue
//! MarshSpawn.HowlBurnt, 21786, 25095, 1
//! ";
//! let state_data = StateData::from_reader(input.as_bytes()).unwrap();
//!
//! assert_eq!(state_data.entries, vec![
//!     StateDataEntry {
//!         identifier: "MarshSpawn.HowlBurnt".to_string(),
//!         uber_identifier: UberIdentifier::new(21786, 25095),
//!         value: Some(1),
//!     }
//! ]);
//! ```

use std::io;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use wotw_seedgen_data::UberIdentifier;

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StateData {
    pub entries: Vec<StateDataEntry>,
}
// TODO maybe a custom deserialize could eliminate the need for separate input/output structs?
/// Information about an obtainable world state
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateDataEntry {
    pub identifier: String,
    pub uber_identifier: UberIdentifier,
    pub value: Option<u8>,
}
impl PartialEq for StateDataEntry {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}
impl StateData {
    pub fn from_reader<R: io::Read>(reader: R) -> csv::Result<Self> {
        let entries = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(reader)
            .into_deserialize::<StateDataInput>()
            .map_ok(|input| StateDataEntry {
                identifier: input.node_identifier,
                uber_identifier: UberIdentifier::new(input.uber_group, input.uber_id),
                value: input.uber_state_value,
            })
            .collect::<Result<Vec<_>, _>>()?;

        Ok(Self { entries })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StateDataInput {
    node_identifier: String,
    uber_group: i32,
    uber_id: i32,
    uber_state_value: Option<u8>,
}
