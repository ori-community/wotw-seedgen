use std::{slice, sync::LazyLock, time::Duration};

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use rand_pcg::Pcg64Mcg;
use rustc_hash::{FxHashMap, FxHashSet};
use smallvec::smallvec;
use wotw_seedgen::{item_pool::ItemPoolBuilder, World};
use wotw_seedgen_data::{
    assets::{
        AssetCacheValues, AssetFileAccess, PresetAccess, SnippetAccess, WorldPreset,
        WorldPresetSettings, TEST_ASSETS,
    },
    logic_language::{
        ast::Paths,
        output::{Enemy, Graph, Requirement},
    },
    seed_language::{
        ast::Snippet,
        compile::{self, Compiler},
        output::CommandInteger,
        simulate::{Simulation, Snapshot, WorldState},
    },
    Difficulty, Skill, Spawn, UberIdentifier, UniverseSettings, WorldSettings, DEFAULT_SPAWN,
};

static PATHS: LazyLock<Paths> = LazyLock::new(|| {
    let source = TEST_ASSETS.values.paths();
    Paths::parse(&source.content).eprint_errors(source).unwrap()
});

fn logic_assets(c: &mut Criterion) {
    let mut group = c.benchmark_group("logic_assets");

    group.bench_function("paths", |b| {
        b.iter(|| Paths::parse(include_str!("../../assets/logic/paths.wotwl")))
    });

    let paths = &*PATHS;
    let loc_data = TEST_ASSETS.loc_data().unwrap();
    let state_data = TEST_ASSETS.state_data().unwrap();

    group.bench_function("compile", |b| {
        b.iter(|| Graph::compile(paths.clone(), loc_data.clone(), state_data.clone(), &[]))
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
                None,
                true,
                false,
            );

            for identifier in &available_snippets {
                compiler.compile_snippet(&identifier).unwrap();
            }

            compiler.finish()
        })
    });

    group.finish();
}

fn is_met(c: &mut Criterion) {
    let mut group = c.benchmark_group("is_met");

    let world_settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    let mut world = spawnless_world(&TEST_ASSETS.graphs.moki, &world_settings);

    let req_a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let req_b = Requirement::Damage(20.0);
    let req_c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let req_d = Requirement::Damage(10.0);
    world.store_skill(Skill::Blaze, true, &[]);
    world.add_base_max_health(20, &[]);
    world.add_base_max_energy((2.).into(), &[]);
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
    world.add_base_max_energy((10.).into(), &[]);
    let requirement = Requirement::Combat(smallvec![(Enemy::Lizard, 3),]);
    group.bench_function("short_combat", |b| {
        b.iter(|| world.is_met(&requirement, &mut smallvec![world.max_orbs()]))
    });

    group.finish();
}

fn solutions(c: &mut Criterion) {
    let mut group = c.benchmark_group("solutions");

    let world_settings = WorldSettings::default();
    // TODO maybe later :p
    // let mut world_settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    // world_settings.tricks.extend(Trick::VARIANTS);
    let graph = TEST_ASSETS.graph(slice::from_ref(&world_settings));
    let item_pool = ItemPoolBuilder::new(&mut Pcg64Mcg::new(0)).finish();

    const SPAWNS: [(&str, &str); 14] = [
        ("marsh", "MarshSpawn.Main"),
        ("den", "HowlsDen.Teleporter"),
        ("hollow", "EastHollow.Teleporter"),
        ("glades", "GladesTown.Teleporter"),
        ("wellspring", "InnerWellspring.Teleporter"),
        ("woods_entrance", "WoodsEntry.Teleporter"),
        ("woods_exit", "WoodsMain.Teleporter"),
        ("reach", "LowerReach.Teleporter"),
        ("depths", "UpperDepths.Teleporter"),
        ("pools", "EastPools.Teleporter"),
        ("feeding_grounds", "LowerWastes.FeedingGroundsTP"),
        ("central_wastes", "LowerWastes.CentralTP"),
        ("willow", "WillowsEnd.InnerTP"),
        ("burrows", "MidnightBurrows.Teleporter"),
    ];

    for (id, spawn) in SPAWNS {
        let mut world = world(&graph, &world_settings, spawn);
        world.traverse_spawn(&[]);

        group.bench_function(id, |b| {
            b.iter(|| world.find_solutions(&item_pool, &[], 7, 7, None))
        });
    }

    group.finish();
}

fn reach_check(c: &mut Criterion) {
    let mut group = c.benchmark_group("reach_check");

    let world_settings = WorldSettings::default();
    let mut world = world(&TEST_ASSETS.graphs.moki, &world_settings, DEFAULT_SPAWN);

    group.bench_function("short", |b| {
        b.iter(|| {
            world.snapshot();
            world.traverse_spawn(&[]);
            world.store_spirit_light(10000, &[]);
            world.store_base_max_health(200, &[]);
            world.store_base_max_energy(20.0.into(), &[]);
            world.store_keystones(34, &[]);
            world.store_gorlek_ore(40, &[]);
            world.store_shard_slots(8, &[]);
            world.store_skill(Skill::Sword, true, &[]);
            world.store_skill(Skill::DoubleJump, true, &[]);
            world.store_skill(Skill::Dash, true, &[]);
            world.reached_nodes().for_each(drop);
            world.restore_snapshot();
        })
    });

    let item_pool = ItemPoolBuilder::new(&mut Pcg64Mcg::new(0)).finish();

    group.bench_function("long", |b| {
        b.iter(|| {
            world.snapshot();
            world.traverse_spawn(&[]);
            for item in item_pool.clone().take() {
                world.simulate(&item, &[]);
            }
            world.reached_nodes().for_each(drop);
            world.restore_snapshot();
        })
    });

    group.finish();
}

fn simulation(c: &mut Criterion) {
    let mut group = c.benchmark_group("simulation");

    let mut world = WorldState::new(TEST_ASSETS.uber_states.clone(), &mut []);

    let uber_identifier = UberIdentifier::GORLEK_ORE;

    group.bench_function("gorlek_ore", |b| b.iter(|| world.gorlek_ore()));

    group.bench_function("fetch_integer ore", |b| {
        b.iter(|| world.fetch_integer(uber_identifier))
    });

    group.bench_function("fetch ore", |b| b.iter(|| world.fetch(uber_identifier)));

    group.bench_function("simulate fetch ore", |b| {
        b.iter(|| world.simulate(&CommandInteger::FetchInteger { uber_identifier }, &[]))
    });

    group.bench_function("add_gorlek_ore", |b| {
        b.iter(|| world.add_gorlek_ore(1, &[]))
    });

    group.bench_function("add_integer ore", |b| {
        b.iter(|| world.add_integer(uber_identifier, 1, &[]))
    });

    group.bench_function("simulate ore", |b| {
        b.iter(|| world.simulate(&compile::gorlek_ore(), &[]))
    });

    // TODO can't bench this because lookup simulation is not implemented
    // let mut compiler = Compiler::new(
    //     &mut Pcg64Mcg::new(0),
    //     &*TEST_ASSETS,
    //     TEST_ASSETS.values.uber_state_data(),
    //     FxHashMap::default(),
    //     false,
    // );
    // compiler.compile_snippet("launch_fragments").unwrap();
    // let output = compiler.finish().output;
    // let launch_fragment = output
    //     .item_pool_changes
    //     .keys()
    //     .find(|item| matches!(item, CommandVoid::Lookup { .. }))
    //     .unwrap();

    // dbg!(launch_fragment);

    // group.bench_function("simulate launch_fragment", |b| {
    //     b.iter(|| world.simulate(launch_fragment, &output.events))
    // });

    group.finish();
}

fn generation(c: &mut Criterion) {
    let mut group = c.benchmark_group("generation");
    group.measurement_time(Duration::from_secs(10));

    let mut universe_settings = UniverseSettings::new(String::default());
    let mut seed = 0..;
    let graph = &TEST_ASSETS.graphs.moki;
    let test_assets = &*TEST_ASSETS;
    let loc_data = test_assets.values.loc_data();
    let uber_state_data = test_assets.values.uber_state_data();

    group.bench_function("default", |b| {
        b.iter(|| {
            universe_settings.seed = seed.next().unwrap().to_string();
            wotw_seedgen::generate_seed(
                graph,
                loc_data,
                uber_state_data,
                test_assets,
                &universe_settings,
                false,
            )
            .unwrap()
        })
    });

    TEST_ASSETS
        .world_preset("rspawn")
        .unwrap()
        .apply(&mut universe_settings.world_settings[0], &*TEST_ASSETS)
        .unwrap();
    universe_settings.world_settings[0]
        .snippets
        .push("trees".to_owned());

    for (identifier, preset) in TEST_ASSETS.world_base_presets() {
        let mut universe_settings = universe_settings.clone();
        preset
            .apply(&mut universe_settings.world_settings[0], &*TEST_ASSETS)
            .unwrap();
        let graph = TEST_ASSETS.graph(&universe_settings.world_settings);

        seed = 0..;

        group.bench_function(format!("{identifier} rspawn trees"), |b| {
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
    }

    let preset = WorldPreset {
        assets_version: 1,
        info: None,
        settings: WorldPresetSettings {
            includes: Some(FxHashSet::from_iter(["gorlek".to_owned()])),
            difficulty: Some(Difficulty::Unsafe),
            spawn: Some(Spawn::FullyRandom),
            ..Default::default()
        },
    };
    preset
        .apply(&mut universe_settings.world_settings[0], &*TEST_ASSETS)
        .unwrap();
    let graph = TEST_ASSETS.graph(&universe_settings.world_settings);

    seed = 0..;

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
    let graph = TEST_ASSETS.graph(&universe_settings.world_settings);

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

fn world<'graph, 'settings>(
    graph: &'graph Graph,
    settings: &'settings WorldSettings,
    spawn: &str,
) -> World<'graph, 'settings> {
    let spawn = graph.find_node(spawn).unwrap();
    World::new(
        &*graph,
        spawn,
        settings,
        TEST_ASSETS.uber_states.clone(),
        &mut [],
    )
}

fn spawnless_world<'graph, 'settings>(
    graph: &'graph Graph,
    settings: &'settings WorldSettings,
) -> World<'graph, 'settings> {
    World::new(
        &*graph,
        0,
        settings,
        TEST_ASSETS.uber_states.clone(),
        &mut [],
    )
}

criterion_group!(
    all,
    logic_assets,
    snippets,
    is_met,
    solutions,
    reach_check,
    simulation,
    generation,
    multiworld
);
criterion_main!(all);
