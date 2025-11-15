use crate::{LocData, StateData};
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    hash::Hash,
};
use wotw_seedgen_data::UberIdentifier;

/// Information about all UberStates used by the game
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UberStateData {
    /// Two-level map to resolve UberStates by their internal name.
    /// Resolve by the group name first, then the member name.
    /// UberState names are often written as `<group>.<member>`
    ///
    /// Every UberState does have a name, but multiple UberStates can have
    /// the same name, which is why this resolves to [`Vec<UberStateAlias>`]
    ///
    /// This may also include randomizer aliases, which are generally more intuitive.
    ///
    /// If successful, this lookup will yield you the name's corresponding [`UberIdentifier`],
    /// which you can use to query `id_lookup` for additional information
    pub name_lookup: FxHashMap<String, FxHashMap<String, Vec<UberStateAlias>>>,
    /// Query a unique `UberIdentifier` for information about the UberState
    pub id_lookup: FxHashMap<UberIdentifier, UberStateDataEntry>,
}

/// Successful Resolution of an UberState name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UberStateAlias {
    /// The unique `UberIdentifier` corresponding to this name
    pub uber_identifier: UberIdentifier,
    /// `None` for all regular UberState names
    ///
    /// For custom identifiers from the randomizer, an additional value may be associated which
    /// represents the minimum value inside the UberState. For instance, all Hand to Hand steps
    /// have individual custom identifiers, even though Hand to Hand progress is stored in a single
    /// UberState. The value represents the current step of Hand to Hand.
    pub value: Option<i32>,
}

impl Display for UberStateAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uber_identifier)?;
        if let Some(value) = self.value {
            write!(f, " >= {}", value)?;
        }
        Ok(())
    }
}

/// Information about an UberState
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct UberStateDataEntry {
    /// Regular name of this UberState as defined by the base game
    ///
    /// These names are not always unique to the UberState
    pub name: String,
    /// If one exists, the randomizer's custom identifier for this UberState, which is often more intuitive than the regular name
    pub rando_name: Option<String>,
    /// Default `UberStateValue` of this UberState after starting a new save
    pub default_value: UberStateValue,
    /// If `true`, writing to this UberState manually will fail
    pub readonly: bool,
}

impl UberStateDataEntry {
    /// Returns `rando_name` if available, otherwise returns `name`
    pub fn preferred_name(&self) -> &String {
        self.rando_name.as_ref().unwrap_or(&self.name)
    }
}

/// Typed value stored inside an UberState
///
/// The types are simplified since a lot of the used types are similar in nature
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum UberStateValue {
    Boolean(bool),
    Integer(i32),
    Float(f32),
}

impl UberStateValue {
    pub fn as_boolean(self) -> bool {
        match self {
            UberStateValue::Boolean(value) => value,
            _ => {
                eprintln!("Attempted to access {self} UberState as Boolean");
                Default::default()
            }
        }
    }

    pub fn expect_boolean(self) -> bool {
        match self {
            UberStateValue::Boolean(value) => value,
            _ => panic!("Attempted to access {self} UberState as Boolean"),
        }
    }

    pub fn as_integer(self) -> i32 {
        match self {
            UberStateValue::Integer(value) => value,
            _ => {
                eprintln!("Attempted to access {self} UberState as Integer");
                Default::default()
            }
        }
    }

    pub fn expect_integer(self) -> i32 {
        match self {
            UberStateValue::Integer(value) => value,
            _ => panic!("Attempted to access {self} UberState as Integer"),
        }
    }

    pub fn as_float(self) -> f32 {
        match self {
            UberStateValue::Float(value) => value,
            _ => {
                eprintln!("Attempted to access {self} UberState as Float");
                Default::default()
            }
        }
    }

    pub fn expect_float(self) -> f32 {
        match self {
            UberStateValue::Float(value) => value,
            _ => panic!("Attempted to access {self} UberState as Float"),
        }
    }
}

impl PartialEq<bool> for UberStateValue {
    fn eq(&self, other: &bool) -> bool {
        self.as_boolean() == *other
    }
}

impl PartialOrd<bool> for UberStateValue {
    fn partial_cmp(&self, other: &bool) -> Option<Ordering> {
        self.as_boolean().partial_cmp(other)
    }
}

impl PartialEq<i32> for UberStateValue {
    fn eq(&self, other: &i32) -> bool {
        self.as_integer() == *other
    }
}

impl PartialOrd<i32> for UberStateValue {
    fn partial_cmp(&self, other: &i32) -> Option<Ordering> {
        self.as_integer().partial_cmp(other)
    }
}

impl PartialEq<f32> for UberStateValue {
    fn eq(&self, other: &f32) -> bool {
        self.as_float() == *other
    }
}

impl PartialOrd<f32> for UberStateValue {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.as_float().partial_cmp(other)
    }
}

impl Display for UberStateValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            UberStateValue::Boolean(value) => value.fmt(f),
            UberStateValue::Integer(value) => value.fmt(f),
            UberStateValue::Float(value) => value.fmt(f),
        }
    }
}

impl UberStateData {
    pub fn from_parts(dump: UberStateDump, loc_data: &LocData, state_data: &StateData) -> Self {
        let mut uber_state_data = Self::default();

        for (group, dump_group) in dump.groups {
            let group_map = uber_state_data
                .name_lookup
                .entry(dump_group.name.clone())
                .or_default();

            for (member, dump_member) in dump_group.states {
                let name = format!("{}.{}", dump_group.name, dump_member.name);

                group_map
                    .entry(dump_member.name)
                    .or_default()
                    .push(UberStateAlias {
                        uber_identifier: UberIdentifier::new(group, member),
                        value: None,
                    });

                let default_value = match dump_member.value_type {
                    UberStateDumpValueType::Boolean => {
                        UberStateValue::Boolean(dump_member.value != 0.)
                    }
                    UberStateDumpValueType::Byte | UberStateDumpValueType::Integer => {
                        UberStateValue::Integer(dump_member.value as i32)
                    }
                    UberStateDumpValueType::Float => UberStateValue::Float(dump_member.value),
                    UberStateDumpValueType::Unknown => continue,
                };

                uber_state_data.id_lookup.insert(
                    UberIdentifier::new(group, member),
                    UberStateDataEntry {
                        name,
                        rando_name: None,
                        default_value,
                        readonly: dump_member.readonly,
                    },
                );
            }
        }

        for record in &loc_data.entries {
            uber_state_data.add_rando_name(
                record.identifier.clone(),
                record.uber_identifier,
                record.value,
            );
        }

        for record in &state_data.entries {
            uber_state_data.add_rando_name(
                record.identifier.clone(),
                record.uber_identifier,
                record.value,
            );
        }

        uber_state_data
    }

    fn add_rando_name(
        &mut self,
        name: String,
        uber_identifier: UberIdentifier,
        value: Option<i32>,
    ) {
        let (group, member) = name.split_once('.').expect("Invalid UberState name");

        self.name_lookup
            .entry(group.to_string())
            .or_default()
            .entry(member.to_string())
            .or_default()
            .push(UberStateAlias {
                uber_identifier,
                value,
            });

        self.id_lookup.get_mut(&uber_identifier).unwrap().rando_name = Some(name);
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UberStateDump {
    #[serde(serialize_with = "serialize_sorted_map")]
    pub groups: FxHashMap<i32, UberStateDumpGroup>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UberStateDumpGroup {
    pub name: String,
    #[serde(serialize_with = "serialize_sorted_map")]
    pub states: FxHashMap<i32, UberStateDumpMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UberStateDumpMember {
    pub name: String,
    pub readonly: bool,
    #[serde(rename = "type")]
    pub uber_state_type: String,
    pub value: f32,
    pub value_type: UberStateDumpValueType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UberStateDumpValueType {
    Boolean,
    Byte,
    Integer,
    Float,
    Unknown,
}

fn serialize_sorted_map<K, V, S>(t: &FxHashMap<K, V>, ser: S) -> Result<S::Ok, S::Error>
where
    K: Hash + Eq + Ord + Serialize,
    V: Serialize,
    S: Serializer,
{
    let mut map = t.into_iter().collect::<IndexMap<_, _, FxBuildHasher>>();

    map.sort_unstable_keys();

    map.serialize(ser)
}
