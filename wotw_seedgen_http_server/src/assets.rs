use rustc_hash::FxHashMap;
use wotw_seedgen::{
    assets::{
        AssetCache, AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultAssetCacheValues,
        DefaultFileAccess, LocData, PresetFileAccess, SnippetFileAccess, Source, StateData,
        UberStateData, Watcher,
    },
    logic_language::{ast::Areas, output::Graph},
    seed_language::simulate::UberStates,
};

use crate::{
    RouterState,
    api::reach_check::{MapIcons, RelevantUberStates},
    error::Error,
};

pub type Cache = AssetCache<DefaultFileAccess, CacheValues>;

pub struct CacheValues {
    pub base: DefaultAssetCacheValues,
    pub graph: Graph,
    pub uber_states: UberStates,
    pub map_icons: MapIcons,
    pub node_index_to_map_icon_index: FxHashMap<usize, usize>,
    pub relevant_uber_states: RelevantUberStates,
}

impl AssetCacheValues for CacheValues {
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let base = DefaultAssetCacheValues::new(file_access)?;

        let map_icons = MapIcons::new(&base.loc_data);
        let uber_states = UberStates::new(&base.uber_state_data);
        let relevant_uber_states = RelevantUberStates::new(&base.loc_data, &base.state_data);

        let graph = graph(&base.areas, &base.loc_data, &base.state_data)?;

        let node_index_to_map_icon_index = node_index_to_map_icon_index(&graph, &map_icons);

        Ok(Self {
            base,
            graph,
            uber_states,
            map_icons,
            node_index_to_map_icon_index,
            relevant_uber_states,
        })
    }

    fn loc_data(&self) -> &LocData {
        &self.base.loc_data
    }

    fn state_data(&self) -> &StateData {
        &self.base.state_data
    }

    fn uber_state_data(&self) -> &UberStateData {
        &self.base.uber_state_data
    }

    fn areas(&self) -> &Source {
        &self.base.areas
    }

    fn snippet(&self, identifier: &str) -> Result<&Source, String> {
        self.base.snippet(identifier)
    }

    fn available_snippets(&self) -> impl Iterator<Item = &String> {
        self.base.available_snippets()
    }

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets) -> Result<(), String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        self.base.update(file_access, changed.clone())?;

        if changed.loc_data {
            self.map_icons = MapIcons::new(&self.base.loc_data);
        }

        if changed.uber_state_dump {
            self.uber_states = UberStates::new(&self.base.uber_state_data);
        }

        if changed.loc_data || changed.state_data {
            self.relevant_uber_states =
                RelevantUberStates::new(&self.base.loc_data, &self.base.state_data);
        }

        if changed.loc_data || changed.state_data || changed.areas {
            self.graph = graph(&self.base.areas, &self.base.loc_data, &self.base.state_data)?;

            self.node_index_to_map_icon_index =
                node_index_to_map_icon_index(&self.graph, &self.map_icons);
        }

        Ok(())
    }
}

fn graph(source: &Source, loc_data: &LocData, state_data: &StateData) -> Result<Graph, String> {
    let areas = Areas::parse(&source.content)
        .eprint_errors(source)
        .ok_or(String::new())?;

    Graph::compile(areas, loc_data.clone(), state_data.clone(), &[])
        .eprint_errors(source)
        .ok_or(String::new())
}

fn node_index_to_map_icon_index(graph: &Graph, map_icons: &MapIcons) -> FxHashMap<usize, usize> {
    graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(node_index, node)| {
            map_icons
                .map_icons
                .iter()
                .position(|map_icon| map_icon.identifier == node.identifier())
                .map(|map_icon_index| (node_index, map_icon_index))
        })
        .collect()
}

pub async fn watch_assets(state: RouterState, watcher: Watcher) {
    for res in watcher {
        or_print(
            (async || {
                let events = res?;

                let mut cache = state.write().await;
                cache
                    .update_from_watcher_event(&events)
                    .map_err(Error::ReloadAssets)?;

                eprintln!("Reloaded assets");

                Ok(())
            })()
            .await,
        );
    }
}

fn or_print(res: Result<(), Error>) {
    if let Err(err) = res {
        eprintln!("error in file watcher: {err}");
    }
}
