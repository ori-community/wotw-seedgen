//! Utilities to deal with some of the simpler asset files used by the seed generator.

mod known_asset_access;
pub use known_asset_access::KnownAssetAccess;
mod pickups;
pub use pickups::{Pickups, PickupsEntry};
mod states;
pub use states::{States, StatesEntry};
mod uber_state_data;
pub use uber_state_data::{
    RandoUberStateGroup, UberStateAlias, UberStateData, UberStateDataEntry, UberStateDump,
    UberStateDumpGroup, UberStateDumpMember, UberStateDumpValueType, UberStateNameEntry,
    UberStateValue,
};
mod snippet_access;
pub use snippet_access::{ChainedSnippetAccess, InlineSnippets, SnippetAccess};
mod plando_access;
pub use plando_access::PlandoAccess;
mod presets;
pub use presets::{
    PresetAccess, PresetGroup, PresetInfo, Tricks, UniversePreset, UniversePresetSettings,
    WorldPreset, WorldPresetSettings, CURRENT_ASSETS_VERSION,
};
mod no_access;
pub use no_access::NoAccess;
mod file_access;
pub use file_access::{
    canonicalize, create_dir_all, file_create, file_err, file_open, metadata, read_dir,
    read_to_string, write, AssetCache, AssetCacheValues, ChangedAssets, DefaultAssetCacheValues,
    DefaultFileAccess, FullFileAccess, KnownAssetFileAccess, PlandoDirAccess, PlandoFileAccess,
    PresetFileAccess, ReadDir, SnippetFileAccess, Watcher, WatcherError, WatcherMessage,
    WatcherResult, CLIENT_USER_DATA_DIR, EXECUTABLE_DIR, LOG_DATA_DIR, RANDOMIZER_USER_DATA_DIR,
    SEEDGEN_USER_DATA_DIR,
};
#[cfg(any(test, feature = "test_helpers"))]
mod test_access;
#[cfg(any(test, feature = "test_helpers"))]
pub use test_access::{TestAccess, TEST_ASSETS};

use serde::{de::Error, Deserialize};

use crate::UberIdentifier;

struct LogicCondition {
    /// `UberIdentifier` where this world state is stored
    ///
    /// world states are either stored as booleans or as integers where being above a certain value means the world state is completed
    pub uber_identifier: UberIdentifier,
    /// `None` if `uber_identifier` holds a boolean value. Otherwise, has the minimum integer value at which this world state is completed
    pub value: Option<i32>,
}

impl<'de> Deserialize<'de> for LogicCondition {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let str = <&str>::deserialize(deserializer)?;

        str.split_once('|')
            .and_then(|(group, remaining)| {
                let group = group.parse().ok()?;

                match remaining.split_once(|c: char| !c.is_ascii_digit()) {
                    None => Some(Self {
                        uber_identifier: UberIdentifier::new(group, remaining.parse().ok()?),
                        value: None,
                    }),
                    Some((member, condition)) => {
                        let member = member.parse().ok()?;

                        let (operator, value) =
                            condition.rsplit_once(|c: char| !c.is_ascii_digit())?;

                        if operator.trim() != ">=" {
                            return None;
                        }

                        Some(Self {
                            uber_identifier: UberIdentifier::new(group, member),
                            value: Some(value.parse().ok()?),
                        })
                    }
                }
            })
            .ok_or_else(|| D::Error::custom("expected group|member or group|member >= value"))
    }
}
