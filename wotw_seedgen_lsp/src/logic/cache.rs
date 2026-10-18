use std::iter;

use wotw_seedgen_data::{
    assets::{
        AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultFileAccess, Pickups,
        PresetFileAccess, SnippetFileAccess, States, UberStateData,
    },
    parse::Source,
};
use wotw_seedgen_server_shared::ServerState;

pub type Cache = ServerState<DefaultFileAccess, CacheValues>;

pub struct CacheValues {
    pub pickups: Result<Pickups, String>,
    pub states: Result<States, String>,
}

impl AssetCacheValues for CacheValues {
    fn new<F>(file_access: &F) -> Self
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let pickups = file_access.pickups();
        let states = file_access.states();

        Self {
            pickups,
            states,
        }
    }

    fn pickups(&self) -> Result<&Pickups, String> {
        self.pickups.as_ref().map_err(String::clone)
    }

    fn states(&self) -> Result<&States, String> {
        self.states.as_ref().map_err(String::clone)
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
        if changed.pickups {
            self.pickups = file_access.pickups();
        }
        if changed.states {
            self.states = file_access.states();
        }
    }
}
