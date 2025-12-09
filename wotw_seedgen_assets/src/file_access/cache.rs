use std::{ffi::OsStr, ops::Deref, path::Path};

use notify_debouncer_full::{
    notify::{EventKind, RecursiveMode},
    DebouncedEvent,
};
use rustc_hash::FxHashMap;

use crate::{
    AssetFileAccess, LocData, PresetAccess, PresetFileAccess, SnippetAccess, SnippetFileAccess,
    Source, StateData, UberStateData, UniversePreset, Watcher, WatcherError, WorldPreset,
};

pub struct AssetCache<F, V> {
    file_access: F,
    values: V,
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

    pub fn watch(&self, watcher: &mut Watcher) -> Result<(), WatcherError> {
        for folder in AssetFileAccess::folders(&self.file_access) {
            watcher.watch(folder, RecursiveMode::Recursive)?;
        }

        Ok(())
    }

    pub fn update_from_watcher_event(&mut self, events: &[DebouncedEvent]) -> Result<bool, String> {
        let mut changed = ChangedAssets::default();

        for event in events {
            for path in &event.paths {
                let Ok(path) = path.canonicalize() else {
                    continue;
                };

                if path.ends_with(F::LOC_DATA_PATH) {
                    changed.loc_data = true;
                } else if path.ends_with(F::STATE_DATA_PATH) {
                    changed.state_data = true;
                } else if path.ends_with(F::UBER_STATE_DUMP_PATH) {
                    changed.loc_data = true;
                    changed.state_data = true;
                    changed.uber_state_dump = true;
                } else if path.ends_with(F::AREAS_PATH) {
                    changed.areas = true;
                } else {
                    let folders = SnippetFileAccess::folders(&self.file_access);
                    subfolder_changed(&mut changed.snippets, &path, folders, "wotws", event.kind);

                    let folders = PresetFileAccess::universe_folders(&self.file_access);
                    subfolder_changed(
                        &mut changed.universe_presets,
                        &path,
                        folders,
                        "json",
                        event.kind,
                    );

                    let folders = PresetFileAccess::world_folders(&self.file_access);
                    subfolder_changed(
                        &mut changed.world_presets,
                        &path,
                        folders,
                        "json",
                        event.kind,
                    );
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

    fn folders(&self) -> Self::Folders {
        self.file_access.folders()
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

    fn areas(&self) -> Result<Source, String> {
        Ok(self.values.areas().clone())
    }
}

pub trait AssetCacheValues: Sized {
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess;

    fn loc_data(&self) -> &LocData;

    fn state_data(&self) -> &StateData;

    fn uber_state_data(&self) -> &UberStateData;

    fn areas(&self) -> &Source;

    fn snippet(&self, identifier: &str) -> Result<&Source, String>;

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
    pub areas: bool,
    pub snippets: Vec<(String, EventKind)>,
    pub universe_presets: Vec<(String, EventKind)>,
    pub world_presets: Vec<(String, EventKind)>,
}

fn subfolder_changed(
    identifiers: &mut Vec<(String, EventKind)>,
    path: &Path,
    mut folders: impl Iterator<Item = impl AsRef<Path>>,
    extension: &str,
    kind: EventKind,
) {
    if path.extension() != Some(OsStr::new(extension)) {
        return;
    }

    let Some(parent) = path.parent() else { return };

    if !folders.any(|folder| parent.ends_with(folder)) {
        return;
    }

    identifiers.push((
        path.file_stem().unwrap().to_str().unwrap().to_string(),
        kind,
    ));
}

impl<F: SnippetFileAccess, V: AssetCacheValues> SnippetAccess for AssetCache<F, V> {
    fn read_snippet(&self, identifier: &str) -> Result<Source, String> {
        self.values.snippet(identifier).cloned()
    }

    fn read_file(&self, path: &Path) -> Result<Vec<u8>, String> {
        self.file_access.read_file(path)
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
    pub areas: Source,
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
        let areas = file_access.areas()?;

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
            areas,
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

    fn areas(&self) -> &Source {
        &self.areas
    }

    fn update<F>(&mut self, file_access: &F, changed: ChangedAssets) -> Result<(), String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let ChangedAssets {
            loc_data,
            state_data,
            uber_state_dump,
            areas,
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

        if areas {
            self.areas = file_access.areas()?;
        }

        update_subfolder(snippets, &mut self.snippets, |identifier| {
            file_access.read_snippet(&identifier)
        })?;

        update_subfolder(universe_presets, &mut self.universe_presets, |identifier| {
            file_access.universe_preset(&identifier)
        })?;

        update_subfolder(world_presets, &mut self.world_presets, |identifier| {
            file_access.world_preset(&identifier)
        })?;

        Ok(())
    }

    fn snippet(&self, identifier: &str) -> Result<&Source, String> {
        self.snippets
            .get(identifier)
            .ok_or_else(|| format!("unknown snippet \"{identifier}\""))
    }

    fn available_snippets(&self) -> impl Iterator<Item = &String> {
        self.snippets.keys()
    }
}

fn update_subfolder<F, V>(
    identifiers: Vec<(String, EventKind)>,
    values: &mut FxHashMap<String, V>,
    mut f: F,
) -> Result<(), String>
where
    F: FnMut(&str) -> Result<V, String>,
{
    for (identifier, kind) in identifiers {
        if kind.is_remove() {
            values.remove(&identifier);
        } else {
            let source = f(&identifier)?;
            values.insert(identifier, source);
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
