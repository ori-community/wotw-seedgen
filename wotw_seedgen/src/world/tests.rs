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
    seed_language::simulate::{Simulation, Snapshot},
    test_logger, Difficulty, Shard, Skill, WorldSettings, DEFAULT_SPAWN,
};

pub fn empty_test_world<'graph, 'settings>(
    graph: &'graph Graph,
    settings: &'settings WorldSettings,
    spawn: &str,
) -> World<'graph, 'settings> {
    let mut world = test_world(graph, settings, spawn);

    world.store_max_health(0, &[]);
    world.store_max_energy((0.).into(), &[]);
    world.store_shard_slots(0, &[]);

    world
}

pub fn test_world<'graph, 'settings>(
    graph: &'graph Graph,
    settings: &'settings WorldSettings,
    spawn: &str,
) -> World<'graph, 'settings> {
    let spawn = graph.find_node(spawn).unwrap();
    World::new(&*graph, spawn, settings, TEST_ASSETS.uber_states.clone())
}

#[test]
fn full_reach_check() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&TEST_ASSETS.graphs.gorlek, &settings, DEFAULT_SPAWN);

    let mut pool = ItemPoolBuilder::new(&mut Pcg64Mcg::new(0)).finish();
    for item in pool.take() {
        world.simulate(&item, &[]);
    }
    world.add_spirit_light(10000, &[]);

    world.traverse_spawn(&[]);

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
        eprintln!(
            "remaining uber state fails:\n{}",
            world
                .fails()
                .uber_state
                .values()
                .flatten()
                .cloned()
                .collect::<FxHashSet<_>>()
                .into_iter()
                .format_with("\n", |connection, f| {
                    f(&connection.display(world.graph))
                })
        );

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

    world.store_skill(Skill::DoubleJump, true, &[]);
    world.store_shard(Shard::TripleJump, true, &[]);
    world.add_max_health(5, &[]);

    world.traverse_spawn(&[]);

    let reached = world
        .reached_pickups()
        .map(|pickup| pickup.identifier.as_str())
        .collect::<FxHashSet<_>>();
    assert_eq!(
        reached,
        FxHashSet::from_iter([
            "GladesTown.UpdraftCeilingEX",
            "GladesTown.AboveTpEX",
            "GladesTown.BountyShard",
            "GladesTown.BelowHoleHutEX"
        ])
    );
}

#[test]
fn max_energy() {
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    assert_eq!(world.max_energy(), 0.0);

    world.add_max_energy((5.).into(), &[]);
    world.store_shard(Shard::Energy, true, &[]);
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
        world.add_max_health(5, &[]);
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
        world.add_max_health(5, &[]);
    }

    world.restore_snapshot();

    world.store_shard(Shard::Energy, true, &[]);
    world.store_shard(Shard::Vitality, true, &[]);

    assert_eq!(
        world.checkpoint_orbs(),
        Orbs {
            energy: 1.0,
            health: 0.0
        }
    );

    world.add_max_health(35, &[]);

    assert_eq!(
        world.checkpoint_orbs(),
        Orbs {
            health: 35.0,
            energy: 1.0
        }
    );

    world.add_max_health(105, &[]);

    assert_eq!(
        world.checkpoint_orbs(),
        Orbs {
            health: 45.0,
            energy: 1.0
        }
    );

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

    world.store_skill(Skill::Spear, true, &[]);
    assert_eq!(world.destroy_cost::<false>(10.0, true), Some(4.0));
    assert_eq!(world.destroy_cost::<false>(0.0, false), Some(0.0));

    world.store_skill(Skill::Bow, true, &[]);
    assert_eq!(world.destroy_cost::<false>(10.0, false), Some(1.5));

    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    world.store_skill(Skill::GladesAncestralLight, true, &[]);
    world.store_skill(Skill::MarshAncestralLight, true, &[]);
    world.store_shard(Shard::Wingclip, true, &[]);
    world.add_shard_slots(1, &[]);
    world.store_skill(Skill::Bow, false, &[]);
    assert_eq!(world.destroy_cost::<false>(1.0, false), Some(2.0));

    world.store_skill(Skill::Bow, true, &[]);
    assert_eq!(world.destroy_cost::<false>(10.0, true), Some(0.25));

    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Grenade, true, &[]);
    world.store_skill(Skill::Shuriken, true, &[]);
    assert_eq!(world.destroy_cost::<false>(20.0, false), Some(1.5));
    assert_eq!(world.destroy_cost::<false>(24.0, false), Some(1.5));
    assert_eq!(world.destroy_cost::<false>(34.0, false), Some(2.0));
}

#[test]
fn is_met() {
    macro_rules! test {
        ($world:expr, $req:expr, [$world_orbs:expr], "✅") => {
            test!($world, $req, [$world.max_orbs()], ControlFlow::is_continue);
        };
        ($world:expr, $req:expr, [$world_orbs:expr], "❌") => {
            test!($world, $req, [$world.max_orbs()], ControlFlow::is_break);
        };
        ($world:expr, $req:expr, [$world_orbs:expr], $f:path) => {
            {
                let req = $req;
                let mut orb_variants: OrbVariants = smallvec![$world_orbs];
                let control_flow = $world.is_met(&req, &mut orb_variants);
                assert!($f(&control_flow));
            }
        };
        ($world:expr, $req:expr, [$world_orbs:expr], [$($orbs:expr),* $(,)?]) => {
            {
                let req = $req;
                let mut left: OrbVariants = smallvec![$world_orbs];
                let _ = $world.is_met(&req, &mut left);
                left.sort_unstable_by(|a, b| a.health.total_cmp(&b.health));
                let mut right: OrbVariants = smallvec![$($world_orbs + $orbs),*];
                right.sort_unstable_by(|a, b| a.health.total_cmp(&b.health));
                assert_eq!(left, right);
            }
        };
        ($world:expr, $req:expr, $symbol:tt) => {
            test!($world, $req, [$world.max_orbs()], $symbol);
        };
        ($world:expr, $req:expr, [$($orbs:tt)*]) => {
            test!($world, $req, [$world.max_orbs()], [$($orbs)*]);
        };
    }

    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    let mut world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);

    let orbs = Orbs::default();

    test!(&world, Requirement::Skill(Skill::Blaze), "❌");
    world.store_skill(Skill::Blaze, true, &[]);
    test!(&world, Requirement::Skill(Skill::Blaze), "✅");

    test!(
        &world,
        Requirement::And(vec![Requirement::Skill(Skill::Blaze), Requirement::Free]),
        "✅"
    );
    test!(
        &world,
        Requirement::Or(vec![
            Requirement::Skill(Skill::Blaze),
            Requirement::Impossible
        ]),
        "✅"
    );

    test!(&world, Requirement::EnergySkill(Skill::Blaze, 1.0), "❌");
    world.add_max_energy((1.).into(), &[]);
    test!(&world, Requirement::EnergySkill(Skill::Blaze, 1.0), "❌");

    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs {
            energy: -1.0,
            ..orbs
        }]
    );
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    world.settings = &settings;
    world.add_max_energy((1.).into(), &[]);
    test!(
        &world,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );

    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Blaze, true, &[]);
    world.add_max_energy((0.5).into(), &[]);
    world.add_max_health(15, &[]);
    world.store_shard(Shard::LifePact, true, &[]);
    test!(
        &world,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs {
            energy: -0.5,
            health: -5.0
        }]
    );
    test!(
        &world,
        Requirement::NonConsumingEnergySkill(Skill::Blaze),
        [Orbs {
            health: -5.0,
            ..orbs
        }]
    );
    test!(
        &world,
        Requirement::NonConsumingEnergySkill(Skill::Blaze),
        [Orbs {
            energy: 0.0,
            health: world.max_health()
        }],
        [Orbs {
            energy: 0.5,
            health: -10.0
        }]
    );

    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.add_max_energy((2.).into(), &[]);
    world.add_max_health(30, &[]);
    test!(&world, Requirement::Damage(30.0), "❌");
    world.add_max_health(5, &[]);
    test!(
        &world,
        Requirement::Damage(30.0),
        [Orbs {
            health: -30.0,
            ..orbs
        }]
    );
    world.add_max_energy((1.).into(), &[]);
    world.store_skill(Skill::Regenerate, true, &[]);
    test!(&world, Requirement::Damage(60.0), "❌");
    world.add_max_health(30, &[]);
    test!(
        &world,
        Requirement::Damage(60.0),
        [Orbs {
            health: 30.0,
            energy: world.max_energy()
        }],
        [Orbs {
            health: -25.0,
            energy: -2.0
        }]
    );
    test!(
        &world,
        Requirement::Danger(30.0),
        [Orbs {
            health: 30.0,
            energy: world.max_energy()
        }],
        [Orbs {
            health: 30.0,
            energy: -1.0
        }]
    );
    test!(
        &world,
        Requirement::Danger(60.0),
        [Orbs {
            health: 30.0,
            energy: world.max_energy()
        }],
        [Orbs {
            health: 35.0,
            energy: -2.0
        }]
    );

    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    test!(&world, Requirement::BreakWall(12.0), "❌");
    world.store_skill(Skill::Sword, true, &[]);
    test!(&world, Requirement::BreakWall(12.0), [world.max_orbs()]);
    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Grenade, true, &[]);
    test!(&world, Requirement::BreakWall(12.0), "❌");
    world.add_max_energy((1.5).into(), &[]);
    test!(&world, Requirement::BreakWall(12.0), "❌");
    world.add_max_energy((0.5).into(), &[]);
    test!(
        &world,
        Requirement::BreakWall(12.0),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );
    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Grenade, true, &[]);
    world.add_max_energy((1.).into(), &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::BreakWall(16.0),
        [Orbs {
            energy: -1.0,
            ..orbs
        }]
    );
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    world.settings = &settings;
    world.add_max_energy((0.5).into(), &[]);
    test!(&world, Requirement::BreakWall(12.0), "❌");

    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Shuriken, true, &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    test!(&world, Requirement::ShurikenBreak(12.0), "❌");
    world.add_max_energy((2.).into(), &[]);
    test!(
        &world,
        Requirement::ShurikenBreak(12.0),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );
    world.add_max_energy((3.).into(), &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    world.settings = &settings;
    test!(&world, Requirement::ShurikenBreak(12.0), "❌");
    world.add_max_energy((1.).into(), &[]);
    test!(
        &world,
        Requirement::ShurikenBreak(12.0),
        [Orbs {
            energy: -6.0,
            ..orbs
        }]
    );

    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Bow, true, &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        "❌"
    );
    world.add_max_energy((3.5).into(), &[]);
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        [Orbs {
            energy: -3.25,
            ..orbs
        }]
    );
    world.add_max_energy((3.).into(), &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    world.settings = &settings;
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        "❌"
    );
    world.store_skill(Skill::DoubleJump, true, &[]);
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        [Orbs {
            energy: -6.5,
            ..orbs
        }]
    );
    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    let req = Requirement::Combat(smallvec![
        (Enemy::Sandworm, 1),
        (Enemy::Bat, 1),
        (Enemy::EnergyRefill, 99),
        (Enemy::ShieldMiner, 2),
        (Enemy::EnergyRefill, 1),
        (Enemy::Balloon, 4)
    ]);
    world.store_skill(Skill::Shuriken, true, &[]);
    world.store_skill(Skill::Spear, true, &[]);
    world.add_max_energy((13.5).into(), &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    test!(&world, &req, "❌");
    world.add_max_energy((0.5).into(), &[]);
    test!(
        &world,
        &req,
        [Orbs {
            energy: -14.0,
            ..orbs
        }]
    );
    world.add_max_energy((18.5).into(), &[]);
    world.store_skill(Skill::Bash, true, &[]);
    world.store_skill(Skill::Launch, true, &[]);
    world.store_skill(Skill::Burrow, true, &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    world.settings = &settings;
    test!(&world, &req, "❌");
    world.add_max_energy((0.5).into(), &[]);
    test!(
        &world,
        &req,
        [Orbs {
            energy: -33.0,
            ..orbs
        }]
    );
    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Spear, true, &[]);
    world.store_skill(Skill::DoubleJump, true, &[]);
    world.add_max_energy((2.).into(), &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    world.settings = &settings;
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );
    let settings = WorldSettings::difficulty_default(Difficulty::Moki);
    world.settings = &settings;
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        "❌"
    );
    world.add_max_energy((5.5).into(), &[]);
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        "❌"
    );
    world.add_max_energy((0.5).into(), &[]);
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        [Orbs {
            energy: -8.0,
            ..orbs
        }]
    );

    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    let a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let b = Requirement::Damage(20.0);
    let c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let d = Requirement::Damage(10.0);
    world.store_skill(Skill::Blaze, true, &[]);
    world.add_max_energy((2.).into(), &[]);
    world.add_max_health(25, &[]);
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::And(vec![c.clone(), d.clone()]),
        [Orbs {
            health: -10.0,
            energy: -1.0
        }]
    );
    test!(
        &world,
        Requirement::Or(vec![a.clone(), b.clone()]),
        [
            Orbs {
                energy: -2.0,
                ..orbs
            },
            Orbs {
                health: -20.0,
                ..orbs
            }
        ]
    );
    test!(
        &world,
        Requirement::Or(vec![
            Requirement::And(vec![a.clone(), b.clone()]),
            Requirement::And(vec![c.clone(), d.clone()]),
            a.clone(),
            b.clone()
        ]),
        [
            Orbs {
                energy: -1.0,
                health: -10.0
            },
            Orbs {
                energy: -2.0,
                ..orbs
            },
            Orbs {
                health: -20.0,
                ..orbs
            }
        ]
    );
    test!(
        &world,
        Requirement::And(vec![
            Requirement::Or(vec![a.clone(), d.clone()]),
            Requirement::Or(vec![b.clone(), c.clone()])
        ]),
        [Orbs {
            energy: -1.0,
            health: -10.0
        }]
    );
    world.add_max_health(40, &[]);
    world.add_max_energy((4.).into(), &[]);
    test!(
        &world,
        Requirement::And(vec![
            Requirement::Or(vec![a.clone(), d.clone()]),
            Requirement::Or(vec![b.clone(), c.clone()]),
            Requirement::Or(vec![a.clone(), d.clone()]),
            Requirement::Or(vec![b.clone(), c.clone()])
        ]),
        [
            Orbs {
                energy: -6.0,
                ..orbs
            },
            Orbs {
                energy: -4.0,
                health: -10.0
            },
            Orbs {
                health: -60.0,
                ..orbs
            },
            Orbs {
                energy: -1.0,
                health: -40.0
            },
            Orbs {
                energy: -2.0,
                health: -20.0
            }
        ]
    );
    test!(
        &world,
        Requirement::Or(vec![Requirement::Free, b.clone()]),
        [Orbs::default()]
    );
    test!(
        &world,
        Requirement::Or(vec![b.clone(), Requirement::Free]),
        [Orbs::default()]
    );

    world = empty_test_world(&TEST_ASSETS.graphs.moki, &settings, DEFAULT_SPAWN);
    let settings = WorldSettings::difficulty_default(Difficulty::Unsafe);
    world.settings = &settings;
    world.add_max_health(35, &[]);
    world.add_max_energy((1.).into(), &[]);
    test!(
        &world,
        Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]),
        "❌"
    );
    world.store_skill(Skill::Regenerate, true, &[]);
    test!(
        &world,
        Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]),
        [Orbs {
            energy: -1.0,
            health: -30.0
        }]
    );

    let req = Requirement::Or(vec![
        Requirement::Damage(10.0),
        Requirement::EnergySkill(Skill::Blaze, 1.0),
    ]);
    world.store_skill(Skill::Blaze, true, &[]);
    world.add_max_energy((1.).into(), &[]);
    test!(
        &world,
        Requirement::And(vec![req.clone(), req.clone()]),
        [
            Orbs {
                health: -20.0,
                ..orbs
            },
            Orbs {
                health: -10.0,
                energy: -1.0
            },
            Orbs {
                energy: -2.0,
                ..orbs
            }
        ]
    );
}
