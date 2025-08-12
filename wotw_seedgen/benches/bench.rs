use std::time::Duration;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use lazy_static::lazy_static;
use rand_pcg::Pcg64Mcg;
use smallvec::smallvec;
use wotw_seedgen::{item_pool::ItemPool, UberStates, World};
use wotw_seedgen_assets::{PresetAccess, WorldPreset, WorldPresetSettings};
use wotw_seedgen_data::Skill;
use wotw_seedgen_logic_language::{
    ast::{parse, Areas},
    output::{Enemy, Graph, Requirement},
};
use wotw_seedgen_settings::{Difficulty, Spawn, UniverseSettings, WorldSettings, DEFAULT_SPAWN};
use wotw_seedgen_static_assets::{
    LOC_DATA, PRESET_ACCESS, SNIPPET_ACCESS, STATE_DATA, UBER_STATE_DATA,
};

lazy_static! {
    static ref AREAS: Areas<'static> = parse(include_str!("../areas.wotw")).into_result().unwrap();
}

fn logic_parsing(c: &mut Criterion) {
    let mut group = c.benchmark_group("parse");

    group.bench_function("areas", |b| {
        b.iter(|| parse::<Areas>(include_str!("../areas.wotw")))
    });

    let areas = &*AREAS;
    let loc_data = &*LOC_DATA;
    let state_data = &*STATE_DATA;
    group.bench_function("compile", |b| {
        b.iter(|| Graph::compile(areas.clone(), loc_data.clone(), state_data.clone(), &[]))
    });

    group.finish();
}

fn requirements(c: &mut Criterion) {
    let mut group = c.benchmark_group("requirements");

    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    let graph = compile_graph(&[]);
    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    let uber_states = UberStates::new(&UBER_STATE_DATA);
    let mut world = World::new(&graph, spawn, &world_settings, uber_states);

    let req_a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let req_b = Requirement::Damage(20.0);
    let req_c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let req_d = Requirement::Damage(10.0);
    world.set_skill(Skill::Blaze, true, &[]);
    world.modify_max_health(20, &[]);
    world.modify_max_energy((2.).into(), &[]);
    let requirement = Requirement::And(vec![
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
    ]);
    group.bench_function("nesting", |b| {
        b.iter(|| world.is_met(&requirement, &mut smallvec![world.max_orbs()]))
    });

    world.set_skill(Skill::Bow, true, &[]);
    world.modify_max_energy((10.).into(), &[]);
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

fn reach_checking(c: &mut Criterion) {
    let mut group = c.benchmark_group("reach_check");

    let graph = compile_graph(&[]);
    let uber_states = UberStates::new(&UBER_STATE_DATA);
    let world_settings = WorldSettings::default();
    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    let world = World::new(&graph, spawn, &world_settings, uber_states.clone());

    group.bench_function("short", |b| {
        b.iter(|| {
            let mut world = world.clone();
            world.traverse_spawn(&[]);
            world.set_spirit_light(10000, &[]);
            world.set_max_health(200, &[]);
            world.set_max_energy(20.0.into(), &[]);
            world.set_keystones(34, &[]);
            world.set_gorlek_ore(40, &[]);
            world.set_shard_slots(8, &[]);
            world.set_skill(Skill::Sword, true, &[]);
            world.set_skill(Skill::DoubleJump, true, &[]);
            world.set_skill(Skill::Dash, true, &[]);
            world.reached_nodes().for_each(drop);
        })
    });

    let world_settings = WorldSettings::default();
    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    let uber_states = UberStates::new(&UBER_STATE_DATA);
    let world = World::new(&graph, spawn, &world_settings, uber_states);
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
    let snippet_access = &*SNIPPET_ACCESS;
    let uber_state_data = &*UBER_STATE_DATA;

    group.bench_function("default", |b| {
        b.iter(|| {
            universe_settings.seed = seed.next().unwrap().to_string();
            wotw_seedgen::generate_seed(
                &graph,
                uber_state_data,
                snippet_access,
                &universe_settings,
                false,
            )
            .unwrap()
        })
    });

    seed = 0..;
    let preset = PRESET_ACCESS.world_preset("moki").unwrap();
    preset
        .apply(&mut universe_settings.world_settings[0], &*PRESET_ACCESS)
        .unwrap();

    group.bench_function("moki", |b| {
        b.iter(|| {
            universe_settings.seed = seed.next().unwrap().to_string();
            wotw_seedgen::generate_seed(
                &graph,
                uber_state_data,
                snippet_access,
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
        .apply(&mut universe_settings.world_settings[0], &*PRESET_ACCESS)
        .unwrap();
    let graph = compile_graph(&universe_settings.world_settings);
    group.bench_function("unsafe", |b| {
        b.iter(|| {
            universe_settings.seed = seed.next().unwrap().to_string();
            wotw_seedgen::generate_seed(
                &graph,
                uber_state_data,
                snippet_access,
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
    let preset = PRESET_ACCESS.world_preset("gorlek").unwrap();
    preset
        .apply(&mut universe_settings.world_settings[0], &*PRESET_ACCESS)
        .unwrap();
    let graph = compile_graph(&universe_settings.world_settings);

    let snippet_access = &*SNIPPET_ACCESS;
    let uber_state_data = &*UBER_STATE_DATA;

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
                    uber_state_data,
                    snippet_access,
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
        LOC_DATA.clone(),
        STATE_DATA.clone(),
        settings,
    )
    .into_result()
    .unwrap()
}

criterion_group!(
    all,
    logic_parsing,
    requirements,
    reach_checking,
    generation,
    multiworld
);
criterion_main!(all);
