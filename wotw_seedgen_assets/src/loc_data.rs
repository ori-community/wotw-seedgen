//! Parses pickup location data from a csv format
//!
//! # Examples
//!
//! ```
//! use wotw_seedgen_assets::{LocData, LocDataEntry};
//! use wotw_seedgen_assets::data::{UberIdentifier, Position, Zone};
//!
//! let input = "NodeIdentifier, Zone, PickupType, PickupDetails, UberGroupName, UberGroup, UberIdName, UberId, UberStateValue, X, Y, MapX, MapY
//! MarshSpawn.RockHC, Inkwater Marsh, Resource, Life, swampStateGroup, 21786, healthContainerA, 60210, , -958.6, -4313.2, -958.6199, -4312.245
//! GladesTown.MotayHutEX, Wellspring Glades, SpiritLight, 100, hubUberStateGroup, 42178, hutCExpOrb, 57455, , -172.7, -4583.2, -392.8, -4130.6";
//! let loc_data = LocData::from_reader(input.as_bytes()).unwrap();
//!
//! assert_eq!(loc_data.entries, vec![
//!     LocDataEntry {
//!         identifier: "MarshSpawn.RockHC".to_string(),
//!         zone: Zone::Inkwater,
//!         uber_identifier: UberIdentifier::new(21786, 60210),
//!         value: Some(1),
//!         position: Some(Position::new(-958., -4313.)),
//!         map_position: Some(Position::new(-958., -4313.)),
//!     },
//!     LocDataEntry {
//!         identifier: "GladesTown.MotayHutEX".to_string(),
//!         zone: Zone::Glades,
//!         uber_identifier: UberIdentifier::new(42178, 57455),
//!         value: Some(1),
//!         position: Some(Position::new(-172., -4584.)),
//!         map_position: Some(Position::new(-394., -4136.)),
//!     }
//! ]);
//! ```

use serde::{Deserialize, Serialize};
use std::io;
use wotw_seedgen_data::{Position, UberIdentifier, Zone};

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct LocData {
    pub entries: Vec<LocDataEntry>,
}
// TODO while breaking everything could also just change the loc data format to save this transformation
/// Information about a pickup location
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocDataEntry {
    pub identifier: String,
    pub zone: Zone,
    pub uber_identifier: UberIdentifier,
    pub value: Option<u8>,
    pub position: Option<Position>,
    pub map_position: Option<Position>,
}
impl PartialEq for LocDataEntry {
    fn eq(&self, other: &Self) -> bool {
        self.identifier == other.identifier
    }
}
impl LocData {
    pub fn from_reader<R: io::Read>(reader: R) -> csv::Result<Self> {
        let mut entries = vec![];
        let mut reader = csv::ReaderBuilder::new()
            .trim(csv::Trim::All)
            .from_reader(reader);
        let mut record = csv::StringRecord::new();
        while reader.read_record(&mut record)? {
            let record = record.deserialize::<LocDataInput>(None)?;
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
                uber_identifier: UberIdentifier::new(record.uber_group, record.uber_id),
                value: record.uber_state_value,
                position,
                map_position,
            })
        }
        Ok(Self { entries })
    }
}

#[derive(Deserialize)]
#[serde(rename_all = "PascalCase")]
struct LocDataInput<'a> {
    node_identifier: String,
    #[serde(with = "LocDataZone")]
    zone: Zone,
    // TODO this might be cool to have
    _pickup_type: &'a str,
    _pickup_details: &'a str,
    _uber_group_name: &'a str,
    uber_group: i32,
    _uber_id_name: &'a str,
    uber_id: i32,
    uber_state_value: Option<u8>,
    x: Option<f32>,
    y: Option<f32>,
    map_x: Option<f32>,
    map_y: Option<f32>,
}
#[derive(Deserialize)]
#[serde(remote = "Zone")]
enum LocDataZone {
    #[serde(rename = "Inkwater Marsh")]
    Inkwater,
    #[serde(rename = "Kwoloks Hollow")]
    Hollow,
    #[serde(rename = "Wellspring Glades")]
    Glades,
    #[serde(rename = "The Wellspring")]
    Wellspring,
    #[serde(rename = "Silent Woods")]
    Woods,
    #[serde(rename = "Baurs Reach")]
    Reach,
    #[serde(rename = "Mouldwood Depths")]
    Depths,
    #[serde(rename = "Luma Pools")]
    Luma,
    #[serde(rename = "Windswept Wastes")]
    Wastes,
    #[serde(rename = "Windtorn Ruins")]
    Ruins,
    #[serde(rename = "Willows End")]
    Willow,
    #[serde(rename = "Midnight Burrows")]
    Burrows,
    #[serde(rename = "Shop")]
    Shop,
    #[serde(rename = "Void")]
    Void,
}
