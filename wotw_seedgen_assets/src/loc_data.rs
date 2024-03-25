//! Parses pickup location data from a csv format
//!
//! # Examples
//!
//! ```
//! # use wotw_seedgen::logic::{parse_locations, Location};
//! use wotw_seedgen::uber_state::UberStateTrigger;
//! use wotw_seedgen::util::Position;
//! use wotw_seedgen::util::Zone;
//!
//! let input = "
//! PickupIdentifier, Zone, PickupType, PickupDetails, UberGroupName, UberGroup, UberIdName, UberId, UberStateValue, X, Y, MapX, MapY
//! MarshSpawn.RockHC, Inkwater Marsh, Resource, Life, swampStateGroup, 21786, healthContainerA, 60210, 1, -958, -4313, -958, -4313
//! GladesTown.MotayHutEX, Wellspring Glades, SpiritLight, 100, hubUberStateGroup, 42178, hutCExpOrb, 57455, 1, -172, -4584, -394, -4136";
//! let locations = parse_locations(input).unwrap();
//!
//! assert_eq!(locations, vec![
//!     Location {
//!         name: "MarshSpawn.RockHC".to_string(),
//!         zone: Zone::Marsh,
//!         trigger: "21786|60210>=1".parse().unwrap(),
//!         position: Position::new(-958., -4313.),
//!         map_position: Position::new(-958., -4313.),
//!     },
//!     Location {
//!         name: "GladesTown.MotayHutEX".to_string(),
//!         zone: Zone::Glades,
//!         trigger: "42178|57455>=1".parse().unwrap(),
//!         position: Position::new(-172., -4584.),
//!         map_position: Position::new(-394., -4136.),
//!     }
//! ]);
//! ```

use serde::{Deserialize, Serialize};
use std::io;
use wotw_seedgen_data::Position;

pub use wotw_seedgen_data::{UberIdentifier, Zone};

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
