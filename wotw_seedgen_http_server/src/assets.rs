use std::{ffi::OsStr, fs, path::Path};

use rustc_hash::FxHashMap;
use wotw_seedgen::{
    Generator, SeedUniverse,
    data::{
        MapIcon, UniverseSettings,
        assets::{
            AssetCache, AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultAssetCacheValues,
            DefaultFileAccess, LocData, PresetFileAccess, SEEDGEN_USER_DATA_DIR, SnippetFileAccess,
            StateData, UberStateData,
        },
        logic_language::{ast::Paths, output::Graph},
        parse::Source,
        seed_language::{metadata::Metadata, simulate::UberStates},
    },
    log_capture::{LogCapture, Record},
};

use crate::api::{
    logic::{MapIcons, RelevantUberStates, SpawnAnchors},
    snippets::{SnippetInfo, SnippetOrigin},
};

pub type Cache = AssetCache<DefaultFileAccess, CacheValues>;

pub struct CacheValues {
    pub base: DefaultAssetCacheValues,
    pub graph: Graph,
    pub uber_states: UberStates,
    pub map_icons: MapIcons,
    pub grom_shop_map_icon_index: usize,
    pub node_index_to_map_icon_index: FxHashMap<usize, usize>,
    pub relevant_uber_states: RelevantUberStates,
    pub spawn_anchors: SpawnAnchors,
    pub snippet_info: FxHashMap<String, SnippetInfo>,
}

impl CacheValues {
    pub fn generate(
        &self,
        settings: &UniverseSettings,
        log_level: log::LevelFilter,
    ) -> Result<(SeedUniverse, Vec<Record>), String> {
        let log_capture = LogCapture::new().with_max_level(log_level);

        let seed = Generator::new(
            &self.graph,
            &self.base.loc_data,
            &self.base.uber_state_data,
            &self.base,
            settings,
        )
        .with_log_capture(&log_capture)
        .generate()?;

        Ok((seed, log_capture.finish()))
    }
}

impl AssetCacheValues for CacheValues {
    // TODO custom error types on traits?
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let base = DefaultAssetCacheValues::new(file_access)?;

        let map_icons = MapIcons::new(&base.loc_data);
        let grom_shop_map_icon_index = grom_shop_map_icon_index(&map_icons);
        let uber_states = UberStates::new(&base.uber_state_data);
        let relevant_uber_states = RelevantUberStates::new(&base.loc_data, &base.state_data);

        let graph = graph(&base.paths, &base.loc_data, &base.state_data)?;
        let spawn_anchors = SpawnAnchors::new(&graph);

        let node_index_to_map_icon_index = node_index_to_map_icon_index(&graph, &map_icons);

        let snippet_info = snippet_info(&base.snippets);

        Ok(Self {
            base,
            graph,
            uber_states,
            map_icons,
            grom_shop_map_icon_index,
            node_index_to_map_icon_index,
            relevant_uber_states,
            spawn_anchors,
            snippet_info,
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

    fn paths(&self) -> &Source {
        &self.base.paths
    }

    fn snippet(&self, identifier: &str) -> Result<&Source, String> {
        self.base.snippet(identifier)
    }

    fn allow_read_file(&self) -> bool {
        false
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
            self.grom_shop_map_icon_index = grom_shop_map_icon_index(&self.map_icons);
        }

        if changed.uber_state_dump {
            self.uber_states = UberStates::new(&self.base.uber_state_data);
        }

        if changed.loc_data || changed.state_data {
            self.relevant_uber_states =
                RelevantUberStates::new(&self.base.loc_data, &self.base.state_data);
        }

        if changed.loc_data || changed.state_data || changed.paths {
            self.graph = graph(&self.base.paths, &self.base.loc_data, &self.base.state_data)?;
            self.spawn_anchors = SpawnAnchors::new(&self.graph);

            self.node_index_to_map_icon_index =
                node_index_to_map_icon_index(&self.graph, &self.map_icons);
        }

        // TODO patch maybe?
        if !changed.snippets.is_empty() {
            self.snippet_info = snippet_info(&self.base.snippets);
        }

        Ok(())
    }
}

fn grom_shop_map_icon_index(map_icons: &MapIcons) -> usize {
    map_icons
        .map_icons
        .iter()
        .position(|map_icon| map_icon.label == "GromShop")
        .unwrap()
}

fn graph(source: &Source, loc_data: &LocData, state_data: &StateData) -> Result<Graph, String> {
    let paths = Paths::parse(&source.content)
        .eprint_errors(source)
        .ok_or(String::new())?;

    Graph::compiler()
        .compile(paths, loc_data.clone(), state_data.clone())
        .eprint_errors(source)
        .ok_or(String::new())
}

fn node_index_to_map_icon_index(graph: &Graph, map_icons: &MapIcons) -> FxHashMap<usize, usize> {
    graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(node_index, node)| {
            let identifier = node.identifier();

            map_icons
                .map_icons
                .iter()
                .position(|map_icon| match map_icon.icon {
                    MapIcon::Opher | MapIcon::Twillen | MapIcon::Lupo => {
                        identifier.starts_with(&map_icon.label)
                    }
                    _ => map_icon.label == identifier,
                })
                .map(|map_icon_index| (node_index, map_icon_index))
        })
        .collect()
}

fn snippet_info(snippets: &FxHashMap<String, Source>) -> FxHashMap<String, SnippetInfo> {
    // TODO cache asts?
    let mut snippet_info = snippets
        .iter()
        .map(|(identifier, source)| {
            (
                identifier.clone(),
                SnippetInfo {
                    origin: SnippetOrigin::ExecutableDir,
                    metadata: Metadata::from_source(&source.content),
                },
            )
        })
        .collect::<FxHashMap<_, _>>();

    for data_dir_snippet in data_dir_snippets() {
        snippet_info.get_mut(&data_dir_snippet).unwrap().origin = SnippetOrigin::UserDataDir;
    }

    snippet_info
}

fn data_dir_snippets() -> impl Iterator<Item = String> {
    fs::read_dir(SEEDGEN_USER_DATA_DIR.join("snippets"))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.file_name())
        .filter(|file_name| Path::new(file_name).extension() == Some(OsStr::new("wotws")))
        .map(|file_name| {
            Path::new(&file_name)
                .file_stem()
                .unwrap()
                .to_string_lossy()
                .to_string()
        })
}
