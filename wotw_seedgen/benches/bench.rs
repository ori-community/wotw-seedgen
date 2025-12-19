use std::{sync::LazyLock, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use smallvec::smallvec;
use wotw_seedgen::{item_pool::ItemPool, World};
use wotw_seedgen_data::{
    assets::{
        AssetCacheValues, AssetFileAccess, PresetAccess, SnippetAccess, WorldPreset,
        WorldPresetSettings, TEST_ASSETS,
    },
    logic_language::{
        ast::Areas,
        output::{Enemy, Graph, Requirement},
    },
    seed_language::{
        ast::Snippet,
        compile::Compiler,
        simulate::{Simulation, UberStates},
    },
    Difficulty, Skill, Spawn, UniverseSettings, WorldSettings, DEFAULT_SPAWN,
};

static AREAS: LazyLock<Areas> = LazyLock::new(|| {
    let source = TEST_ASSETS.values.areas();
    Areas::parse(&source.content).eprint_errors(source).unwrap()
});

fn logic_assets(c: &mut Criterion) {
    let mut group = c.benchmark_group("logic_assets");

    group.bench_function("areas", |b| {
        b.iter(|| Areas::parse(include_str!("../areas.wotw")))
    });

    let areas = &*AREAS;
    let loc_data = TEST_ASSETS.loc_data().unwrap();
    let state_data = TEST_ASSETS.state_data().unwrap();

    group.bench_function("compile", |b| {
        b.iter(|| Graph::compile(areas.clone(), loc_data.clone(), state_data.clone(), &[]))
    });

    group.finish();
}

fn snippets(c: &mut Criterion) {
    let mut group = c.benchmark_group("snippets");

    let stats = TEST_ASSETS.read_snippet("stats").unwrap();

    group.bench_function("ast_stats", |b| b.iter(|| Snippet::parse(&stats.content)));

    let available_snippets = TEST_ASSETS.available_snippets();
    let snippet_sources = available_snippets
        .iter()
        .map(|identifier| TEST_ASSETS.read_snippet(identifier).unwrap())
        .collect::<Vec<_>>();

    group.bench_function("ast_snippets", |b| {
        b.iter(|| {
            snippet_sources
                .iter()
                .map(|source| Snippet::parse(&source.content))
                .collect::<Vec<_>>()
        })
    });

    group.bench_function("compile_snippets", |b| {
        b.iter(|| {
            let mut rng = Pcg64Mcg::new(0);
            let mut compiler = Compiler::new(
                &mut rng,
                &*TEST_ASSETS,
                TEST_ASSETS.values.uber_state_data(),
                FxHashMap::default(),
                false,
            );

            for identifier in &available_snippets {
                compiler.compile_snippet(&identifier).unwrap();
            }

            compiler.finish()
        })
    });
}

fn requirements(c: &mut Criterion) {
    let mut group = c.benchmark_group("requirements");

    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    let graph = compile_graph(&[]);
    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    let uber_states = UberStates::new(TEST_ASSETS.values.uber_state_data());
    let mut world = World::new(&graph, spawn, &world_settings, uber_states);

    let req_a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let req_b = Requirement::Damage(20.0);
    let req_c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let req_d = Requirement::Damage(10.0);
    world.store_skill(Skill::Blaze, true, &[]);
    world.add_max_health(20, &[]);
    world.add_max_energy((2.).into(), &[]);
    let requirement = Requirement::And(vec![
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
    ]);
    group.bench_function("nesting", |b| {
        b.iter(|| world.is_met(&requirement, &mut smallvec![world.max_orbs()]))
    });

    world.store_skill(Skill::Bow, true, &[]);
    world.add_max_energy((10.).into(), &[]);
    let requirement = Requirement::Combat(smallvec![(Enemy::Lizard, 3),]);
    group.bench_function("short_combat", |b| {
        b.iter(|| world.is_met(&requirement, &mut smallvec![world.max_orbs()]))
    });

    // TODO reenable if this concept is added again
    // let requirement = Requirement::And(vec![
    //     Requirement::Combat(smallvec![
    //         (Enemy::Mantis, 2),
    //         (Enemy::Lizard, 2),
    //         (Enemy::EnergyRefill, 4),
    //         (Enemy::SneezeSlug, 2),
    //         (Enemy::Mantis, 1),
    //         (Enemy::Skeeto, 1),
    //         (Enemy::EnergyRefill, 4),
    //         (Enemy::SmallSkeeto, 7),
    //         (Enemy::Skeeto, 2),
    //         (Enemy::EnergyRefill, 4),
    //         (Enemy::Lizard, 2),
    //         (Enemy::Mantis, 2),
    //     ]),
    //     Requirement::Damage(50.0),
    // ]);
    // player.inventory.clear();
    // group.bench_function("long_combat_progression", |b| {
    //     b.iter(|| {
    //         player.solutions(
    //             &requirement,
    //             &states,
    //             smallvec![player.max_orbs()],
    //             1000,
    //             1000,
    //         )
    //     })
    // });

    group.finish();
}

fn reach_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("reach_check");

    let uber_states = UberStates::new(TEST_ASSETS.values.uber_state_data());
    let world_settings = WorldSettings::default();
    let graph = compile_graph(&[]);
    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    let world = World::new(&graph, spawn, &world_settings, uber_states.clone());

    group.bench_function("short", |b| {
        b.iter(|| {
            let mut world = world.clone();
            world.traverse_spawn(&[]);
            world.store_spirit_light(10000, &[]);
            world.store_max_health(200, &[]);
            world.store_max_energy(20.0.into(), &[]);
            world.store_keystones(34, &[]);
            world.store_gorlek_ore(40, &[]);
            world.store_shard_slots(8, &[]);
            world.store_skill(Skill::Sword, true, &[]);
            world.store_skill(Skill::DoubleJump, true, &[]);
            world.store_skill(Skill::Dash, true, &[]);
            world.reached_nodes().for_each(drop);
        })
    });

    let mut pool = ItemPool::new(&mut Pcg64Mcg::new(0));

    group.bench_function("long", |b| {
        b.iter(|| {
            let mut world = world.clone();
            world.traverse_spawn(&[]);
            for item in pool.drain(..) {
                world.simulate(&item, &[]);
            }
            world.reached_nodes().for_each(drop);
        })
    });
}

fn generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation");
    group.measurement_time(Duration::from_secs(10));

    let mut universe_settings = UniverseSettings::new(String::default());
    let mut seed = 0..;
    let graph = compile_graph(&universe_settings.world_settings);
    let test_assets = &*TEST_ASSETS;
    let loc_data = test_assets.values.loc_data();
    let uber_state_data = test_assets.values.uber_state_data();

    group.bench_function("default", |b| {
        b.iter(|| {
            universe_settings.seed = seed.next().unwrap().to_string();
            wotw_seedgen::generate_seed(
                &graph,
                loc_data,
                uber_state_data,
                test_assets,
                &universe_settings,
                false,
            )
            .unwrap()
        })
    });

    seed = 0..;
    let preset = TEST_ASSETS.world_preset("moki").unwrap();
    preset
        .apply(&mut universe_settings.world_settings[0], &*TEST_ASSETS)
        .unwrap();

    group.bench_function("moki", |b| {
        b.iter(|| {
            universe_settings.seed = seed.next().unwrap().to_string();
            wotw_seedgen::generate_seed(
                &graph,
                loc_data,
                uber_state_data,
                test_assets,
                &universe_settings,
                false,
            )
            .unwrap()
        })
    });

    seed = 0..;
    let mut universe_settings = UniverseSettings::new(String::default());
    let preset = WorldPreset {
        assets_version: 1,
        info: None,
        settings: WorldPresetSettings {
            includes: Some(
                ["gorlek".to_string(), "rspawn".to_string()]
                    .into_iter()
                    .collect(),
            ),
            difficulty: Some(Difficulty::Unsafe),
            spawn: Some(Spawn::FullyRandom),
            ..Default::default()
        },
    };
    preset
        .apply(&mut universe_settings.world_settings[0], &*TEST_ASSETS)
        .unwrap();
    let graph = compile_graph(&universe_settings.world_settings);
    group.bench_function("unsafe", |b| {
        b.iter(|| {
            universe_settings.seed = seed.next().unwrap().to_string();
            wotw_seedgen::generate_seed(
                &graph,
                loc_data,
                uber_state_data,
                test_assets,
                &universe_settings,
                false,
            )
            .unwrap()
        })
    });

    group.finish();
}

fn multiworld(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiworld");

    let mut universe_settings = UniverseSettings::new(String::default());
    let preset = TEST_ASSETS.world_preset("gorlek").unwrap();
    preset
        .apply(&mut universe_settings.world_settings[0], &*TEST_ASSETS)
        .unwrap();
    let graph = compile_graph(&universe_settings.world_settings);

    let test_assets = &*TEST_ASSETS;
    let loc_data = test_assets.values.loc_data();
    let uber_state_data = test_assets.values.uber_state_data();

    let world_settings = universe_settings.world_settings.pop().unwrap();
    let mut seed = 0..;

    for worlds in (0..5).map(|x| 2_usize.pow(x)) {
        group.throughput(Throughput::Elements(worlds as u64));
        group.bench_with_input(BenchmarkId::from_parameter(worlds), &worlds, |b, worlds| {
            universe_settings.world_settings = vec![world_settings.clone(); *worlds];
            b.iter(|| {
                universe_settings.seed = seed.next().unwrap().to_string();
                wotw_seedgen::generate_seed(
                    &graph,
                    loc_data,
                    uber_state_data,
                    test_assets,
                    &universe_settings,
                    false,
                )
                .unwrap()
            });
        });
    }

    group.finish();
}

fn compile_graph(settings: &[WorldSettings]) -> Graph {
    Graph::compile(
        AREAS.clone(),
        TEST_ASSETS.loc_data().unwrap(),
        TEST_ASSETS.state_data().unwrap(),
        settings,
    )
    .parsed
    .unwrap()
}

criterion_group!(
    all,
    logic_assets,
    snippets,
    requirements,
    reach_check,
    generation,
    multiworld
);
criterion_main!(all);
