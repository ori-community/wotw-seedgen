use wotw_seedgen_parse::Source;

use crate::assets::{Pickups, States, UberStateData, UberStateDump};

/// Access to known assets
pub trait KnownAssetAccess {
    /// Return the [`Pickups`]
    fn pickups(&self) -> Result<Pickups, String>;

    /// Return the [`States`]
    fn states(&self) -> Result<States, String>;

    /// Return the paths [`Source`]
    fn paths(&self) -> Result<Source, String>;

    /// Return the [`UberStateDump`]
    fn uber_state_dump(&self) -> Result<UberStateDump, String>;

    fn uber_state_data(&self, pickups: &Pickups, states: &States) -> Result<UberStateData, String> {
        let dump = self.uber_state_dump()?;

        Ok(UberStateData::from_parts(dump, pickups, states))
    }
}
