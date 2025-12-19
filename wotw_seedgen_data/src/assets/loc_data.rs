//! Parses pickup location data from a csv format
//!
//! # Examples
//!
//! ```
//! use wotw_seedgen_data::{assets::{LocData, LocDataEntry}, MapIcon, Position, UberIdentifier, Zone};
//!
//! let input = "NodeIdentifier, Zone, PickupType, PickupDetails, UberGroup, UberId, UberStateValue, X, Y, MapX, MapY
//! MarshSpawn.RockHC, Marsh, HealthFragment, , 21786, 60210, , -958.6, -4313.2, -958.6199, -4312.245
//! GladesTown.MotayHutEX, Glades, SpiritLight, 100, 42178, 57455, , -172.7, -4583.2, -392.8, -4130.6";
//! let loc_data = LocData::from_reader(input.as_bytes()).unwrap();
//!
// TODO excuse me why is this equal?
//! assert_eq!(loc_data.entries, vec![
//!     LocDataEntry {
//!         identifier: "MarshSpawn.RockHC".to_string(),
//!         zone: Zone::Marsh,
//!         map_icon: MapIcon::HealthFragment,
//!         uber_identifier: UberIdentifier::new(21786, 60210),
//!         value: Some(1),
//!         position: Some(Position::new(-958., -4313.)),
//!         map_position: Some(Position::new(-958., -4313.)),
//!     },
//!     LocDataEntry {
//!         identifier: "GladesTown.MotayHutEX".to_string(),
//!         zone: Zone::Glades,
//!         map_icon: MapIcon::SpiritLight,
//!         uber_identifier: UberIdentifier::new(42178, 57455),
//!         value: Some(1),
//!         position: Some(Position::new(-172., -4584.)),
//!         map_position: Some(Position::new(-394., -4136.)),
//!     }
//! ]);
//! ```

use crate::{MapIcon, Position, UberIdentifier, Zone};
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, DisplayFromStr};
use std::io::Read;
use utoipa::ToSchema;

/// Information about all pickup locations which should be filled by the randomizer
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LocData {
    /// List of individual pickup locations
    pub entries: Vec<LocDataEntry>,
}

// TODO while breaking everything could also just change the loc data format to save this transformation
/// Information about a pickup location
#[derive(Debug, Clone, Serialize, Deserialize, ToSchema)]
pub struct LocDataEntry {
    /// Unique identifier for this pickup location which is used in `areas.wotw`
    pub identifier: String,
    /// Map zone containing this pickup location
    pub zone: Zone,
    /// Vanilla map icon
    pub map_icon: MapIcon,
    /// `UberIdentifier` where this pickup location's corresponding world state is stored
    ///
    /// pickup locations are either stored as booleans or as integers where being above a certain value means the pickup is collected
    pub uber_identifier: UberIdentifier,
    /// `None` if `uber_identifier` holds a boolean value. Otherwise, has the minimum integer value at which this pickup is collected
    pub value: Option<i32>,
    /// World coordinates of this pickup location, if applicable
    pub position: Option<Position>,
    /// Map coordinates of this pickup location, if applicable
    pub map_position: Option<Position>,
}

impl PartialEq for LocDataEntry {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}

impl LocData {
    /// Parse from a [`Read`] implementation, such as a file or byte slice
    ///
    /// Note that the underlying CSV reader is buffered automatically, so you should not
    /// wrap `reader` in a buffered reader like `io::BufReader`.
    pub fn from_reader<R: Read>(reader: R) -> Result<Self, String> {
        let mut entries = vec![];

        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(reader);
        let mut record = csv::StringRecord::new();

        while reader
            .read_record(&mut record)
            .map_err(|err| err.to_string())?
        {
            let record = record
                .deserialize::<LocDataInput>(None)
                .map_err(|err| err.to_string())?;

            let position = match (record.x, record.y) {
                (Some(x), Some(y)) => Some(Position::new(x, y)),
                _ => None,
            };

            let map_position = match (record.map_x, record.map_y) {
                (Some(x), Some(y)) => Some(Position::new(x, y)),
                _ => None,
            };

            entries.push(LocDataEntry {
                identifier: record.node_identifier,
                zone: record.zone,
                map_icon: record.pickup_type,
                uber_identifier: UberIdentifier::new(record.uber_group, record.uber_id),
                value: record.uber_state_value,
                position,
                map_position,
            })
        }
        Ok(Self { entries })
    }
}

#[serde_as]
#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LocDataInput<'a> {
    node_identifier: String,
    #[serde_as(as = "DisplayFromStr")]
    zone: Zone,
    #[serde_as(as = "DisplayFromStr")]
    pickup_type: MapIcon,
    _pickup_details: &'a str,
    uber_group: i32,
    uber_id: i32,
    uber_state_value: Option<i32>,
    x: Option<f32>,
    y: Option<f32>,
    map_x: Option<f32>,
    map_y: Option<f32>,
}
