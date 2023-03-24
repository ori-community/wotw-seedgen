use crate::uber_state::{
    UberIdentifier, UberStateComparator, UberStateCondition, UberStateTrigger,
};
use crate::util::{self, Position, Zone};
use serde::Deserialize;

/// Information about a pickup location
#[derive(Debug, Clone, PartialEq)]
pub struct Location {
    pub name: String,
    pub zone: Zone,
    pub trigger: UberStateTrigger,
    pub position: Position,
    pub map_position: Position,
}

#[derive(Deserialize)]
struct LocationEntry<'a> {
    name: String,
    zone: LocationZone,
    _kind: &'a str,
    _variant: &'a str,
    _uber_group_name: &'a str,
    uber_group: u16,
    _uber_id_name: &'a str,
    uber_id: u16,
    uber_state_value: Option<u32>,
    x: f32,
    y: f32,
    map_x: f32,
    map_y: f32,
}

#[derive(Deserialize)]
enum LocationZone {
    #[serde(rename(deserialize = "Inkwater Marsh"))]
    Marsh = 0,
    #[serde(rename(deserialize = "Kwoloks Hollow"))]
    Hollow = 1,
    #[serde(rename(deserialize = "Wellspring Glades"))]
    Glades = 2,
    #[serde(rename(deserialize = "The Wellspring"))]
    Wellspring = 3,
    #[serde(rename(deserialize = "Silent Woods"))]
    Woods = 7,
    #[serde(rename(deserialize = "Baurs Reach"))]
    Reach = 6,
    #[serde(rename(deserialize = "Mouldwood Depths"))]
    Depths = 8,
    #[serde(rename(deserialize = "Luma Pools"))]
    Pools = 4,
    #[serde(rename(deserialize = "Windswept Wastes"))]
    Wastes = 9,
    #[serde(rename(deserialize = "Windtorn Ruins"))]
    Ruins = 10,
    #[serde(rename(deserialize = "Willows End"))]
    Willow = 11,
    #[serde(rename(deserialize = "Midnight Burrows"))]
    Burrows = 5,
    #[serde(rename(deserialize = "Shop"))]
    Shop = 12,
    #[serde(rename(deserialize = "Void"))]
    Void = 13,
}
impl From<LocationZone> for Zone {
    fn from(zone: LocationZone) -> Self {
        Zone::from(zone as u8)
    }
}

/// Parses pickup location data from a csv format
///
/// # Examples
///
/// ```
/// # use wotw_seedgen::logic::{parse_locations, Location};
/// use wotw_seedgen::uber_state::UberStateTrigger;
/// use wotw_seedgen::util::Position;
/// use wotw_seedgen::util::Zone;
///
/// let input = "
/// PickupIdentifier, Zone, PickupType, PickupDetails, UberGroupName, UberGroup, UberIdName, UberId, UberStateValue, X, Y, MapX, MapY
/// MarshSpawn.RockHC, Inkwater Marsh, Resource, Life, swampStateGroup, 21786, healthContainerA, 60210, 1, -958, -4313, -958, -4313
/// GladesTown.MotayHutEX, Wellspring Glades, SpiritLight, 100, hubUberStateGroup, 42178, hutCExpOrb, 57455, 1, -172, -4584, -394, -4136";
/// let locations = parse_locations(input).unwrap();
///
/// assert_eq!(locations, vec![
///     Location {
///         name: "MarshSpawn.RockHC".to_string(),
///         zone: Zone::Marsh,
///         trigger: "21786|60210>=1".parse().unwrap(),
///         position: Position::new(-958., -4313.),
///         map_position: Position::new(-958., -4313.),
///     },
///     Location {
///         name: "GladesTown.MotayHutEX".to_string(),
///         zone: Zone::Glades,
///         trigger: "42178|57455>=1".parse().unwrap(),
///         position: Position::new(-172., -4584.),
///         map_position: Position::new(-394., -4136.),
///     }
/// ]);
/// ```
pub fn parse_locations(input: &str) -> Result<Vec<Location>, String> {
    let mut reader = csv::ReaderBuilder::new()
        .trim(csv::Trim::All)
        .from_reader(input.as_bytes());

    let mut locations = Vec::with_capacity(389);

    let mut record = csv::StringRecord::new();
    while reader
        .read_record(&mut record)
        .map_err(|err| err.to_string())?
    {
        let record = record.deserialize(None).map_err(|err| err.to_string())?;
        let LocationEntry {
            name,
            zone,
            uber_group,
            uber_id,
            uber_state_value,
            x,
            y,
            map_x,
            map_y,
            ..
        } = record;

        let zone = zone.into();
        let identifier = UberIdentifier::new(uber_group, uber_id);
        let condition = uber_state_value.map(|value| UberStateCondition {
            comparator: UberStateComparator::GreaterOrEquals,
            value,
        });
        let trigger = UberStateTrigger {
            identifier,
            condition,
        };
        let x = util::float_to_real(x)?;
        let y = util::float_to_real(y)?;
        let position = Position { x, y };
        let x = util::float_to_real(map_x)?;
        let y = util::float_to_real(map_y)?;
        let map_position = Position { x, y };
        let location = Location {
            name,
            zone,
            trigger,
            position,
            map_position,
        };

        locations.push(location);
    }

    Ok(locations)
}
