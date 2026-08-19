use std::{
    array, fs, io,
    ops::Deref,
    path::{Path, PathBuf},
};

use itertools::Itertools;
use log::warn;
use notify_debouncer_full::{
    notify::{
        event::{CreateKind, DataChange, ModifyKind, RemoveKind, RenameMode},
        Event, EventKind, RecursiveMode,
    },
    DebouncedEvent,
};
use rustc_hash::FxHashMap;
use wotw_seedgen_parse::Source;

use crate::assets::{
    file_err, AssetFileAccess, LocData, PresetAccess, PresetFileAccess, SnippetAccess,
    SnippetFileAccess, StateData, UberStateData, UniversePreset, Watcher, WatcherResult,
    WorldPreset,
};

pub struct AssetCache<F, V> {
    file_access: F,
    pub values: V,
}

impl<F: AssetFileAccess + SnippetFileAccess + PresetFileAccess, V: AssetCacheValues>
    AssetCache<F, V>
{
    pub fn new(file_access: F) -> Result<Self, String> {
        let values = V::new(&file_access)?;

        Ok(Self {
            file_access,
            values,
        })
    }

    pub fn watch(&self, watcher: &mut Watcher) -> WatcherResult<()> {
        for folder in self.file_access.asset_folders() {
            watcher.watch(folder, RecursiveMode::NonRecursive)?;
        }

        for folder in self.file_access.snippet_folders() {
            watcher.watch(folder, RecursiveMode::Recursive)?;
        }

        for folder in self.file_access.universe_folders() {
            watcher.watch(folder, RecursiveMode::NonRecursive)?;
        }

        for folder in self.file_access.world_folders() {
            watcher.watch(folder, RecursiveMode::NonRecursive)?;
        }

        Ok(())
    }

    pub fn update_from_watcher_event(
        &mut self,
        events: Vec<DebouncedEvent>,
    ) -> Result<bool, String> {
        let mut changed = ChangedAssets::default();

        for debounced in events {
            let Event { kind, paths, .. } = debounced.event;

            match kind {
                EventKind::Create(CreateKind::Any | CreateKind::File) => changed.create(paths, &self.file_access),
                EventKind::Modify(ModifyKind::Data(DataChange::Any | DataChange::Content) | ModifyKind::Any) => changed.modify(paths, &self.file_access),
                EventKind::Modify(ModifyKind::Name(RenameMode::To)) => changed.create(paths, &self.file_access),
                EventKind::Modify(ModifyKind::Name(RenameMode::From)) => changed.remove(paths, &self.file_access),
                EventKind::Modify(ModifyKind::Name(RenameMode::Both)) => changed.rename(paths, &self.file_access),
                EventKind::Remove(RemoveKind::File) => changed.remove(paths, &self.file_access),
                EventKind::Access(_)
                | EventKind::Create(CreateKind::Folder)
                | EventKind::Modify(ModifyKind::Data(DataChange::Size) | ModifyKind::Metadata(_))
                // TODO remove folder could be relevant for contained assets, something we generally don't handle well
                | EventKind::Remove(RemoveKind::Folder) => continue,
                EventKind::Any
                | EventKind::Other
                | EventKind::Create(CreateKind::Other)
                | EventKind::Modify(
                    ModifyKind::Data(DataChange::Other)
                    | ModifyKind::Name(RenameMode::Any | RenameMode::Other)
                    | ModifyKind::Other,
                )
                | EventKind::Remove(RemoveKind::Any | RemoveKind::Other) => {
                    warn!("unprocessable file event {kind:?}");

                    continue;
                }
            }
        }

        let any_changed = changed != ChangedAssets::default();

        self.values.update(&self.file_access, changed)?;

        Ok(any_changed)
    }
}

impl<F, V> Deref for AssetCache<F, V> {
    type Target = V;

    fn deref(&self) -> &Self::Target {
        &self.values
    }
}

impl<F: AssetFileAccess, V: AssetCacheValues> AssetFileAccess for AssetCache<F, V> {
    type Folders = F::Folders;
    type Path = F::Path;

    fn asset_folders(&self) -> Self::Folders {
        self.file_access.asset_folders()
    }

    fn loc_data(&self) -> Result<LocData, String> {
        Ok(self.values.loc_data().clone())
    }

    fn state_data(&self) -> Result<StateData, String> {
        Ok(self.values.state_data().clone())
    }

    fn uber_state_data(
        &self,
        _loc_data: &LocData,
        _state_data: &StateData,
    ) -> Result<UberStateData, String> {
        Ok(self.values.uber_state_data().clone())
    }

    fn paths(&self) -> Result<Source, String> {
        Ok(self.values.paths().clone())
    }
}

pub trait AssetCacheValues: Sized {
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess;

    fn loc_data(&self) -> &LocData;

    fn state_data(&self) -> &StateData;

    fn uber_state_data(&self) -> &UberStateData;

    fn paths(&self) -> &Source;

    fn snippet(&self, identifier: &str) -> Result<&Source, String>;

    fn allow_read_file(&self) -> bool;

    fn available_snippets(&self) -> impl Iterator<Item = &String>;

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets) -> Result<(), String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess;
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ChangedAssets {
    pub loc_data: bool,
    pub state_data: bool,
    pub uber_state_dump: bool,
    pub paths: bool,
    pub snippets: Vec<ChangeDetails>,
    pub universe_presets: Vec<ChangeDetails>,
    pub world_presets: Vec<ChangeDetails>,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ChangeDetails {
    Create(String),
    Modify(String),
    Remove(String),
    Rename(String, String),
}

impl ChangedAssets {
    fn create<F>(&mut self, paths: Vec<PathBuf>, file_access: &F)
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        self.update_single_path(paths, "create", ChangeDetails::Create, file_access);
    }

    fn remove<F>(&mut self, paths: Vec<PathBuf>, file_access: &F)
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        self.update_single_path(paths, "remove", ChangeDetails::Remove, file_access);
    }

    fn rename<F>(&mut self, paths: Vec<PathBuf>, file_access: &F)
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let Some([from, to]) = validate_paths(paths, "rename") else {
            return;
        };

        let from_kind = PathKind::detect(&from, file_access);
        let to_kind = PathKind::detect(&to, file_access);

        let rename_for_kind = |changes: &mut Vec<ChangeDetails>, kind: PathKind| match (
            from_kind == Some(kind),
            to_kind == Some(kind),
        ) {
            (false, false) => {}
            (true, false) => changes.push(ChangeDetails::Remove(to_identifier(&from))),
            (false, true) => changes.push(ChangeDetails::Create(to_identifier(&to))),
            (true, true) => {
                let details = ChangeDetails::Rename(to_identifier(&from), to_identifier(&to));
                changes.push(details);
            }
        };

        rename_for_kind(&mut self.snippets, PathKind::Snippet);
        rename_for_kind(&mut self.universe_presets, PathKind::UniversePreset);
        rename_for_kind(&mut self.world_presets, PathKind::WorldPreset);

        if let Some(path_kind) = from_kind {
            self.update_known_files(path_kind);
        }
        if let Some(path_kind) = to_kind {
            self.update_known_files(path_kind);
        }
    }

    fn modify<F>(&mut self, paths: Vec<PathBuf>, file_access: &F)
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        self.update_single_path(paths, "modify", ChangeDetails::Modify, file_access);
    }

    fn update_single_path<D, F>(
        &mut self,
        paths: Vec<PathBuf>,
        kind: &str,
        details: D,
        file_access: &F,
    ) where
        D: FnOnce(String) -> ChangeDetails,
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let Some([path]) = validate_paths(paths, kind) else {
            return;
        };

        let Some(path_kind) = PathKind::detect(&path, file_access) else {
            return;
        };

        match path_kind {
            PathKind::Snippet => self.snippets.push(details(to_identifier(&path))),
            PathKind::UniversePreset => self.universe_presets.push(details(to_identifier(&path))),
            PathKind::WorldPreset => self.world_presets.push(details(to_identifier(&path))),
            _ => self.update_known_files(path_kind),
        }
    }

    fn update_known_files(&mut self, path_kind: PathKind) {
        match path_kind {
            PathKind::LocData => self.loc_data = true,
            PathKind::StateData => self.state_data = true,
            PathKind::UberStateDump => self.uber_state_dump = true,
            PathKind::Paths => self.paths = true,
            _ => {}
        }
    }
}

fn validate_paths<const N: usize>(paths: Vec<PathBuf>, kind: &str) -> Option<[PathBuf; N]> {
    if paths.len() == N {
        let mut iter = paths.into_iter();
        Some(array::from_fn(|_| iter.next().unwrap()))
    } else {
        warn!(
            "unprocessable {kind} event paths [{paths}]",
            paths = paths.iter().map(|path| path.display()).format(", ")
        );

        None
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum PathKind {
    LocData,
    StateData,
    UberStateDump,
    Paths,
    Snippet,
    UniversePreset,
    WorldPreset,
}

impl PathKind {
    fn detect<F>(path: &Path, file_access: &F) -> Option<Self>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        if path.ends_with(F::LOC_DATA_PATH) {
            Some(Self::LocData)
        } else if path.ends_with(F::STATE_DATA_PATH) {
            Some(Self::StateData)
        } else if path.ends_with(F::UBER_STATE_DUMP_PATH) {
            Some(Self::UberStateDump)
        } else if path.ends_with(F::PATHS_PATH) {
            Some(Self::Paths)
        } else {
            let extension = path.extension()?;

            if extension == "wotws" {
                is_in_folders(path, file_access.snippet_folders()).then_some(Self::Snippet)
            } else if extension == "json" {
                if is_in_folders(path, file_access.world_folders()) {
                    Some(Self::WorldPreset)
                } else if is_in_folders(path, file_access.universe_folders()) {
                    Some(Self::UniversePreset)
                } else {
                    None
                }
            } else {
                None
            }
        }
    }
}

fn is_in_folders(path: &Path, mut folders: impl Iterator<Item = impl AsRef<Path>>) -> bool {
    folders.any(|folder| fs::canonicalize(folder).is_ok_and(|folder| path.starts_with(folder)))
}

fn to_identifier(path: &Path) -> String {
    path.file_stem().unwrap().to_str().unwrap().to_string()
}

impl<F: SnippetAccess, V: AssetCacheValues> SnippetAccess for AssetCache<F, V> {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        self.values.snippet(identifier).cloned()
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        if self.values.allow_read_file() {
            self.file_access.read_file(path)
        } else {
            Err(file_err(
                "read",
                path,
                io::Error::new(
                    io::ErrorKind::PermissionDenied,
                    "file includes are not allowed in this context",
                ),
            ))
        }
    }

    fn available_snippets(&self) -> Vec<String> {
        self.values.available_snippets().cloned().collect()
    }
}

impl<F, V: PresetAccess> PresetAccess for AssetCache<F, V> {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        self.values.universe_preset(identifier)
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        self.values.world_preset(identifier)
    }

    fn available_universe_presets(&self) -> Vec<String> {
        self.values.available_universe_presets()
    }

    fn available_world_presets(&self) -> Vec<String> {
        self.values.available_world_presets()
    }
}

pub struct DefaultAssetCacheValues {
    pub loc_data: LocData,
    pub state_data: StateData,
    pub uber_state_data: UberStateData,
    pub paths: Source,
    pub snippets: FxHashMap<String, Source>,
    pub universe_presets: FxHashMap<String, UniversePreset>,
    pub world_presets: FxHashMap<String, WorldPreset>,
}

impl AssetCacheValues for DefaultAssetCacheValues {
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let loc_data = file_access.loc_data()?;
        let state_data = file_access.state_data()?;
        let uber_state_data = file_access.uber_state_data(&loc_data, &state_data)?;
        let paths = file_access.paths()?;

        let snippets = file_access
            .available_snippets()
            .into_iter()
            .map(|identifier| {
                file_access
                    .read_snippet(&identifier)
                    .map(|source| (identifier, source))
            })
            .collect::<Result<_, _>>()?;

        let universe_presets = file_access
            .available_universe_presets()
            .into_iter()
            .map(|identifier| {
                file_access
                    .universe_preset(&identifier)
                    .map(|universe_preset| (identifier, universe_preset))
            })
            .collect::<Result<_, _>>()?;

        let world_presets = file_access
            .available_world_presets()
            .into_iter()
            .map(|identifier| {
                file_access
                    .world_preset(&identifier)
                    .map(|universe_preset| (identifier, universe_preset))
            })
            .collect::<Result<_, _>>()?;

        Ok(Self {
            loc_data,
            state_data,
            uber_state_data,
            paths,
            snippets,
            universe_presets,
            world_presets,
        })
    }

    fn loc_data(&self) -> &LocData {
        &self.loc_data
    }

    fn state_data(&self) -> &StateData {
        &self.state_data
    }

    fn uber_state_data(&self) -> &UberStateData {
        &self.uber_state_data
    }

    fn paths(&self) -> &Source {
        &self.paths
    }

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets) -> Result<(), String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let ChangedAssets {
            loc_data,
            state_data,
            uber_state_dump,
            paths,
            snippets,
            universe_presets,
            world_presets,
        } = changed;

        if uber_state_dump || loc_data {
            self.loc_data = file_access.loc_data()?;
        }

        if uber_state_dump || state_data {
            self.state_data = file_access.state_data()?;
        }

        if uber_state_dump {
            self.uber_state_data = file_access.uber_state_data(&self.loc_data, &self.state_data)?;
        }

        if paths {
            self.paths = file_access.paths()?;
        }

        update_subfolder(snippets, &mut self.snippets, |identifier| {
            file_access.read_snippet(identifier)
        })?;

        update_subfolder(universe_presets, &mut self.universe_presets, |identifier| {
            file_access.universe_preset(identifier)
        })?;

        update_subfolder(world_presets, &mut self.world_presets, |identifier| {
            file_access.world_preset(identifier)
        })?;

        Ok(())
    }

    fn snippet(&self, identifier: &str) -> Result<&Source, String> {
        self.snippets
            .get(identifier)
            .ok_or_else(|| format!("unknown snippet \"{identifier}\""))
    }

    fn allow_read_file(&self) -> bool {
        true
    }

    fn available_snippets(&self) -> impl Iterator<Item = &String> {
        self.snippets.keys()
    }
}

fn update_subfolder<F, V>(
    changes: Vec<ChangeDetails>,
    values: &mut FxHashMap<String, V>,
    mut f: F,
) -> Result<(), String>
where
    F: FnMut(&str) -> Result<V, String>,
{
    for change in changes {
        match change {
            ChangeDetails::Create(identifier) | ChangeDetails::Modify(identifier) => {
                let value = f(&identifier)?;
                values.insert(identifier, value);
            }
            ChangeDetails::Remove(identifier) => {
                values.remove(&identifier);
            }
            ChangeDetails::Rename(from, to) => {
                let value = values.remove(&from).unwrap();
                values.insert(to, value);
            }
        }
    }

    Ok(())
}

impl PresetAccess for DefaultAssetCacheValues {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        self.universe_presets
            .get(identifier)
            .cloned()
            .ok_or_else(|| format!("unknown universe preset \"{identifier}\""))
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        self.world_presets
            .get(identifier)
            .cloned()
            .ok_or_else(|| format!("unknown world preset \"{identifier}\""))
    }

    fn available_universe_presets(&self) -> Vec<String> {
        self.universe_presets.keys().cloned().collect()
    }

    fn available_world_presets(&self) -> Vec<String> {
        self.world_presets.keys().cloned().collect()
    }
}

impl SnippetAccess for DefaultAssetCacheValues {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        self.snippets
            .get(identifier)
            .cloned()
            .ok_or_else(|| format!("unknown snippet \"{identifier}\""))
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        Err(format!(
            "tried to read non-default file \"{}\" directly from cache",
            path.display()
        ))
    }

    fn available_snippets(&self) -> Vec<String> {
        self.snippets.keys().cloned().collect()
    }
}
