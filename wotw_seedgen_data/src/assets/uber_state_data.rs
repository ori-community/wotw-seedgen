use crate::{
    assets::{LocData, StateData},
    UberIdentifier,
};
use indexmap::IndexMap;
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::{Deserialize, Serialize, Serializer};
use std::{
    cmp::Ordering,
    fmt::{self, Display},
    hash::Hash,
};

/// Information about all UberStates used by the game
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct UberStateData {
    /// Nested lookup to resolve UberStates by name.
    ///
    /// Vanilla UberState names are usually written as camelCase `<group>.<member>`.
    /// Resolve by `<group>` first, then `<member>`. This should yield a [`Vec<UberIdentifier>`].
    ///
    /// Rando names are usually written as PascalCase `<zone>[.<region>].<item>`.
    /// Resolve by the first two parts, then continue as desired in the resulting [`RandoUberStateGroup`].
    ///
    /// In both cases once you have resolved to an [`UberIdentifier`],
    /// you can query `id_lookup` for additional information.
    pub name_lookup: FxHashMap<String, FxHashMap<String, UberStateNameEntry>>,
    /// Query a unique `UberIdentifier` for information about the UberState
    pub id_lookup: FxHashMap<UberIdentifier, UberStateDataEntry>,
}

/// Successful resolution of two UberState name parts
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UberStateNameEntry {
    /// Vanilla names are always two-parted and resolve directly to [`UberIdentifier`]s, but multiple may have the same name.
    Vanilla(Vec<UberIdentifier>),
    /// Rando names can resolve to different expressions, see [`RandoUberStateGroup`]
    Rando(RandoUberStateGroup),
}

/// Successful resolution of two name parts of a rando UberState
#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
pub struct RandoUberStateGroup {
    /// Two-parted rando names will be found here.
    pub root_member: Option<UberStateAlias>,
    /// Three-parted rando names can be looked up further in this map.
    pub members: FxHashMap<String, UberStateAlias>,
}

/// Successful Resolution of a rando UberState name
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct UberStateAlias {
    /// The unique `UberIdentifier` corresponding to this name
    pub uber_identifier: UberIdentifier,
    /// `None` if the rando name resolves directly to `uber_identifier`.
    ///
    /// If `Some`, this represents the expression `uber_identifier` >= `value`, which is commonly used in logic.
    /// For instance, all Hand to Hand steps have individual rando names, even though Hand to Hand progress
    /// is stored in a single UberState. The value then represents the current step of Hand to Hand.
    pub value: Option<i32>,
}

impl UberStateAlias {
    /// An alias directly referring to an [`UberIdentifier`]
    pub fn identifier(uber_identifier: UberIdentifier) -> Self {
        Self {
            uber_identifier,
            value: None,
        }
    }
}

impl Display for UberStateAlias {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.uber_identifier)?;
        if let Some(value) = self.value {
            write!(f, " >= {value}")?;
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
        if let UberStateValue::Boolean(value) = self {
            value
        } else {
            eprintln!("Attempted to access {self:?} UberState as Boolean");
            false
        }
    }

    pub fn expect_boolean(self) -> bool {
        match self {
            UberStateValue::Boolean(value) => value,
            _ => panic!("Attempted to access {self:?} UberState as Boolean"),
        }
    }

    pub fn as_integer(self) -> i32 {
        if let UberStateValue::Integer(value) = self {
            value
        } else {
            eprintln!("Attempted to access {self:?} UberState as Integer");
            0
        }
    }

    pub fn expect_integer(self) -> i32 {
        match self {
            UberStateValue::Integer(value) => value,
            _ => panic!("Attempted to access {self:?} UberState as Integer"),
        }
    }

    pub fn as_float(self) -> f32 {
        if let UberStateValue::Float(value) = self {
            value
        } else {
            eprintln!("Attempted to access {self:?} UberState as Float");
            0.
        }
    }

    pub fn expect_float(self) -> f32 {
        match self {
            UberStateValue::Float(value) => value,
            _ => panic!("Attempted to access {self:?} UberState as Float"),
        }
    }
}

impl From<bool> for UberStateValue {
    fn from(value: bool) -> Self {
        Self::Boolean(value)
    }
}

impl From<i32> for UberStateValue {
    fn from(value: i32) -> Self {
        Self::Integer(value)
    }
}

impl From<f32> for UberStateValue {
    fn from(value: f32) -> Self {
        Self::Float(value)
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

                match group_map
                    .entry(dump_member.name)
                    .or_insert(UberStateNameEntry::Vanilla(Vec::new()))
                {
                    UberStateNameEntry::Vanilla(uber_identifiers) => {
                        uber_identifiers.push(UberIdentifier::new(group, member))
                    }
                    UberStateNameEntry::Rando(_) => rando_name_in_vanilla_group(&dump_group.name),
                };

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
                &record.identifier,
                record.uber_identifier,
                record.value,
            );
        }

        for record in &state_data.entries {
            uber_state_data.add_rando_name(
                &record.identifier,
                record.uber_identifier,
                record.value,
            );
        }

        uber_state_data
    }

    fn add_rando_name(&mut self, name: &str, uber_identifier: UberIdentifier, value: Option<i32>) {
        let mut parts = name.split('.');
        let zone = parts.next().unwrap();
        let region_or_pickup = parts.next().expect("Invalid UberState name");

        let rando_group = match self
            .name_lookup
            .entry(zone.to_string())
            .or_default()
            .entry(region_or_pickup.to_string())
            .or_insert(UberStateNameEntry::Rando(RandoUberStateGroup::default()))
        {
            UberStateNameEntry::Vanilla(_) => rando_name_in_vanilla_group(zone),
            UberStateNameEntry::Rando(rando_group) => rando_group,
        };

        let alias = UberStateAlias {
            uber_identifier,
            value,
        };

        let previous = match parts.next() {
            None => rando_group.root_member.replace(alias),
            Some(pickup) => rando_group.members.insert(pickup.to_string(), alias),
        };

        if previous.is_some() {
            panic!("duplicate rando name \"{name}\"");
        }

        if parts.next().is_some() {
            panic!("rando name \"{name}\" exceeds three parts");
        }

        if value.is_none() {
            self.id_lookup.get_mut(&uber_identifier).unwrap().rando_name = Some(name.to_string());
        }
    }
}

fn rando_name_in_vanilla_group(group: &str) -> ! {
    panic!("Vanilla UberState group \"{group}\" cannot contain rando names")
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
    let mut map = t.iter().collect::<IndexMap<_, _, FxBuildHasher>>();

    map.sort_unstable_keys();

    map.serialize(ser)
}
