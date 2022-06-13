#![allow(clippy::too_many_arguments)]
#![allow(clippy::cast_precision_loss)]
#![allow(clippy::missing_errors_doc)]
#![allow(clippy::missing_panics_doc)]
#![allow(clippy::match_bool)]
#![allow(clippy::must_use_candidate)]
#![allow(clippy::module_name_repetitions)]
#![allow(clippy::too_many_lines)]
#![allow(clippy::struct_excessive_bools)]

pub mod languages;
pub mod world;
pub mod inventory;
pub mod item;
pub mod preset;
pub mod settings;
pub mod generator;
pub mod util;

pub use languages::{logic, header::{self, Header}};
pub use world::World;
pub use inventory::Inventory;
pub use item::{Item, VItem};
pub use preset::Preset;
pub use settings::Settings;
pub use generator::generate_seed;

#[cfg(test)]
mod tests {
    use crate::{preset::WorldPreset, settings::Difficulty};

    use super::*;

    #[test]
    fn some_seeds() {
        let mut settings = Settings::default();
        let areas = util::read_file("areas.wotw", "logic").unwrap();
        let locations = util::read_file("loc_data.csv", "logic").unwrap();
        let states = util::read_file("state_data.csv", "logic").unwrap();
        let mut graph = logic::parse_logic(&areas, &locations, &states, &settings, false).unwrap();

        generate_seed(&graph, settings.clone()).unwrap();

        settings.world_settings[0].difficulty = Difficulty::Unsafe;
        graph = logic::parse_logic(&areas, &locations, &states, &settings, false).unwrap();
        generate_seed(&graph, settings.clone()).unwrap();

        settings.world_settings[0].headers = vec![
            "bingo".to_string(),
            "bonus+".to_string(),
            "glades_done".to_string(),
            "launch_fragments".to_string(),
            "launch_from_bingo".to_string(),
            "no_combat".to_string(),
            "no_ks_doors".to_string(),
            "no_quests".to_string(),
            "no_willow_hearts".to_string(),
            "open_mode".to_string(),
            "spawn_with_sword".to_string(),
            "util_twillen".to_string(),
            "vanilla_opher_upgrades".to_string(),
            "bonus_opher_upgrades".to_string(),
        ];

        for preset in ["gorlek", "rspawn"] {
            let preset = WorldPreset::read_file(preset.to_string()).unwrap();
            settings.world_settings[0].apply_world_preset(preset).unwrap();
        }

        let preset = Preset {
            world_settings: Some(vec![WorldPreset::default(); 2]),
            ..Preset::default()
        };
        settings.apply_preset(preset).unwrap();

        generate_seed(&graph, settings.clone()).unwrap();
    }
}
