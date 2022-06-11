use std::fs;

use criterion::{criterion_group, criterion_main, Criterion};

use rustc_hash::FxHashSet;
use smallvec::smallvec;

use wotw_seedgen::*;
use logic::*;
use world::*;
use item::*;
use util::*;
use settings::*;

fn parsing(c: &mut Criterion) {
    let input = read_file("areas.wotw", "logic").unwrap();
    let areas = logic::Areas::parse(&input).unwrap();
    c.bench_function("parse areas", |b| b.iter(|| logic::Areas::parse(&input)));

    let input = read_file("loc_data.csv", "logic").unwrap();
    let locations = logic::parse_locations(&input).unwrap();
    c.bench_function("parse locations", |b| b.iter(|| logic::parse_locations(&input)));
    let input = read_file("state_data.csv", "logic").unwrap();
    let states = logic::parse_states(&input).unwrap();

    let mut settings = Settings::default();
    settings.world_settings[0].difficulty = Difficulty::Unsafe;

    logic::build(areas.clone(), locations.clone(), states.clone(), &settings, false).unwrap();
    c.bench_function("build", |b| b.iter(|| logic::build(areas.clone(), locations.clone(), states.clone(), &settings, false)));
}

fn requirements(c: &mut Criterion) {
    let world_settings = WorldSettings { difficulty: Difficulty::Unsafe, ..WorldSettings::default() };
    let mut player = Player::new(&world_settings);
    let states = FxHashSet::default();

    let req_a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let req_b = Requirement::Damage(20.0);
    let req_c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let req_d = Requirement::Damage(10.0);
    player.inventory.grant(Item::Skill(Skill::Blaze), 1);
    player.inventory.grant(Item::Resource(Resource::Energy), 4);
    player.inventory.grant(Item::Resource(Resource::Health), 4);
    let req = Requirement::And(vec![Requirement::Or(vec![req_a.clone(), req_d.clone()]), Requirement::Or(vec![req_b.clone(), req_c.clone()]), Requirement::Or(vec![req_a.clone(), req_d.clone()]), Requirement::Or(vec![req_b.clone(), req_c.clone()])]);
    c.bench_function("nested ands and ors", |b| b.iter(|| req.is_met(&player, &states, player.max_orbs())));

    let world_settings = WorldSettings::default();
    player = Player::new(&world_settings);
    player.inventory.grant(Item::Skill(Skill::Bow), 1);
    player.inventory.grant(Item::Resource(Resource::Energy), 40);
    let req = Requirement::Combat(smallvec![
        (Enemy::Lizard, 3),
    ]);
    c.bench_function("short combat", |b| b.iter(|| req.is_met(&player, &states, player.max_orbs())));
    let req = Requirement::Combat(smallvec![
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
    ]);
    let states = Vec::default();
    c.bench_function("long combat progression", |b| b.iter(|| req.items_needed(&player, &states)));
}

fn reach_checking(c: &mut Criterion) {
    let areas = fs::read_to_string("areas.wotw").unwrap();
    let locations = fs::read_to_string("loc_data.csv").unwrap();
    let states = fs::read_to_string("state_data.csv").unwrap();
    let graph = parse_logic(&areas, &locations, &states, &Settings::default(), false).unwrap();

    c.bench_function("short reach check", |b| b.iter(|| {
        let world_settings = WorldSettings::default();
        let mut player = Player::new(&world_settings);
        player.inventory.grant(Item::Resource(Resource::Health), 40);
        player.inventory.grant(Item::Resource(Resource::Energy), 40);
        player.inventory.grant(Item::Resource(Resource::Keystone), 34);
        player.inventory.grant(Item::Resource(Resource::Ore), 40);
        player.inventory.grant(Item::SpiritLight(1), 10000);
        player.inventory.grant(Item::Resource(Resource::ShardSlot), 8);
        player.inventory.grant(Item::Skill(Skill::Sword), 1);
        player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
        player.inventory.grant(Item::Skill(Skill::Dash), 1);
        let world = World::new(&graph, &world_settings);
        let spawn = world.graph.find_spawn("MarshSpawn.Main").unwrap();
        world.graph.reached_locations(&world.player, spawn, &world.uber_states, &world.sets).unwrap();
    }));
    c.bench_function("long reach check", |b| b.iter(|| {
        let world_settings = WorldSettings::default();
        let mut world = World::new(&graph, &world_settings);
        world.player.inventory = Pool::preset().inventory;
        world.player.inventory.grant(Item::SpiritLight(1), 10000);
        let spawn = world.graph.find_spawn("MarshSpawn.Main").unwrap();
        world.graph.reached_locations(&world.player, spawn, &world.uber_states, &world.sets).unwrap();
    }));
}

fn generation(c: &mut Criterion) {
    let mut settings = Settings::default();

    let areas = fs::read_to_string("areas.wotw").unwrap();
    let locations = fs::read_to_string("loc_data.csv").unwrap();
    let states = fs::read_to_string("state_data.csv").unwrap();

    c.bench_function("singleplayer", |b| b.iter(|| {
        let graph = parse_logic(&areas, &locations, &states, &Settings::default(), false).unwrap();
        wotw_seedgen::generate_seed(&graph, settings.clone()).unwrap();
    }));

    settings.world_settings.extend_from_within(..);

    c.bench_function("two worlds", |b| b.iter(|| {
        let graph = parse_logic(&areas, &locations, &states, &Settings::default(), false).unwrap();
        wotw_seedgen::generate_seed(&graph, settings.clone()).unwrap();
    }));
}

criterion_group!(all, parsing, requirements, reach_checking, generation);
criterion_group!(only_parsing, parsing);
criterion_group!(only_requirements, requirements);
criterion_group!(only_reach_checking, reach_checking);
criterion_group!(only_generation, generation);
criterion_main!(only_generation);  // put any of the group names in here
