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

use std::io::Read;

use itertools::Itertools;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
use wotw_seedgen_data::UberIdentifier;

/// Information about all world states which are considered by the randomizer logic
///
/// Does not contain information about world states already present in [`LocData`]
///
/// [`Locdata`]: crate::LocData
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct StateData {
    /// List of individual world states
    pub entries: Vec<StateDataEntry>,
}

// TODO maybe a custom deserialize could eliminate the need for separate input/output structs?
/// Information about an obtainable world state
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct StateDataEntry {
    /// Unique identifier for this world state which is used in `areas.wotw`
    pub identifier: String,
    /// `UberIdentifier` where this world state is stored
    ///
    /// world states are either stored as booleans or as integers where being above a certain value means the world state is completed
    pub uber_identifier: UberIdentifier,
    /// `None` if `uber_identifier` holds a boolean value. Otherwise, has the minimum integer value at which this world state is completed
    pub value: Option<i32>,
}

impl PartialEq for StateDataEntry {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl StateData {
    /// Parse from a [`Read`] implementation, such as a file or byte slice
    ///
    /// Note that the underlying CSV reader is buffered automatically, so you should not
    /// wrap `reader` in a buffered reader like `io::BufReader`.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, String> {
        let entries = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(reader)
            .into_deserialize::<StateDataInput>()
            .map_ok(|input| StateDataEntry {
                identifier: input.node_identifier,
                uber_identifier: UberIdentifier::new(input.uber_group, input.uber_id),
                value: input.uber_state_value,
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;

        Ok(Self { entries })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StateDataInput {
    node_identifier: String,
    uber_group: i32,
    uber_id: i32,
    uber_state_value: Option<i32>,
}
