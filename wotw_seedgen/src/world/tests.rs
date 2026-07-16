use std::ops::ControlFlow;

use crate::{
    item_pool::ItemPoolBuilder,
    orbs::{OrbVariants, Orbs},
    World,
};
use itertools::Itertools;
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashSet;
use smallvec::smallvec;
use wotw_seedgen_data::{
    assets::{AssetCacheValues, TEST_ASSETS},
    logic_language::output::{Enemy, Graph, RefillValue, Requirement},
    seed_language::{
        output::CommandsOutput,
        simulate::{Simulation, Snapshot},
    },
    test_logger, Difficulty, Shard, Skill, WorldSettings, DEFAULT_SPAWN,
};

pub fn empty_test_world<'graph, 'settings>(
    graph: &'graph Graph,
    settings: &'settings WorldSettings,
    spawn: &str,
) -> World<'graph, 'settings, 'graph> {
    let mut world = test_world(graph, settings, spawn);

    world.store_base_max_health(0, &CommandsOutput::NONE);
    world.store_base_max_energy(0., &CommandsOutput::NONE);
    world.store_shard_slots(0, &CommandsOutput::NONE);

    world
}

pub fn test_world<'graph, 'settings>(
    graph: &'graph Graph,
    settings: &'settings WorldSettings,
    spawn: &str,
) -> World<'graph, 'settings, 'graph> {
    let spawn = graph.find_node(spawn).unwrap();
    World::new(
        &*graph,
        spawn,
        settings,
        TEST_ASSETS.uber_states.clone(),
        &mut [],
        None,
    )
}

#[test]
fn full_reach_check() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&TEST_ASSETS.graphs.gorlek, &settings, DEFAULT_SPAWN);

    let mut pool = ItemPoolBuilder::new(&mut Pcg64Mcg::new(0)).finish();
    for item in pool.take() {
        world.simulate(&item, &CommandsOutput::NONE);
    }
    world.add_spirit_light(10000, &CommandsOutput::NONE);

    world.traverse_spawn(&CommandsOutput::NONE);

    let reached = world
        .reached_pickups()
        .map(|pickup| pickup.identifier.as_str())
        .collect();

    let all_locations = TEST_ASSETS
        .values
        .loc_data()
        .entries
        .iter()
        .map(|location| location.identifier.as_str())
        .collect::<FxHashSet<_>>();

    if !(reached == all_locations) {
        eprintln!("remaining fails:\n{}", world.fails().display(world.graph));

        let mut diff = all_locations.difference(&reached).collect::<Vec<_>>();
        diff.sort_unstable();
        eprintln!(
            "difference (reached {reached_len} / {total_len} items): {diff}",
            reached_len = reached.len(),
            total_len = all_locations.len(),
            diff = diff.iter().format(", ")
        );
    }

    assert_eq!(reached, all_locations);
}

#[test]
fn small_reach_check() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(
        &TEST_ASSETS.graphs.gorlek,
        &settings,
        "GladesTown.Teleporter",
    );

    world.store_skill(Skill::DoubleJump, true, &CommandsOutput::NONE);
    world.store_shard(Shard::TripleJump, true, &CommandsOutput::NONE);
    world.store_base_max_health(5, &CommandsOutput::NONE);

    world.traverse_spawn(&CommandsOutput::NONE);

    let reached = world
        .reached_pickups()
        .map(|pickup| pickup.identifier.as_str())
        .collect::<FxHashSet<_>>();
    assert_eq!(
        reached,
        FxHashSet::from_iter([
            "GladesTown.UpdraftCeilingSL",
            "GladesTown.AboveTPSL",
            "GladesTown.BountyShard",
            "GladesTown.BelowHoleHutSL"
        ])
    );
}

#[test]
fn max_energy() {
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    assert_eq!(world.max_energy(), 0.0);

    world.add_base_max_energy(5., &CommandsOutput::NONE);
    world.store_shard(Shard::Energy, true, &CommandsOutput::NONE);
    assert_eq!(world.max_energy(), 5.0);

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    world.settings = &settings;
    assert_eq!(world.max_energy(), 6.0);
}

#[test]
fn refill_orbs() {
    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.snapshot();

    let expected = [
        0., 5., 10., 15., 20., 25., 30., 35., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40.,
        40., 40., 40., 40., 40., 40., 40., 40., 40., 41., 42., 44., 45., 47., 48., 50., 52., 53.,
        55., 56., 58., 59., 61., 62., 64., 65., 66., 68., 69.,
    ];
    for health in expected {
        assert_eq!(world.checkpoint_orbs().health, health);
        world.add_base_max_health(5, &CommandsOutput::NONE);
    }

    world.restore_snapshot();
    world.snapshot();

    let expected = [
        0., 0., 0., 0., 1., 1., 1., 1., 1., 2., 2., 2., 2., 2., 2., 2., 3., 3., 3., 3., 3., 4., 4.,
        4., 4., 4., 4., 4., 5., 5., 5., 5., 5., 6., 6., 6., 6., 6., 6., 6., 7., 7., 7., 7., 7., 8.,
        8.,
    ];
    for drops in expected {
        assert_eq!(world.health_plant_drops(), drops);
        world.add_base_max_health(5, &CommandsOutput::NONE);
    }

    world.restore_snapshot();

    world.store_shard(Shard::Energy, true, &CommandsOutput::NONE);
    world.store_shard(Shard::Vitality, true, &CommandsOutput::NONE);
    assert_eq!(world.checkpoint_orbs(), Orbs::new(0.0, 1.0));

    world.store_base_max_health(35, &CommandsOutput::NONE);
    assert_eq!(world.checkpoint_orbs(), Orbs::new(35.0, 1.0));

    world.store_base_max_health(140, &CommandsOutput::NONE);
    assert_eq!(world.checkpoint_orbs(), Orbs::new(45.0, 1.0));

    let world = test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);

    let mut orb_variants = smallvec![Orbs::default()];
    world.refill(RefillValue::Full, &mut orb_variants);
    assert_eq!(&orb_variants[..], &[world.max_orbs()]);
}

#[test]
fn destroy_cost() {
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    assert_eq!(world.destroy_cost::<false>(10.0, false), None);

    world.store_skill(Skill::Spear, true, &CommandsOutput::NONE);
    assert_eq!(world.destroy_cost::<false>(10.0, true), Some(4.0));
    assert_eq!(world.destroy_cost::<false>(0.0, false), Some(0.0));

    world.store_skill(Skill::Bow, true, &CommandsOutput::NONE);
    assert_eq!(world.destroy_cost::<false>(10.0, false), Some(1.5));

    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    world.store_skill(Skill::GladesAncestralLight, true, &CommandsOutput::NONE);
    world.store_skill(Skill::MarshAncestralLight, true, &CommandsOutput::NONE);
    world.store_shard(Shard::Wingclip, true, &CommandsOutput::NONE);
    world.add_shard_slots(1, &CommandsOutput::NONE);
    world.store_skill(Skill::Bow, false, &CommandsOutput::NONE);
    assert_eq!(world.destroy_cost::<false>(1.0, false), Some(2.0));

    world.store_skill(Skill::Bow, true, &CommandsOutput::NONE);
    assert_eq!(world.destroy_cost::<false>(10.0, true), Some(0.25));

    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Grenade, true, &CommandsOutput::NONE);
    world.store_skill(Skill::Shuriken, true, &CommandsOutput::NONE);
    assert_eq!(world.destroy_cost::<false>(20.0, false), Some(1.5));
    assert_eq!(world.destroy_cost::<false>(24.0, false), Some(1.5));
    assert_eq!(world.destroy_cost::<false>(34.0, false), Some(2.0));
}

#[test]
fn is_met() {
    let settings = WorldSettings::default();
    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.snapshot();

    macro_rules! set_difficulty {
        ($difficulty:expr) => {
            let settings = WorldSettings::difficulty_default($difficulty);
            world.settings = &settings;
        };
    }

    macro_rules! test {
        ($req:expr, [$world_orbs:expr], "✅") => {
            test!($req, [$world_orbs], ControlFlow::is_continue);
        };
        ($req:expr, [$world_orbs:expr], "❌") => {
            test!($req, [$world_orbs], ControlFlow::is_break);
        };

        ($req:expr, [$world_orbs:expr], $f:path) => {
            {
                let req = $req;
                let mut orb_variants: OrbVariants = smallvec![$world_orbs];
                let control_flow = world.is_met(&req, &mut orb_variants);
                assert!($f(&control_flow));
            }
        };

        ($req:expr, [$world_orbs:expr], [$($orbs:expr),* $(,)?]) => {
            {
                let req = $req;
                let mut left: OrbVariants = smallvec![$world_orbs];
                let _ = world.is_met(&req, &mut left);
                left.sort_unstable_by(|a, b| a.health.total_cmp(&b.health));
                let mut right: OrbVariants = smallvec![$($world_orbs + $orbs),*];
                right.sort_unstable_by(|a, b| a.health.total_cmp(&b.health));
                assert_eq!(left, right);
            }
        };

        ($req:expr, $symbol:tt) => {
            test!($req, [world.max_orbs()], $symbol);
        };
        ($req:expr, [$($orbs:tt)*]) => {
            test!($req, [world.max_orbs()], [$($orbs)*]);
        };
    }

    let req = Requirement::Skill(Skill::Blaze);
    eprintln!("testing {req}");

    test!(&req, "❌");
    world.store_skill(Skill::Blaze, true, &CommandsOutput::NONE);
    test!(&req, "✅");

    test!(Requirement::And(vec![req.clone(), Requirement::Free]), "✅");
    test!(Requirement::Or(vec![Requirement::Impossible, req]), "✅");

    let req = Requirement::EnergySkill(Skill::Blaze, 1.0);
    eprintln!("testing {req}");

    test!(&req, "❌");
    world.store_base_max_energy(1., &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_base_max_energy(2., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -2.0)]);

    set_difficulty!(Difficulty::Gorlek);
    world.store_base_max_energy(1., &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_shard(Shard::Energy, true, &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -2.0)]);
    world.store_shard(Shard::Energy, false, &CommandsOutput::NONE);

    set_difficulty!(Difficulty::Unsafe);
    test!(&req, [Orbs::new(0., -1.0)]);

    world.store_shard(Shard::Overcharge, true, &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -1.0 * 0.5)]);
    world.store_shard(Shard::Overcharge, false, &CommandsOutput::NONE);

    world.store_shard(Shard::LifePact, true, &CommandsOutput::NONE);
    world.store_base_max_energy(0.5, &CommandsOutput::NONE);
    world.store_base_max_health(15, &CommandsOutput::NONE);
    test!(
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs::new(-5.0, -0.5)]
    );
    test!(
        Requirement::NonConsumingEnergySkill(Skill::Blaze),
        [Orbs::new(-5.0, 0.)]
    );
    test!(
        Requirement::NonConsumingEnergySkill(Skill::Blaze),
        [Orbs::new(world.max_health(), 0.0)],
        [Orbs::new(-10.0, 0.5)]
    );

    set_difficulty!(Difficulty::Moki);
    world.restore_snapshot();
    world.snapshot();

    eprintln!("testing Damage");

    world.store_base_max_health(30, &CommandsOutput::NONE);
    test!(Requirement::Damage(30.0), "❌");
    world.store_base_max_health(35, &CommandsOutput::NONE);
    test!(Requirement::Damage(30.0), [Orbs::new(-30.0, 0.)]);

    set_difficulty!(Difficulty::Gorlek);
    world.store_base_max_health(30, &CommandsOutput::NONE);
    world.store_shard(Shard::Vitality, true, &CommandsOutput::NONE);
    test!(Requirement::Damage(30.0), [Orbs::new(-30.0, 0.)]);
    world.store_shard(Shard::Vitality, false, &CommandsOutput::NONE);
    world.store_shard(Shard::Resilience, true, &CommandsOutput::NONE);
    test!(Requirement::Damage(30.0), [Orbs::new(-30.0 * 0.9, 0.)]);
    world.store_shard(Shard::Resilience, false, &CommandsOutput::NONE);

    set_difficulty!(Difficulty::Unsafe);
    world.store_base_max_energy(3., &CommandsOutput::NONE);
    world.store_skill(Skill::Regenerate, true, &CommandsOutput::NONE);
    test!(Requirement::Damage(60.0), "❌");
    world.store_base_max_health(65, &CommandsOutput::NONE);
    test!(
        Requirement::Damage(60.0),
        [Orbs::new(30.0, world.max_energy())],
        [Orbs::new(-25.0, -2.0)]
    );
    test!(
        Requirement::Danger(30.0),
        [Orbs::new(30.0, world.max_energy())],
        [Orbs::new(30.0, -1.0)]
    );
    test!(
        Requirement::Danger(60.0),
        [Orbs::new(30.0, world.max_energy())],
        [Orbs::new(35.0, -2.0)]
    );

    set_difficulty!(Difficulty::Moki);
    world.restore_snapshot();
    world.snapshot();

    let req = Requirement::BreakWall(12.0);
    eprintln!("testing {req}");

    test!(&req, "❌");
    world.store_skill(Skill::Sword, true, &CommandsOutput::NONE);
    test!(&req, [world.max_orbs()]);

    world.store_skill(Skill::Sword, false, &CommandsOutput::NONE);

    world.store_skill(Skill::Grenade, true, &CommandsOutput::NONE);
    world.store_base_max_energy(1.5, &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_base_max_energy(2., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -2.0)]);

    set_difficulty!(Difficulty::Unsafe);
    world.store_base_max_energy(1., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -1.0)]);
    set_difficulty!(Difficulty::Moki);
    world.store_base_max_energy(1.5, &CommandsOutput::NONE);
    test!(&req, "❌");

    world.restore_snapshot();
    world.snapshot();

    let req = Requirement::ShurikenBreak(12.0);
    eprintln!("testing {req}");

    world.store_skill(Skill::Shuriken, true, &CommandsOutput::NONE);
    world.store_base_max_energy(5., &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_base_max_energy(6., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -6.0)]);
    set_difficulty!(Difficulty::Unsafe);
    world.store_base_max_energy(2., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -2.0)]);

    world.restore_snapshot();
    world.snapshot();

    // Slug has 13, Skeeto has 20 health
    let req = Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]);
    eprintln!("testing {req}");

    // Bow has 4 damage -> 2 * 4 + 5 = 13 shots * 0.25 energy / shot = 3.25 energy
    world.store_skill(Skill::Bow, true, &CommandsOutput::NONE);
    world.store_base_max_energy(3., &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_base_max_energy(3.25, &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -3.25)]);
    // With 5 damage -> 2 * 3 + 4 = 10 shots * 0.25 energy / shot = 2.5 energy
    world.store_skill(Skill::MarshAncestralLight, true, &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -2.5)]);
    world.store_shard_slots(3, &CommandsOutput::NONE);
    // Wingclip stacks additively, increasing the damage to 4 * 2.25 = 9 against Skeeto
    world.store_shard(Shard::Wingclip, true, &CommandsOutput::NONE);
    // Splinter has 3 shots of half strength -> 7.5 damage against Slug and 13.5 against Skeeto
    world.store_shard(Shard::Splinter, true, &CommandsOutput::NONE);
    // 2 * 2 + 2 = 6 shots * 0.25 energy / shot = 1.5 energy
    test!(&req, [Orbs::new(0., -1.5)]);

    set_difficulty!(Difficulty::Moki);
    world.store_base_max_energy(6.5, &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_skill(Skill::DoubleJump, true, &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -6.5)]);

    set_difficulty!(Difficulty::Unsafe);
    world.restore_snapshot();
    world.snapshot();

    let req = Requirement::Combat(smallvec![
        (Enemy::Sandworm, 1),
        (Enemy::Bat, 1),
        (Enemy::EnergyRefill, 99),
        (Enemy::ShieldMiner, 2),
        (Enemy::EnergyRefill, 1),
        (Enemy::Balloon, 4)
    ]);
    eprintln!("testing {req}");

    world.store_skill(Skill::Shuriken, true, &CommandsOutput::NONE);
    world.store_skill(Skill::Spear, true, &CommandsOutput::NONE);
    world.store_base_max_energy(13.5, &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_base_max_energy(14., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -14.0)]);
    set_difficulty!(Difficulty::Moki);
    world.store_base_max_energy(32.5, &CommandsOutput::NONE);
    world.store_skill(Skill::Bash, true, &CommandsOutput::NONE);
    world.store_skill(Skill::Launch, true, &CommandsOutput::NONE);
    world.store_skill(Skill::Burrow, true, &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_base_max_energy(33., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -33.0)]);

    set_difficulty!(Difficulty::Unsafe);
    world.restore_snapshot();
    world.snapshot();

    let req = Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]);
    eprintln!("testing {req}");

    world.store_skill(Skill::Spear, true, &CommandsOutput::NONE);
    world.store_skill(Skill::DoubleJump, true, &CommandsOutput::NONE);
    world.store_base_max_energy(2., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -2.0)]);
    set_difficulty!(Difficulty::Moki);
    world.store_base_max_energy(7.5, &CommandsOutput::NONE);
    test!(&req, "❌");
    world.store_base_max_energy(8., &CommandsOutput::NONE);
    test!(&req, [Orbs::new(0., -8.0)]);

    set_difficulty!(Difficulty::Unsafe);
    world.restore_snapshot();
    world.snapshot();

    eprintln!("testing requirement chains");

    let a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let b = Requirement::Damage(20.0);
    let c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let d = Requirement::Damage(10.0);

    world.store_skill(Skill::Blaze, true, &CommandsOutput::NONE);
    world.store_base_max_energy(2., &CommandsOutput::NONE);
    world.store_base_max_health(25, &CommandsOutput::NONE);

    test!(
        Requirement::And(vec![c.clone(), d.clone()]),
        [Orbs::new(-10.0, -1.0)]
    );
    test!(
        Requirement::Or(vec![a.clone(), b.clone()]),
        [Orbs::new(0., -2.0), Orbs::new(-20.0, 0.)]
    );
    test!(
        Requirement::Or(vec![
            Requirement::And(vec![a.clone(), b.clone()]),
            Requirement::And(vec![c.clone(), d.clone()]),
            a.clone(),
            b.clone()
        ]),
        [
            Orbs::new(-10.0, -1.0),
            Orbs::new(0., -2.0),
            Orbs::new(-20.0, 0.)
        ]
    );
    test!(
        Requirement::And(vec![
            Requirement::Or(vec![a.clone(), d.clone()]),
            Requirement::Or(vec![b.clone(), c.clone()])
        ]),
        [Orbs::new(-10.0, -1.0)]
    );
    world.store_base_max_energy(6., &CommandsOutput::NONE);
    world.store_base_max_health(65, &CommandsOutput::NONE);
    test!(
        Requirement::And(vec![
            Requirement::Or(vec![a.clone(), d.clone()]),
            Requirement::Or(vec![b.clone(), c.clone()]),
            Requirement::Or(vec![a.clone(), d.clone()]),
            Requirement::Or(vec![b.clone(), c.clone()])
        ]),
        [
            Orbs::new(0., -6.0),
            Orbs::new(-10.0, -4.0),
            Orbs::new(-60.0, 0.),
            Orbs::new(-40.0, -1.0),
            Orbs::new(-20.0, -2.0)
        ]
    );
    test!(
        Requirement::Or(vec![Requirement::Free, b.clone()]),
        [Orbs::default()]
    );
    test!(
        Requirement::Or(vec![b.clone(), Requirement::Free]),
        [Orbs::default()]
    );

    world.restore_snapshot();
    world.snapshot();

    world.store_base_max_health(35, &CommandsOutput::NONE);
    world.store_base_max_energy(1., &CommandsOutput::NONE);
    test!(
        Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]),
        "❌"
    );
    world.store_skill(Skill::Regenerate, true, &CommandsOutput::NONE);
    test!(
        Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]),
        [Orbs::new(-30.0, -1.0)]
    );

    let req = Requirement::Or(vec![
        Requirement::Damage(10.0),
        Requirement::EnergySkill(Skill::Blaze, 1.0),
    ]);
    world.store_skill(Skill::Blaze, true, &CommandsOutput::NONE);
    world.store_base_max_energy(2., &CommandsOutput::NONE);
    test!(
        Requirement::And(vec![req.clone(), req.clone()]),
        [
            Orbs::new(-20.0, 0.),
            Orbs::new(-10.0, -1.0),
            Orbs::new(0., -2.0)
        ]
    );
}
