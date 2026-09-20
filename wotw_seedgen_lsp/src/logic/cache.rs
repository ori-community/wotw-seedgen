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
    pub loc_data: Result<LocData, String>,
    pub state_data: Result<StateData, String>,
}

impl AssetCacheValues for CacheValues {
    fn new<F>(file_access: &F) -> Self
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let loc_data = file_access.loc_data();
        let state_data = file_access.state_data();

        Self {
            loc_data,
            state_data,
        }
    }

    fn loc_data(&self) -> Result<&LocData, String> {
        self.loc_data.as_ref().map_err(String::clone)
    }

    fn state_data(&self) -> Result<&StateData, String> {
        self.state_data.as_ref().map_err(String::clone)
    }

    fn uber_state_data(&self) -> Result<&UberStateData, String> {
        unimplemented!()
    }

    fn paths(&self) -> Result<&Source, String> {
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

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets)
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        if changed.loc_data {
            self.loc_data = file_access.loc_data();
        }
        if changed.state_data {
            self.state_data = file_access.state_data();
        }
    }
}
