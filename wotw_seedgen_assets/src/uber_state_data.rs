use crate::{LocData, StateData};
use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use std::{
    fmt::{self, Display},
    io::Read,
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
    /// With the `loc_data` and/or `state_data` features enabled, this may also include
    /// the randomizer's custom identifiers, which are generally more intuitive
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
    /// have individual custom identifier, even though Hand to Hand progress is stored in a single
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
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum UberStateValue {
    Boolean(bool),
    Integer(i32),
    Float(f32),
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
    /// Parse from a [`Read`] implementation, such as a file or byte slice
    ///
    /// Information from `loc_data` and `state_data` will be included
    pub fn from_reader<R: Read>(
        reader: R,
        loc_data: &LocData,
        state_data: &StateData,
    ) -> serde_json::Result<Self> {
        let mut uber_state_data = Self::default();

        let dump: Dump = serde_json::from_reader(reader)?;
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
                    ValueType::Boolean => UberStateValue::Boolean(dump_member.value != 0.),
                    ValueType::Byte | ValueType::Integer => {
                        UberStateValue::Integer(dump_member.value as i32)
                    }
                    ValueType::Float => UberStateValue::Float(dump_member.value),
                    ValueType::Unknown => continue,
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

        Ok(uber_state_data)
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

#[derive(Deserialize)]
struct Dump {
    groups: FxHashMap<i32, DumpGroup>,
}

#[derive(Deserialize)]
struct DumpGroup {
    name: String,
    states: FxHashMap<i32, DumpMember>,
}

#[derive(Deserialize)]
struct DumpMember {
    name: String,
    readonly: bool,
    value: f32,
    value_type: ValueType,
}

#[derive(Deserialize)]
enum ValueType {
    Boolean,
    Byte,
    Integer,
    Float,
    Unknown,
}
