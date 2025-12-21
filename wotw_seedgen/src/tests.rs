use std::sync::Once;

use crate::generate_seed;

use env_logger::Env;
use log::info;
use wotw_seedgen_data::{
    assets::{
        AssetCacheValues, AssetFileAccess, PresetAccess, UniversePreset, UniversePresetSettings,
        WorldPresetSettings, TEST_ASSETS,
    },
    logic_language::{ast::Areas, output::Graph},
    Difficulty, UniverseSettings,
};

static LOGGER_INITIALIZED: Once = Once::new();

pub fn test_logger() {
    LOGGER_INITIALIZED.call_once(|| {
        env_logger::Builder::from_env(Env::default().default_filter_or("debug"))
            .format_timestamp(None)
            .is_test(true)
            .init();
    });
}

#[test]
fn some_seeds() {
    test_logger();

    fn generate_test_seed(graph: &Graph, universe_settings: &UniverseSettings) {
        generate_seed(
            &graph,
            TEST_ASSETS.values.loc_data(),
            TEST_ASSETS.values.uber_state_data(),
            &*TEST_ASSETS,
            universe_settings,
            false,
        )
        .unwrap();
    }

    let source = TEST_ASSETS.values.areas();
    let areas = Areas::parse(&source.content).eprint_errors(source).unwrap();

    let mut universe_settings = UniverseSettings::new("0".to_string());
    let mut graph = Graph::compile(
        areas.clone(),
        TEST_ASSETS.loc_data().unwrap(),
        TEST_ASSETS.state_data().unwrap(),
        &universe_settings.world_settings,
    )
    .parsed
    .unwrap();
    info!("Testing Default settings");
    generate_test_seed(&graph, &universe_settings);

    universe_settings.world_settings[0].difficulty = Difficulty::Unsafe;
    graph = Graph::compile(
        areas,
        TEST_ASSETS.loc_data().unwrap(),
        TEST_ASSETS.state_data().unwrap(),
        &universe_settings.world_settings,
    )
    .parsed
    .unwrap();
    info!("Testing Unsafe");
    generate_test_seed(&graph, &universe_settings);

    universe_settings.world_settings[0].snippets.extend([
        "bingo".to_string(),
        "glades_done".to_string(),
        "launch_fragments".to_string(),
        "launch_from_bingo".to_string(),
        "no_combat".to_string(),
        "no_ks_doors".to_string(),
        "no_quests".to_string(),
        "no_willow_hearts".to_string(),
        "trees".to_string(),
        "wisps".to_string(),
    ]);

    for preset in ["gorlek", "rspawn", "full_bonus"] {
        let preset = TEST_ASSETS.world_preset(preset).unwrap();
        preset
            .apply(&mut universe_settings.world_settings[0], &*TEST_ASSETS)
            .unwrap();
    }

    let preset = UniversePreset {
        assets_version: 1,
        info: None,
        settings: UniversePresetSettings {
            world_settings: Some(vec![WorldPresetSettings::default(); 2]),
            ..UniversePresetSettings::default()
        },
    };
    preset.apply(&mut universe_settings, &*TEST_ASSETS).unwrap();

    info!("Testing multiworld Gorlek with snippets");
    generate_test_seed(&graph, &universe_settings);
}
