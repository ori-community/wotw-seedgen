use std::collections::hash_map::Entry;

use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;

use crate::{
    assets::{SnippetAccess, TEST_ASSETS},
    UniverseSettings, WorldSettings,
};

#[test]
fn slugification() {
    let mut rng = Pcg64Mcg::new(0xcafef00dd15ea5e5);
    let mut slugs = FxHashMap::default();
    let mut universe_settings = UniverseSettings {
        seed: String::new(),
        world_settings: Vec::with_capacity(1),
    };

    let snippets = TEST_ASSETS.available_snippets_metadata();

    for count in 1..1000 {
        universe_settings
            .world_settings
            .push(WorldSettings::random_with_metadata(&mut rng, &snippets));

        let slug = universe_settings.slugify();

        match slugs.entry(slug) {
            Entry::Occupied(occupied) => {
                panic!(
                    "After {count} settings, two had the same slug: {slug}\nSettings a: {a:?}\nSettings b: {b:?}",
                    slug = occupied.key(),
                    a = occupied.get(),
                    b = &universe_settings.world_settings[0],
                );
            }
            Entry::Vacant(vacant) => {
                vacant.insert(universe_settings.world_settings.pop().unwrap());
            }
        }
    }
}
