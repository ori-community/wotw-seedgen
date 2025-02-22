use crate::generate_seed;

use env_logger::Env;
use lazy_static::lazy_static;
use log::info;
use wotw_seedgen_assets::{
    PresetAccess, UniversePreset, UniversePresetSettings, WorldPresetSettings,
};
use wotw_seedgen_logic_language::{
    ast::{parse, Areas},
    output::Graph,
};
use wotw_seedgen_settings::{Difficulty, UniverseSettings};
use wotw_seedgen_static_assets::{
    LOC_DATA, PRESET_ACCESS, SNIPPET_ACCESS, STATE_DATA, UBER_STATE_DATA,
};

lazy_static! {
    pub static ref AREAS: Areas<'static> =
        parse(include_str!("../areas.wotw")).into_result().unwrap();
}

pub fn test_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("trace"))
        .format_timestamp(None)
        .is_test(true)
        .init();
}

#[test]
fn some_seeds() {
    fn generate_test_seed(graph: &Graph, universe_settings: &UniverseSettings) {
        generate_seed(
            &graph,
            &*UBER_STATE_DATA,
            &*SNIPPET_ACCESS,
            universe_settings,
            false,
        )
        .unwrap();
    }

    test_logger();

    let mut universe_settings = UniverseSettings::new("0".to_string());
    let mut graph = Graph::compile(
        AREAS.clone(),
        LOC_DATA.clone(),
        STATE_DATA.clone(),
        &universe_settings.world_settings,
    )
    .into_result()
    .unwrap();
    info!("Testing Default settings");
    generate_test_seed(&graph, &universe_settings);

    universe_settings.world_settings[0].difficulty = Difficulty::Unsafe;
    graph = Graph::compile(
        AREAS.clone(),
        LOC_DATA.clone(),
        STATE_DATA.clone(),
        &universe_settings.world_settings,
    )
    .into_result()
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
        let preset = PRESET_ACCESS.world_preset(preset).unwrap();
        preset
            .apply(&mut universe_settings.world_settings[0], &*PRESET_ACCESS)
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
    preset
        .apply(&mut universe_settings, &*PRESET_ACCESS)
        .unwrap();

    info!("Testing multiworld Gorlek with headers");
    generate_test_seed(&graph, &universe_settings);
}
