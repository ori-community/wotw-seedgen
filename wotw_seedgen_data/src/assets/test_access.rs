use std::{
    borrow::Cow,
    iter::{self, Once},
    ops::{Deref, DerefMut},
    path::Path,
    sync::LazyLock,
};

use constcat::concat;
use wotw_seedgen_parse::Source;

use crate::{
    assets::{
        AssetCache, AssetCacheValues, AssetFileAccess, ChangedAssets, DefaultAssetCacheValues,
        LocData, LocDataEntry, PresetAccess, PresetFileAccess, SnippetFileAccess, StateData,
        UberStateData, UniversePreset, WorldPreset,
    },
    logic_language::{
        ast::Paths,
        output::{Anchor, Connection, Graph, Node, Requirement},
    },
    seed_language::simulate::UberStates,
    Difficulty, MapIcon, UberIdentifier, WorldSettings, Zone,
};

const ASSETS: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/../assets");

pub struct TestAccess;

pub static TEST_ASSETS: LazyLock<AssetCache<TestAccess, TestCacheValues>> =
    LazyLock::new(|| AssetCache::new(TestAccess).unwrap());

impl AssetFileAccess for TestAccess {
    type Folders = Once<Self::Path>;
    type Path = &'static Path;

    fn asset_folders(&self) -> Self::Folders {
        iter::once(Path::new(ASSETS))
    }
}

impl SnippetFileAccess for TestAccess {
    type Folders = Once<Self::Path>;
    type Path = &'static Path;

    fn snippet_folders(&self) -> Self::Folders {
        iter::once(Path::new(concat!(ASSETS, "/snippets")))
    }
}

impl PresetFileAccess for TestAccess {
    type Folders = Once<Self::Path>;
    type Path = &'static Path;

    fn universe_folders(&self) -> Self::Folders {
        iter::once(Path::new(concat!(ASSETS, "/universe_presets")))
    }

    fn world_folders(&self) -> Self::Folders {
        iter::once(Path::new(concat!(ASSETS, "/world_presets")))
    }
}

pub struct TestCacheValues {
    pub base: DefaultAssetCacheValues,
    pub uber_states: UberStates,
    pub graphs: TestCacheGraphs,
    test_graph: TestGraph,
}

pub struct TestCacheGraphs {
    pub moki: Graph,
    pub gorlek: Graph,
    pub r#unsafe: Graph,
    pub full: Graph,
}

impl AssetCacheValues for TestCacheValues {
    fn new<F>(file_access: &F) -> Result<Self, String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        let base = DefaultAssetCacheValues::new(file_access)?;
        let uber_states = UberStates::new(&base.uber_state_data);
        let graphs = TestCacheGraphs::new(&base);
        let test_graph = TestGraph::new();

        Ok(Self {
            base,
            uber_states,
            graphs,
            test_graph,
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

    fn update<F>(&mut self, _file_access: &F, _changed: ChangedAssets) -> Result<(), String>
    where
        F: AssetFileAccess + SnippetFileAccess + PresetFileAccess,
    {
        Ok(())
    }
}

impl PresetAccess for TestCacheValues {
    fn universe_preset(&self, identifier: &str) -> Result<UniversePreset, String> {
        self.base.universe_preset(identifier)
    }

    fn world_preset(&self, identifier: &str) -> Result<WorldPreset, String> {
        self.base.world_preset(identifier)
    }

    fn available_universe_presets(&self) -> Vec<String> {
        self.base.available_universe_presets()
    }

    fn available_world_presets(&self) -> Vec<String> {
        self.base.available_world_presets()
    }
}

impl TestCacheValues {
    pub fn graph(&self, settings: &[WorldSettings]) -> Cow<'_, Graph> {
        match settings {
            [] => Cow::Borrowed(&self.graphs.full),
            [world] if world == &WorldSettings::default() => Cow::Borrowed(&self.graphs.moki),
            [world] if world == &WorldSettings::difficulty_default(Difficulty::Gorlek) => {
                Cow::Borrowed(&self.graphs.gorlek)
            }
            other => Cow::Owned(graph(&self.base, other)),
        }
    }

    pub fn test_graph(&self, requirement: Requirement) -> TestGraph {
        let mut graph = self.test_graph.clone();
        graph.set_requirement(requirement);
        graph
    }
}

impl TestCacheGraphs {
    fn new(base: &DefaultAssetCacheValues) -> Self {
        let moki = graph(base, &[WorldSettings::default()]);
        let gorlek = graph(
            base,
            &[WorldSettings::difficulty_default(Difficulty::Gorlek)],
        );
        let r#unsafe = graph(
            base,
            &[WorldSettings::difficulty_default(Difficulty::Unsafe)],
        );
        let full = graph(base, &[]);

        Self {
            moki,
            gorlek,
            r#unsafe,
            full,
        }
    }
}

fn graph(base: &DefaultAssetCacheValues, settings: &[WorldSettings]) -> Graph {
    let paths = Paths::parse(&base.paths.content)
        .eprint_errors(&base.paths)
        .unwrap();

    Graph::compiler()
        .with_settings(settings)
        .compile(paths, base.loc_data.clone(), base.state_data.clone())
        .eprint_errors(&base.paths)
        .unwrap()
}

#[derive(Debug, Clone)]
pub struct TestGraph {
    pub inner: Graph,
}

impl TestGraph {
    pub fn new() -> Self {
        let mut graph = Graph::empty();

        graph.nodes.push(Node::Anchor(Anchor::new(
            "spawn".to_owned(),
            vec![Connection {
                to: 1,
                requirement: Requirement::Free,
                implicitly_generated: false,
            }],
        )));
        graph.nodes.push(Node::Pickup(LocDataEntry {
            identifier: "yummy".to_owned(),
            zone: Zone::Marsh,
            map_icon: MapIcon::SpiritLight,
            uber_identifier: UberIdentifier::new(0, 0),
            value: None,
            position: None,
            map_position: None,
        }));

        Self { inner: graph }
    }

    pub fn set_requirement(&mut self, requirement: Requirement) {
        self.inner.nodes[0].expect_anchor_mut().connections[0].requirement = requirement;
    }

    pub fn get_requirement(&self) -> &Requirement {
        &self.inner.nodes[0].expect_anchor().connections[0].requirement
    }
}

impl Deref for TestGraph {
    type Target = Graph;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for TestGraph {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn validate_logic_csvs() {
        let loc_data = &TEST_ASSETS.base.loc_data;
        let state_data = &TEST_ASSETS.base.state_data;
        let uber_state_data = &TEST_ASSETS.base.uber_state_data;

        for (identifier, uber_identifier, value) in loc_data
            .entries
            .iter()
            .map(|entry| (&entry.identifier, entry.uber_identifier, entry.value))
            .chain(
                state_data
                    .entries
                    .iter()
                    .map(|entry| (&entry.identifier, entry.uber_identifier, entry.value)),
            )
        {
            eprintln!("checking {identifier}");

            let data = &uber_state_data.id_lookup[&uber_identifier];

            match value {
                None => {
                    data.default_value.expect_boolean();
                }
                Some(_) => {
                    data.default_value.expect_integer();
                }
            }
        }
    }
}
