use std::{ffi::OsStr, fs, path::PathBuf};

use axum::Json;
use rustc_hash::FxHashMap;
use wotw_seedgen::{
    Generator, SeedUniverse,
    data::{
        MapIcon, UniverseSettings,
        assets::{
            AssetCache, AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultAssetCacheValues,
            DefaultFileAccess, LocData, PresetFileAccess, SEEDGEN_USER_DATA_DIR, SnippetFileAccess,
            StateData, UberStateData, UniversePreset, WorldPreset,
        },
        logic_language::{ast::Paths, output::Graph},
        parse::Source,
        seed_language::{metadata::Metadata, simulate::UberStates},
    },
    log_capture::{LogCapture, Record},
};

use crate::{
    api::{
        SchemaResult,
        assets::AssetOrigin,
        logic::{MapIcons, RelevantUberStates, SpawnAnchors},
        presets::{universe::UniversePresetInfo, world::WorldPresetInfo},
        snippets::SnippetInfo,
    },
    error::Error,
    generate::{GenerateError, GenerateResult},
};

pub type Cache = AssetCache<DefaultFileAccess, CacheValues>;

pub struct CacheValues {
    pub base: DefaultAssetCacheValues,
    pub graph: CacheResult<Graph>,
    pub uber_states: CacheResult<UberStates>,
    pub map_icons: CacheResult<MapIcons>,
    pub grom_shop_map_icon_index: Option<usize>,
    pub node_index_to_map_icon_index: CacheResult<FxHashMap<usize, usize>>,
    pub relevant_uber_states: CacheResult<RelevantUberStates>,
    pub spawn_anchors: CacheResult<SpawnAnchors>,
    pub snippet_info: FxHashMap<String, SchemaResult<SnippetInfo, String>>,
    pub universe_preset_info: FxHashMap<String, SchemaResult<UniversePresetInfo, String>>,
    pub world_preset_info: FxHashMap<String, SchemaResult<WorldPresetInfo, String>>,
}

impl CacheValues {
    pub fn generate(
        &self,
        settings: &UniverseSettings,
        log_level: log::LevelFilter,
    ) -> GenerateResult<(SeedUniverse, Vec<Record>)> {
        let generator = self
            .generator(settings)
            .map_err(|message| GenerateError::new(message, Vec::new()))?;

        let log_capture = LogCapture::new().with_max_level(log_level);

        let result = generator.with_log_capture(&log_capture).generate();
        let logs = log_capture.finish();

        match result {
            Ok(seed) => Ok((seed, logs)),
            Err(message) => Err(GenerateError::new(message, logs)),
        }
    }

    fn generator<'cache, 'settings, 'perf, 'log>(
        &'cache self,
        settings: &'settings UniverseSettings,
    ) -> Result<
        Generator<'cache, 'cache, 'cache, 'cache, 'settings, 'perf, 'log, DefaultAssetCacheValues>,
        String,
    > {
        Ok(Generator::new(
            ok_ref(&self.graph.0)?,
            ok_ref(&self.base.loc_data)?,
            ok_ref(&self.base.uber_state_data)?,
            &self.base,
            settings,
        ))
    }
}

impl AssetCacheValues for CacheValues {
    fn new<F>(file_access: &F) -> Self
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let base = DefaultAssetCacheValues::new(file_access);

        let loc_data = base.loc_data();
        let state_data = base.state_data();

        let uber_states = CacheResult(base.uber_state_data().map(UberStates::new));
        let map_icons = CacheResult(loc_data.clone().map(MapIcons::new));
        let grom_shop_map_icon_index = grom_shop_map_icon_index(&map_icons);

        let relevant_uber_states =
            CacheResult(relevant_uber_states(loc_data.clone(), state_data.clone()));

        let graph = CacheResult(graph(base.paths(), loc_data, state_data));
        let spawn_anchors = CacheResult(ok_ref(&graph.0).map(SpawnAnchors::new));

        let node_index_to_map_icon_index =
            CacheResult(node_index_to_map_icon_index(&graph, &map_icons));

        let snippet_info = snippet_info(&base.snippets);
        let universe_preset_info = universe_preset_info(&base.universe_presets);
        let world_preset_info = world_preset_info(&base.world_presets);

        Self {
            base,
            graph,
            uber_states,
            map_icons,
            grom_shop_map_icon_index,
            node_index_to_map_icon_index,
            relevant_uber_states,
            spawn_anchors,
            snippet_info,
            universe_preset_info,
            world_preset_info,
        }
    }

    // TODO custom error types on traits?
    fn loc_data(&self) -> Result<&LocData, String> {
        self.base.loc_data()
    }

    fn state_data(&self) -> Result<&StateData, String> {
        self.base.state_data()
    }

    fn uber_state_data(&self) -> Result<&UberStateData, String> {
        self.base.uber_state_data()
    }

    fn paths(&self) -> Result<&Source, String> {
        self.base.paths()
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

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets)
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        self.base.update(file_access, changed.clone());

        if changed.uber_state_dump {
            self.uber_states = CacheResult(self.base.uber_state_data().map(UberStates::new));
        }

        if changed.loc_data {
            self.map_icons = CacheResult(self.base.loc_data().map(MapIcons::new));
            self.grom_shop_map_icon_index = grom_shop_map_icon_index(&self.map_icons);
        }

        if changed.loc_data || changed.state_data {
            self.relevant_uber_states = CacheResult(relevant_uber_states(
                self.base.loc_data(),
                self.base.state_data(),
            ));
        }

        if changed.loc_data || changed.state_data || changed.paths {
            self.graph = CacheResult(graph(
                self.base.paths(),
                self.base.loc_data(),
                self.base.state_data(),
            ));
            self.spawn_anchors = CacheResult(ok_ref(&self.graph.0).map(SpawnAnchors::new));

            self.node_index_to_map_icon_index =
                CacheResult(node_index_to_map_icon_index(&self.graph, &self.map_icons));
        }

        // TODO patch maybe?
        if !changed.snippets.is_empty() {
            self.snippet_info = snippet_info(&self.base.snippets);
        }
    }
}

fn grom_shop_map_icon_index(map_icons: &CacheResult<MapIcons>) -> Option<usize> {
    map_icons.0.as_ref().ok().map(|map_icons| {
        map_icons
            .map_icons
            .iter()
            .position(|map_icon| map_icon.label == "GromShop")
            .unwrap()
    })
}

fn relevant_uber_states(
    loc_data: Result<&LocData, String>,
    state_data: Result<&StateData, String>,
) -> Result<RelevantUberStates, String> {
    Ok(RelevantUberStates::new(loc_data?, state_data?))
}

fn graph(
    source: Result<&Source, String>,
    loc_data: Result<&LocData, String>,
    state_data: Result<&StateData, String>,
) -> Result<Graph, String> {
    let source = source?;

    let paths = Paths::parse(&source.content)
        .eprint_errors(source)
        .ok_or("failed to parse paths".to_string())?;

    Graph::compiler()
        .compile(paths, loc_data?.clone(), state_data?.clone())
        .eprint_errors(source)
        .ok_or("failed to compile graph".to_string())
}

fn node_index_to_map_icon_index(
    graph: &CacheResult<Graph>,
    map_icons: &CacheResult<MapIcons>,
) -> Result<FxHashMap<usize, usize>, String> {
    let (graph, map_icons) = (ok_ref(&graph.0)?, ok_ref(&map_icons.0)?);

    Ok(graph
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
        .collect())
}

fn ok_ref<T, E: Clone>(res: &Result<T, E>) -> Result<&T, E> {
    match res {
        Ok(t) => Ok(t),
        Err(err) => Err(err.clone()),
    }
}

pub struct CacheResult<T>(Result<T, String>);

impl<T> CacheResult<T> {
    pub fn get(&self) -> Result<&T, Error> {
        match &self.0 {
            Ok(t) => Ok(t),
            Err(err) => Err(Error::Custom(err.clone())),
        }
    }
}

impl<T: Clone> CacheResult<T> {
    pub fn get_response(&self) -> Result<Json<T>, Error> {
        self.get().cloned().map(Json)
    }
}

trait AssetInfo {
    type Asset;

    fn new(asset: &Self::Asset) -> Self;

    fn origin(&mut self) -> &mut AssetOrigin;
}

impl AssetInfo for SnippetInfo {
    type Asset = Source;

    fn new(asset: &Self::Asset) -> Self {
        // TODO cache asts?
        let metadata = Metadata::from_source(&asset.content);
        // corrected later when the full tree is available
        let tree_requires_local_files = metadata.requires_local_files;

        Self {
            origin: AssetOrigin::ExecutableDir,
            metadata,
            tree_requires_local_files,
        }
    }

    fn origin(&mut self) -> &mut AssetOrigin {
        &mut self.origin
    }
}

impl AssetInfo for UniversePresetInfo {
    type Asset = UniversePreset;

    fn new(asset: &Self::Asset) -> Self {
        Self {
            origin: AssetOrigin::ExecutableDir,
            content: asset.clone(),
        }
    }

    fn origin(&mut self) -> &mut AssetOrigin {
        &mut self.origin
    }
}

impl AssetInfo for WorldPresetInfo {
    type Asset = WorldPreset;

    fn new(asset: &Self::Asset) -> Self {
        Self {
            origin: AssetOrigin::ExecutableDir,
            content: asset.clone(),
        }
    }

    fn origin(&mut self) -> &mut AssetOrigin {
        &mut self.origin
    }
}

fn asset_info<T, I>(
    assets: &FxHashMap<String, Result<T, String>>,
    folder: &str,
    extension: &str,
) -> FxHashMap<String, SchemaResult<I, String>>
where
    I: AssetInfo<Asset = T>,
{
    let mut asset_info = assets
        .iter()
        .map(|(identifier, asset)| (identifier.clone(), ok_ref(asset).map(I::new).into()))
        .collect::<FxHashMap<_, _>>();

    for path in data_dir_assets(folder, extension) {
        let identifier = path.file_stem().unwrap().to_str().unwrap();
        let path = path.to_str().unwrap().to_string();
        if let SchemaResult::Ok(info) = asset_info.get_mut(identifier).unwrap() {
            *info.origin() = AssetOrigin::UserDataDir { path };
        }
    }

    asset_info
}

fn data_dir_assets(folder: &str, extension: &str) -> impl Iterator<Item = PathBuf> {
    fs::read_dir(SEEDGEN_USER_DATA_DIR.join(folder))
        .into_iter()
        .flatten()
        .flatten()
        .map(|entry| entry.path())
        .filter(|path| path.extension() == Some(OsStr::new(extension)))
}

fn snippet_info(
    snippets: &FxHashMap<String, Result<Source, String>>,
) -> FxHashMap<String, SchemaResult<SnippetInfo, String>> {
    let mut snippet_info_map = asset_info::<_, SnippetInfo>(snippets, "snippets", "wotws");

    for identifier in snippets.keys() {
        let SchemaResult::Ok(snippet_info) = &snippet_info_map[identifier] else {
            continue;
        };

        let mut includes = snippet_info.metadata.includes.iter().collect::<Vec<_>>();

        while let Some(included_identifier) = includes.pop() {
            let SchemaResult::Ok(included_info) = &snippet_info_map[included_identifier] else {
                continue;
            };

            if included_info.tree_requires_local_files {
                let SchemaResult::Ok(snippet_info) = snippet_info_map.get_mut(identifier).unwrap()
                else {
                    unreachable!()
                };

                snippet_info.tree_requires_local_files = true;

                break;
            }

            includes.extend(included_info.metadata.includes.iter());
        }
    }

    snippet_info_map
}

fn universe_preset_info(
    universe_presets: &FxHashMap<String, Result<UniversePreset, String>>,
) -> FxHashMap<String, SchemaResult<UniversePresetInfo, String>> {
    asset_info(universe_presets, "universe_presets", "json")
}

fn world_preset_info(
    world_presets: &FxHashMap<String, Result<WorldPreset, String>>,
) -> FxHashMap<String, SchemaResult<WorldPresetInfo, String>> {
    asset_info(world_presets, "world_presets", "json")
}
