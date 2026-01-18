use std::iter;

use wotw_seedgen_data::{
    assets::{
        AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultFileAccess, LocData,
        PresetFileAccess, SnippetFileAccess, StateData, UberStateData,
    },
    parse::Source,
};
use wotw_seedgen_server_shared::ServerState;

pub type Cache = ServerState<DefaultFileAccess, CacheValues>;

pub struct CacheValues {
    pub loc_data: LocData,
    pub state_data: StateData,
}

impl AssetCacheValues for CacheValues {
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let loc_data = file_access.loc_data()?;
        let state_data = file_access.state_data()?;

        Ok(Self {
            loc_data,
            state_data,
        })
    }

    fn loc_data(&self) -> &LocData {
        &self.loc_data
    }

    fn state_data(&self) -> &StateData {
        &self.state_data
    }

    fn uber_state_data(&self) -> &UberStateData {
        unimplemented!()
    }

    fn areas(&self) -> &Source {
        unimplemented!()
    }

    fn snippet(&self, _identifier: &str) -> Result<&Source, String> {
        unimplemented!()
    }

    fn allow_read_file(&self) -> bool {
        unimplemented!()
    }

    fn available_snippets(&self) -> impl Iterator<Item = &String> {
        #[allow(unreachable_code)]
        iter::once(unimplemented!())
    }

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets) -> Result<(), String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        if changed.loc_data {
            self.loc_data = file_access.loc_data()?;
        }
        if changed.state_data {
            self.state_data = file_access.state_data()?;
        }

        Ok(())
    }
}
