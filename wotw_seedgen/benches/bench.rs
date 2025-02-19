use std::fs;

use criterion::{criterion_group, criterion_main, Criterion};
use rand::prelude::StdRng;
use rand_seeder::Seeder;
use rustc_hash::FxHashSet;
use smallvec::smallvec;

use files::*;
use item::*;
use logic::*;
use settings::*;
use util::*;
use world::*;
use wotw_seedgen::*;

fn parsing(c: &mut Criterion) {
    let input = fs::read_to_string("areas.wotw").unwrap();
    let areas = logic::Areas::parse(&input).unwrap();
    c.bench_function("parse areas", |b| b.iter(|| logic::Areas::parse(&input)));

    let input = fs::read_to_string("loc_data.csv").unwrap();
    let locations = logic::parse_locations(&input).unwrap();
    c.bench_function("parse locations", |b| {
        b.iter(|| logic::parse_locations(&input))
    });
    let input = fs::read_to_string("state_data.csv").unwrap();
    let states = logic::parse_states(&input).unwrap();

    let mut universe_settings = UniverseSettings::default();
    universe_settings.world_settings[0].difficulty = Difficulty::Unsafe;

    logic::build(
        areas.clone(),
        locations.clone(),
        states.clone(),
        &universe_settings,
        false,
    )
    .unwrap();
    c.bench_function("build", |b| {
        b.iter(|| {
            logic::build(
                areas.clone(),
                locations.clone(),
                states.clone(),
                &universe_settings,
                false,
            )
        })
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
    player.inventory.grant(Item::Skill(Skill::Blaze), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 4);
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 4);
    let req = Requirement::And(vec![
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
        Requirement::Or(vec![req_a.clone(), req_d.clone()]),
        Requirement::Or(vec![req_b.clone(), req_c.clone()]),
    ]);
    c.bench_function("nested ands and ors", |b| {
        b.iter(|| req.is_met(&player, &states, smallvec![player.max_orbs()]))
    });

    player.inventory.grant(Item::Skill(Skill::Bow), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 20);
    let req = Requirement::Combat(smallvec![(Enemy::Lizard, 3),]);
    c.bench_function("short combat", |b| {
        b.iter(|| req.is_met(&player, &states, smallvec![player.max_orbs()]))
    });
    let req = Requirement::And(vec![
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
    player.inventory.items.clear();
    c.bench_function("long combat progression", |b| {
        b.iter(|| req.solutions(&player, &states, smallvec![player.max_orbs()], 1000, 1000))
    });
}

fn reach_checking(c: &mut Criterion) {
    let areas = fs::read_to_string("areas.wotw").unwrap();
    let locations = fs::read_to_string("loc_data.csv").unwrap();
    let states = fs::read_to_string("state_data.csv").unwrap();
    let graph = parse_logic(
        &areas,
        &locations,
        &states,
        &UniverseSettings::default(),
        false,
    )
    .unwrap();

    c.bench_function("short reach check", |b| {
        b.iter(|| {
            let world_settings = WorldSettings::default();
            let mut player = Player::new(&world_settings);
            player
                .inventory
                .grant(Item::Resource(Resource::HealthFragment), 40);
            player
                .inventory
                .grant(Item::Resource(Resource::EnergyFragment), 40);
            player
                .inventory
                .grant(Item::Resource(Resource::Keystone), 34);
            player
                .inventory
                .grant(Item::Resource(Resource::GorlekOre), 40);
            player.inventory.grant(Item::SpiritLight(1), 10000);
            player
                .inventory
                .grant(Item::Resource(Resource::ShardSlot), 8);
            player.inventory.grant(Item::Skill(Skill::Sword), 1);
            player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
            player.inventory.grant(Item::Skill(Skill::Dash), 1);
            let world = World::new_spawn(&graph, &world_settings);
            let spawn = world.graph.find_spawn("MarshSpawn.Main").unwrap();
            world
                .graph
                .reached_locations(&world.player, spawn, world.uber_states(), &world.sets);
        })
    });
    c.bench_function("long reach check", |b| {
        b.iter(|| {
            let world_settings = WorldSettings::default();
            let mut world = World::new_spawn(&graph, &world_settings);
            world.player.inventory = Pool::preset().inventory;
            world.player.inventory.grant(Item::SpiritLight(1), 10000);
            let spawn = world.graph.find_spawn("MarshSpawn.Main").unwrap();
            world
                .graph
                .reached_locations(&world.player, spawn, world.uber_states(), &world.sets);
        })
    });
}

fn doors(c: &mut Criterion) {
    let universe_settings = UniverseSettings::default();

    let areas = fs::read_to_string("areas.wotw").unwrap();
    let locations = fs::read_to_string("loc_data.csv").unwrap();
    let states = fs::read_to_string("state_data.csv").unwrap();
    let graph = parse_logic(&areas, &locations, &states, &universe_settings, false).unwrap();

    c.bench_function("door headers", |b| {
        b.iter(|| {
            let world_settings = WorldSettings::default();
            let mut world = World::new_spawn(&graph, &world_settings);

            let mut rng: StdRng = Seeder::from(&"Test").make_rng();

            let _ = generator::doors::generate_door_headers(&mut world, &mut rng);
        })
    });
}

fn generation(c: &mut Criterion) {
    let mut universe_settings = UniverseSettings::default();
    let mut seed = 0;

    let areas = fs::read_to_string("areas.wotw").unwrap();
    let locations = fs::read_to_string("loc_data.csv").unwrap();
    let states = fs::read_to_string("state_data.csv").unwrap();
    let graph = parse_logic(&areas, &locations, &states, &universe_settings, false).unwrap();

    c.bench_function("singleplayer", |b| {
        b.iter(|| {
            universe_settings.seed = seed.to_string();
            seed += 1;
            wotw_seedgen::generate_seed(&graph, &NO_FILE_ACCESS, &universe_settings).unwrap();
        })
    });

    seed = 0;
    universe_settings.world_settings[0].difficulty = Difficulty::Unsafe;
    universe_settings.world_settings[0].spawn = Spawn::FullyRandom;
    let graph = parse_logic(&areas, &locations, &states, &universe_settings, false).unwrap();
    Criterion::default()
        .sample_size(10)
        .bench_function("unsafe", |b| {
            b.iter(|| {
                universe_settings.seed = seed.to_string();
                seed += 1;
                wotw_seedgen::generate_seed(&graph, &NO_FILE_ACCESS, &universe_settings).unwrap();
            })
        });

    seed = 0;
    universe_settings = UniverseSettings::default();
    universe_settings.world_settings.extend_from_within(..);
    let graph = parse_logic(&areas, &locations, &states, &universe_settings, false).unwrap();

    c.bench_function("two worlds", |b| {
        b.iter(|| {
            universe_settings.seed = seed.to_string();
            seed += 1;
            wotw_seedgen::generate_seed(&graph, &NO_FILE_ACCESS, &universe_settings).unwrap();
        })
    });

    let seed = wotw_seedgen::generate_seed(&graph, &NO_FILE_ACCESS, &universe_settings).unwrap();

    c.bench_function("convert seed to text", |b| b.iter(|| seed.seed_files()));
}

criterion_group!(all, parsing, requirements, reach_checking, doors, generation);
criterion_group!(only_parsing, parsing);
criterion_group!(only_requirements, requirements);
criterion_group!(only_reach_checking, reach_checking);
criterion_group!(only_doors, doors);
criterion_group!(only_generation, generation);
criterion_main!(only_doors); // put any of the group names in here
