//! Parses state data from a csv format
//!
//! # Examples
//!
//! ```
//! use wotw_seedgen_data::{assets::{StateData, StateDataEntry}, UberIdentifier};
//!
//! let input = "
//! Identifier, Condition
//! MarshSpawn.TeleporterActive, 21786|10185
//! ";
//! let state_data = StateData::from_reader(input.as_bytes()).unwrap();
//!
//! assert_eq!(state_data.entries, vec![
//!     StateDataEntry {
//!         identifier: "MarshSpawn.TeleporterActive".to_string(),
//!         uber_identifier: UberIdentifier::new(21786, 10185),
//!         value: None,
//!     }
//! ]);
//! ```

use std::io::Read;

use crate::{assets::LogicCondition, UberIdentifier};
use itertools::Itertools;
use serde::{Deserialize, Serialize};
use utoipa::ToSchema;

/// Information about all world states which are considered by the randomizer logic
///
/// Does not contain information about world states already present in [`LocData`]
///
/// [`Locdata`]: crate::LocData
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StateData {
    /// List of individual world states
    pub entries: Vec<StateDataEntry>,
}

// TODO maybe a custom deserialize could eliminate the need for separate input/output structs?
/// Information about an obtainable world state
#[derive(Debug, Clone, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct StateDataEntry {
    /// Unique identifier for this world state which is used in `paths.wotwl`
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
                identifier: input.identifier,
                uber_identifier: input.condition.uber_identifier,
                value: input.condition.value,
            })
            .collect::<Result<Vec<_>, _>>()
            .map_err(|err| err.to_string())?;

        Ok(Self { entries })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct StateDataInput {
    identifier: String,
    condition: LogicCondition,
}
