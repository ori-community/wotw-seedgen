use criterion::{criterion_group, criterion_main, Criterion};
use lazy_static::lazy_static;
use rustc_hash::FxHashSet;
use smallvec::smallvec;
use std::io;
use wotw_seedgen::{item_pool::ItemPool, Player, UberStates, World};
use wotw_seedgen_assets::{LocData, StateData};
use wotw_seedgen_data::Skill;
use wotw_seedgen_logic_language::{
    ast::{parse, Areas},
    output::{Enemy, Graph, Requirement},
};
use wotw_seedgen_seed_language::output::CompilerOutput;
use wotw_seedgen_settings::{
    Difficulty, PresetAccess, Spawn, UniverseSettings, WorldPreset, WorldSettings, DEFAULT_SPAWN,
};
use wotw_seedgen_static_assets::{
    LOC_DATA, PRESET_ACCESS, SNIPPET_ACCESS, STATE_DATA, UBER_STATE_DATA,
};

lazy_static! {
    pub static ref AREAS: Areas<'static> =
        parse(include_str!("../areas.wotw")).into_result().unwrap();
}

fn logic_parsing(c: &mut Criterion) {
    c.bench_function("parse areas", |b| {
        b.iter(|| parse::<Areas>(include_str!("../areas.wotw")))
    });
    c.bench_function("parse locations and states", |b| {
        b.iter(|| {
            (
                LocData::from_reader(include_bytes!("../../assets/loc_data.csv").as_slice()),
                StateData::from_reader(include_bytes!("../../assets/state_data.csv").as_slice()),
            )
        })
    });
    let areas = &*AREAS;
    let loc_data = &*LOC_DATA;
    let state_data = &*STATE_DATA;
    c.bench_function("compile", |b| {
        b.iter(|| Graph::compile(areas.clone(), loc_data.clone(), state_data.clone(), &[]))
    });
}

fn requirements(c: &mut Criterion) {
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    let mut player = Player::new(&world_settings);
    let states = FxHashSet::default();

    let req_a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let req_b = Requirement::Damage(20.0);
    let req_c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let req_d = Requirement::Damage(10.0);
    player.inventory.skills.insert(Skill::Blaze);
    player.inventory.health += 20;
    player.inventory.energy += 2.;
    let requirement = Requirement::And(vec![
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
    ]);
    c.bench_function("nested ands and ors", |b| {
        b.iter(|| player.is_met(&requirement, &states, smallvec![player.max_orbs()]))
    });

    player.inventory.skills.insert(Skill::Bow);
    player.inventory.energy += 10.;
    let requirement = Requirement::Combat(smallvec![(Enemy::Lizard, 3),]);
    c.bench_function("short combat", |b| {
        b.iter(|| player.is_met(&requirement, &states, smallvec![player.max_orbs()]))
    });
    let requirement = Requirement::And(vec![
        Requirement::Combat(smallvec![
            (Enemy::Mantis, 2),
            (Enemy::Lizard, 2),
            (Enemy::EnergyRefill, 4),
            (Enemy::SneezeSlug, 2),
            (Enemy::Mantis, 1),
            (Enemy::Skeeto, 1),
            (Enemy::EnergyRefill, 4),
            (Enemy::SmallSkeeto, 7),
            (Enemy::Skeeto, 2),
            (Enemy::EnergyRefill, 4),
            (Enemy::Lizard, 2),
            (Enemy::Mantis, 2),
        ]),
        Requirement::Damage(50.0),
    ]);
    player.inventory.clear();
    c.bench_function("long combat progression", |b| {
        b.iter(|| {
            player.solutions(
                &requirement,
                &states,
                smallvec![player.max_orbs()],
                1000,
                1000,
            )
        })
    });
}

fn reach_checking(c: &mut Criterion) {
    let graph = Graph::compile(AREAS.clone(), LOC_DATA.clone(), STATE_DATA.clone(), &[])
        .into_result()
        .unwrap();
    let uber_states = UberStates::new(&UBER_STATE_DATA);

    c.bench_function("short reach check", |b| {
        b.iter(|| {
            let output = CompilerOutput::default();
            let world_settings = WorldSettings::default();
            let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
            let mut world = World::new(&graph, spawn, &world_settings, uber_states.clone());
            world.set_spirit_light(10000, &output);
            world.set_max_health(200, &output);
            world.set_max_energy(20.0.into(), &output);
            world.set_keystones(34, &output);
            world.set_gorlek_ore(40, &output);
            world.set_shard_slots(8, &output);
            world.set_skill(Skill::Sword, true, &output);
            world.set_skill(Skill::DoubleJump, true, &output);
            world.set_skill(Skill::Dash, true, &output);
            world.reached()
        })
    });
    let output = CompilerOutput::default();
    let world_settings = WorldSettings::default();
    let spawn = graph.find_node(DEFAULT_SPAWN).unwrap();
    let uber_states = UberStates::new(&UBER_STATE_DATA);
    let mut world = World::new_spawn(&graph, spawn, &world_settings, uber_states);
    let mut pool = ItemPool::default();
    for item in pool.drain() {
        world.simulate(&item, &output);
    }
    c.bench_function("long reach check", |b| b.iter(|| world.reached()));
}

fn generation(c: &mut Criterion) {
    let mut universe_settings = UniverseSettings::new(String::default());
    universe_settings.world_settings[0]
        .apply_world_preset(PRESET_ACCESS.world_preset("moki").unwrap(), &*PRESET_ACCESS)
        .unwrap();
    let mut seed = 0;
    let graph = Graph::compile(
        AREAS.clone(),
        LOC_DATA.clone(),
        STATE_DATA.clone(),
        &universe_settings.world_settings,
    )
    .into_result()
    .unwrap();

    let snippet_access = &*SNIPPET_ACCESS;
    let uber_state_data = &*UBER_STATE_DATA;
    c.bench_function("moki", |b| {
        b.iter(|| {
            universe_settings.seed = seed.to_string();
            seed += 1;
            wotw_seedgen::generate_seed(
                &graph,
                snippet_access,
                uber_state_data,
                &mut io::stderr(),
                &universe_settings,
            )
            .unwrap()
        })
    });

    seed = 0;
    let mut universe_settings = UniverseSettings::new(String::default());
    universe_settings.world_settings[0]
        .apply_world_preset(
            WorldPreset {
                includes: Some(
                    ["gorlek".to_string(), "rspawn".to_string()]
                        .into_iter()
                        .collect(),
                ),
                difficulty: Some(Difficulty::Unsafe),
                spawn: Some(Spawn::FullyRandom),
                ..Default::default()
            },
            &*PRESET_ACCESS,
        )
        .unwrap();
    let graph = Graph::compile(
        AREAS.clone(),
        LOC_DATA.clone(),
        STATE_DATA.clone(),
        &universe_settings.world_settings,
    )
    .into_result()
    .unwrap();
    Criterion::default()
        .sample_size(10)
        .bench_function("unsafe", |b| {
            b.iter(|| {
                universe_settings.seed = seed.to_string();
                seed += 1;
                wotw_seedgen::generate_seed(
                    &graph,
                    snippet_access,
                    uber_state_data,
                    &mut io::stderr(),
                    &universe_settings,
                )
                .unwrap()
            })
        });

    seed = 0;
    universe_settings = UniverseSettings::new(String::default());
    universe_settings.world_settings.extend_from_within(..);
    let graph = Graph::compile(
        AREAS.clone(),
        LOC_DATA.clone(),
        STATE_DATA.clone(),
        &universe_settings.world_settings,
    )
    .into_result()
    .unwrap();

    c.bench_function("two worlds", |b| {
        b.iter(|| {
            universe_settings.seed = seed.to_string();
            seed += 1;
            wotw_seedgen::generate_seed(
                &graph,
                snippet_access,
                uber_state_data,
                &mut io::stderr(),
                &universe_settings,
            )
            .unwrap()
        })
    });
}

criterion_group!(all, logic_parsing, requirements, reach_checking, generation);
criterion_group!(only_parsing, logic_parsing);
criterion_group!(only_requirements, requirements);
criterion_group!(only_reach_checking, reach_checking);
criterion_group!(only_generation, generation);
criterion_main!(only_reach_checking); // put any of the group names in here
