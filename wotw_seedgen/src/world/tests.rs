use std::{ops::ControlFlow, sync::LazyLock};

use super::*;
use crate::{
    item_pool::ItemPool,
    tests::test_logger,
    world::reached::{Progression, ALL_CONNECTIONS},
};
use itertools::Itertools;
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashSet;
use wotw_seedgen_data::{
    assets::{AssetCacheValues, AssetFileAccess, TEST_ASSETS},
    logic_language::{
        ast::Areas,
        output::{Enemy, Node, Requirement},
    },
    Difficulty, DEFAULT_SPAWN,
};

fn test_settings(difficulty: Difficulty) -> WorldSettings {
    WorldSettings {
        difficulty,
        ..Default::default()
    }
}

fn empty_test_world<'settings>(
    settings: &'settings WorldSettings,
    spawn: &str,
) -> World<'static, 'settings> {
    let mut world = test_world(settings, spawn);

    world.store_max_health(0, &[]);
    world.store_max_energy((0.).into(), &[]);
    world.store_shard_slots(0, &[]);

    world
}

static GRAPH: LazyLock<Graph> = LazyLock::new(|| {
    let source = TEST_ASSETS.values.areas();
    let areas = Areas::parse(&source.content).eprint_errors(source).unwrap();

    Graph::compile(
        areas,
        TEST_ASSETS.loc_data().unwrap(),
        TEST_ASSETS.state_data().unwrap(),
        &[],
    )
    .eprint_errors(source)
    .unwrap()
});

fn test_world<'settings>(
    settings: &'settings WorldSettings,
    spawn: &str,
) -> World<'static, 'settings> {
    let spawn = GRAPH.find_node(spawn).unwrap();
    let uber_states = UberStates::new(TEST_ASSETS.values.uber_state_data());

    World::new(&*GRAPH, spawn, settings, uber_states)
}

#[test]
fn full_reach_check() {
    test_logger();

    let settings = test_settings(Difficulty::Gorlek);
    let mut world = test_world(&settings, DEFAULT_SPAWN);

    let mut pool = ItemPool::new(&mut Pcg64Mcg::new(0));
    for item in pool.drain(..) {
        world.simulate(&item, &[]);
    }
    world.add_spirit_light(10000, &[]);

    world.traverse_spawn(&[]);

    let reached = world
        .reached_nodes()
        .filter_map(|node| match node {
            Node::Pickup(_) => Some(node.identifier()),
            _ => None,
        })
        .collect();

    let all_locations = TEST_ASSETS
        .values
        .loc_data()
        .entries
        .iter()
        .map(|location| location.identifier.as_str())
        .collect::<FxHashSet<_>>();

    if !(reached == all_locations) {
        fn format_progressions<'a, I>(
            progressions: I,
            world: &'a World,
        ) -> impl Display + use<'a, I>
        where
            I: IntoIterator<Item = &'a Progression>,
        {
            progressions
                .into_iter()
                .format_with(", ", |progression, f| {
                    let anchor = world.graph.nodes[progression.node_index].expect_anchor();
                    if progression.connection_index == ALL_CONNECTIONS {
                        f(&format_args!("{} (all connections)", anchor.identifier))
                    } else {
                        let connection = &anchor.connections[progression.connection_index];
                        f(&format_args!(
                            "{from} -> {to}",
                            from = anchor.identifier,
                            to = world.graph.nodes[connection.to].identifier()
                        ))
                    }
                })
        }

        let mut uber_state_progressions = world
            .reach
            .uber_state_progressions
            .iter()
            .collect::<Vec<_>>();
        uber_state_progressions.sort_unstable_by_key(|(uber_identifier, _)| **uber_identifier);
        eprintln!(
            "remaining uber state progressions:\n{}",
            uber_state_progressions.iter().format_with(
                "\n",
                |(uber_identifier, progressions), f| {
                    f(&format_args!(
                        "{}: {}",
                        TEST_ASSETS.values.uber_state_data().id_lookup[uber_identifier]
                            .preferred_name(),
                        format_progressions(*progressions, &world)
                    ))
                }
            )
        );

        eprintln!(
            "remaining orb progressions: {}",
            world.reach.orb_progression
        );

        let mut diff = all_locations.difference(&reached).collect::<Vec<_>>();
        diff.sort_unstable();
        eprintln!(
            "difference (reached {reached_len} / {total_len} items): {diff:?}",
            reached_len = reached.len(),
            total_len = all_locations.len(),
        );
    }

    assert_eq!(reached, all_locations);
}

#[test]
fn small_reach_check() {
    test_logger();

    let settings = test_settings(Difficulty::Gorlek);
    let mut world = test_world(&settings, "GladesTown.Teleporter");

    world.store_skill(Skill::DoubleJump, true, &[]);
    world.store_shard(Shard::TripleJump, true, &[]);
    world.add_max_health(5, &[]);

    world.traverse_spawn(&[]);

    let reached = world
        .reached_nodes()
        .filter(|node| node.can_place())
        .map(Node::identifier)
        .collect::<FxHashSet<_>>();
    assert_eq!(
        reached,
        [
            "GladesTown.UpdraftCeilingEX",
            "GladesTown.AboveTpEX",
            "GladesTown.BountyShard",
            "GladesTown.BelowHoleHutEX"
        ]
        .into_iter()
        .collect()
    );
}

// TODO these tests look like they belong into Inventory now
#[test]
fn weapon_preference() {
    let settings = test_settings(Difficulty::Moki);
    let mut world = empty_test_world(&settings, DEFAULT_SPAWN);
    assert_eq!(
        world.progression_weapons::<false>(),
        SmallVec::from_buf([
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
            Skill::Blaze,
            Skill::Flash,
            Skill::Spear,
        ])
    );

    world.store_skill(Skill::Shuriken, true, &[]);
    assert_eq!(
        world.progression_weapons::<false>(),
        SmallVec::from_buf([
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ])
    );

    let settings = test_settings(Difficulty::Gorlek);
    world.settings = &settings;

    assert_eq!(
        world.progression_weapons::<false>(),
        SmallVec::from_buf([
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
        ])
    );
}

#[test]
fn max_energy() {
    let settings = test_settings(Difficulty::Moki);
    let mut world = empty_test_world(&settings, DEFAULT_SPAWN);
    assert_eq!(world.max_energy(), 0.0);

    world.add_max_energy((5.).into(), &[]);
    world.store_shard(Shard::Energy, true, &[]);
    assert_eq!(world.max_energy(), 5.0);

    let settings = test_settings(Difficulty::Gorlek);
    world.settings = &settings;
    assert_eq!(world.max_energy(), 6.0);
}

#[test]
fn refill_orbs() {
    let settings = test_settings(Difficulty::Gorlek);
    let mut world = empty_test_world(&settings, DEFAULT_SPAWN);
    world.snapshot(0);

    let expected = [
        0., 5., 10., 15., 20., 25., 30., 35., 40., 40., 40., 40., 40., 40., 40., 40., 40., 40.,
        40., 40., 40., 40., 40., 40., 40., 40., 40., 41., 42., 44., 45., 47., 48., 50., 52., 53.,
        55., 56., 58., 59., 61., 62., 64., 65., 66., 68., 69.,
    ];
    for health in expected {
        assert_eq!(world.checkpoint_orbs().health, health);
        world.add_max_health(5, &[]);
    }

    world.restore_snapshot(0);
    world.snapshot(0);

    let expected = [
        0., 0., 0., 0., 1., 1., 1., 1., 1., 2., 2., 2., 2., 2., 2., 2., 3., 3., 3., 3., 3., 4., 4.,
        4., 4., 4., 4., 4., 5., 5., 5., 5., 5., 6., 6., 6., 6., 6., 6., 6., 7., 7., 7., 7., 7., 8.,
        8.,
    ];
    for drops in expected {
        assert_eq!(world.health_plant_drops(), drops);
        world.add_max_health(5, &[]);
    }

    world.restore_snapshot(0);

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

    let world = test_world(&settings, DEFAULT_SPAWN);

    let mut orb_variants = smallvec![Orbs::default()];
    world.refill(RefillValue::Full, &mut orb_variants);
    assert_eq!(&orb_variants[..], &[world.max_orbs()]);
}

#[test]
fn destroy_cost() {
    let settings = test_settings(Difficulty::Moki);
    let mut world = empty_test_world(&settings, DEFAULT_SPAWN);
    assert_eq!(world.destroy_cost::<false>(10.0, false), None);

    world.store_skill(Skill::Spear, true, &[]);
    assert_eq!(world.destroy_cost::<false>(10.0, true), Some(4.0));
    assert_eq!(world.destroy_cost::<false>(0.0, false), Some(0.0));

    world.store_skill(Skill::Bow, true, &[]);
    assert_eq!(world.destroy_cost::<false>(10.0, false), Some(1.5));

    let settings = test_settings(Difficulty::Unsafe);
    world.settings = &settings;
    world.store_skill(Skill::GladesAncestralLight, true, &[]);
    world.store_skill(Skill::MarshAncestralLight, true, &[]);
    world.store_shard(Shard::Wingclip, true, &[]);
    world.add_shard_slots(1, &[]);
    world.store_skill(Skill::Bow, false, &[]);
    assert_eq!(world.destroy_cost::<false>(1.0, false), Some(2.0));

    world.store_skill(Skill::Bow, true, &[]);
    assert_eq!(world.destroy_cost::<false>(10.0, true), Some(0.25));

    let mut world = empty_test_world(&settings, DEFAULT_SPAWN);
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
                let mut orb_variants: OrbVariants = smallvec![$world_orbs];
                let control_flow = $world.is_met(&$req, &mut orb_variants);
                assert!($f(&control_flow));
            }
        };
        ($world:expr, $req:expr, [$world_orbs:expr], [$($orbs:expr),* $(,)?]) => {
            {
                let mut left: OrbVariants = smallvec![$world_orbs];
                let _ = $world.is_met(&$req, &mut left);
                left.sort_unstable_by_key(|orbs: &Orbs| OrderedFloat(orbs.health));
                let mut right: OrbVariants = smallvec![$($world_orbs + $orbs),*];
                right.sort_unstable_by_key(|orbs: &Orbs| OrderedFloat(orbs.health));
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

    let settings = test_settings(Difficulty::Moki);
    let mut world = empty_test_world(&settings, DEFAULT_SPAWN);

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

    let settings = test_settings(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs {
            energy: -1.0,
            ..orbs
        }]
    );
    let settings = test_settings(Difficulty::Moki);
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

    let settings = test_settings(Difficulty::Unsafe);
    world = empty_test_world(&settings, DEFAULT_SPAWN);
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

    world = empty_test_world(&settings, DEFAULT_SPAWN);
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

    let settings = test_settings(Difficulty::Moki);
    world = empty_test_world(&settings, DEFAULT_SPAWN);
    test!(&world, Requirement::BreakWall(12.0), "❌");
    world.store_skill(Skill::Sword, true, &[]);
    test!(&world, Requirement::BreakWall(12.0), [world.max_orbs()]);
    world = empty_test_world(&settings, DEFAULT_SPAWN);
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
    world = empty_test_world(&settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Grenade, true, &[]);
    world.add_max_energy((1.).into(), &[]);
    let settings = test_settings(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::BreakWall(16.0),
        [Orbs {
            energy: -1.0,
            ..orbs
        }]
    );
    let settings = test_settings(Difficulty::Moki);
    world.settings = &settings;
    world.add_max_energy((0.5).into(), &[]);
    test!(&world, Requirement::BreakWall(12.0), "❌");

    world = empty_test_world(&settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Shuriken, true, &[]);
    let settings = test_settings(Difficulty::Unsafe);
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
    let settings = test_settings(Difficulty::Moki);
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

    world = empty_test_world(&settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Bow, true, &[]);
    let settings = test_settings(Difficulty::Unsafe);
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
    let settings = test_settings(Difficulty::Moki);
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
    world = empty_test_world(&settings, DEFAULT_SPAWN);
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
    let settings = test_settings(Difficulty::Unsafe);
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
    let settings = test_settings(Difficulty::Moki);
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
    world = empty_test_world(&settings, DEFAULT_SPAWN);
    world.store_skill(Skill::Spear, true, &[]);
    world.store_skill(Skill::DoubleJump, true, &[]);
    world.add_max_energy((2.).into(), &[]);
    let settings = test_settings(Difficulty::Gorlek);
    world.settings = &settings;
    let settings = test_settings(Difficulty::Unsafe);
    world.settings = &settings;
    test!(
        &world,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );
    let settings = test_settings(Difficulty::Moki);
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

    world = empty_test_world(&settings, DEFAULT_SPAWN);
    let a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let b = Requirement::Damage(20.0);
    let c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let d = Requirement::Damage(10.0);
    world.store_skill(Skill::Blaze, true, &[]);
    world.add_max_energy((2.).into(), &[]);
    world.add_max_health(25, &[]);
    let settings = test_settings(Difficulty::Unsafe);
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

    world = empty_test_world(&settings, DEFAULT_SPAWN);
    let settings = test_settings(Difficulty::Unsafe);
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

// TODO
// #[test]
// fn solutions() {
//     macro_rules! test {
//         ($world:expr, $states:expr, $req:expr, [$($world_orbs:expr),* $(,)?], [$($solutions:expr),* $(,)?]) => {
//             {
//                 fn sort(mut solutions: Vec<Inventory>) -> Vec<Inventory> {
//                     solutions.sort_unstable_by_key(|inventory| {
//                         let mut items = inventory.items.iter().map(|(item, amount)| format!("{item}{amount}")).collect::<Vec<_>>();
//                         items.sort_unstable();
//                         items.concat()
//                     });  // dumb string based sort
//                     solutions
//                 }
//                 assert_eq!(sort($req.solutions($world, $states, smallvec![$($world_orbs),*], 1000, 1000)), sort(vec![$($solutions),*]));
//             }
//         };
//         ($world:expr, $states:expr, $req:expr, [$($solutions:tt)*]) => {
//             test!($world, $states, $req, [$world.max_orbs()], [$($solutions)*]);
//         };
//     }

//     let settings = test_settings(Difficulty::Moki);
//     let mut world = test_world(&settings, DEFAULT_SPAWN);
//     let states = FxHashSet::default();

//     test!(&world, Requirement::Free, [Inventory::default()]);
//     test!(&world, Requirement::Impossible, "❌");
//     test!(
//         &world,

//         Requirement::Or(vec![Requirement::Free, Requirement::Impossible]),
//         [Inventory::default()]
//     );
//     test!(
//         &world,

//         Requirement::And(vec![Requirement::Free, Requirement::Impossible]),
//         []
//     );

//     test!(
//         &world,

//         Requirement::Skill(Skill::Dash),
//         [Item::Skill(Skill::Dash).into()]
//     );
//     test!(
//         &world,

//         Requirement::Or(vec![
//             Requirement::Skill(Skill::Dash),
//             Requirement::Skill(Skill::Bash)
//         ]),
//         [
//             Item::Skill(Skill::Dash).into(),
//             Item::Skill(Skill::Bash).into()
//         ]
//     );
//     test!(
//         &world,

//         Requirement::And(vec![
//             Requirement::Skill(Skill::Dash),
//             Requirement::Skill(Skill::Bash)
//         ]),
//         [[Item::Skill(Skill::Dash), Item::Skill(Skill::Bash)]
//             .into_iter()
//             .collect()]
//     );

//     test!(
//         &world,

//         Requirement::EnergySkill(Skill::Grenade, 2.0),
//         [[
//             (Item::Skill(Skill::Grenade), 1),
//             (Item::Resource(Resource::EnergyFragment), 8)
//         ]
//         .into_iter()
//         .collect()]
//     );

//     let settings = WorldSettings {
//         difficulty: Difficulty::Unsafe,
//         ..test_settings(Difficulty::Moki)
//     };
//     world.settings = &settings;
//     world
//         .inventory
//         .add_resource(Resource::HealthFragment, 8);
//     // TODO this should really be equivalent to Requirement::EnergySkill(Skill::Grenade, 2.0)
//     test!(
//         &world,

//         Requirement::And(vec![
//             Requirement::EnergySkill(Skill::Grenade, 1.0),
//             Requirement::EnergySkill(Skill::Grenade, 1.0)
//         ]),
//         [Orbs::default()],
//         [
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 5)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Resilience), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Resilience), 1),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Shard(Shard::Resilience), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Skill(Skill::Regenerate), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );

//     let settings = WorldSettings {
//         difficulty: Difficulty::Moki,
//         ..test_settings(Difficulty::Moki)
//     };
//     world = test_world(&settings, DEFAULT_SPAWN);
//     test!(
//         &world,

//         Requirement::Resource(Resource::ShardSlot, 3),
//         [(Item::Resource(Resource::ShardSlot), 3).into()]
//     );
//     test!(
//         &world,

//         Requirement::Shard(Shard::Overflow),
//         [Item::Shard(Shard::Overflow).into()]
//     );
//     test!(
//         &world,

//         Requirement::Teleporter(Teleporter::Glades),
//         [Item::Teleporter(Teleporter::Glades).into()]
//     );
//     test!(&world, Requirement::Water, [Item::Water.into()]);

//     test!(
//         &world,

//         Requirement::Damage(36.0),
//         [(Item::Resource(Resource::HealthFragment), 8).into()]
//     );
//     test!(
//         &world,

//         Requirement::And(vec![Requirement::Damage(18.0), Requirement::Damage(18.0)]),
//         [
//             (Item::Resource(Resource::HealthFragment), 8).into(),
//             [
//                 (Item::Resource(Resource::HealthFragment), 4),
//                 (Item::Resource(Resource::EnergyFragment), 4),
//                 (Item::Skill(Skill::Regenerate), 1)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );
//     test!(
//         &world,

//         Requirement::Or(vec![Requirement::Damage(36.0), Requirement::Damage(18.0)]),
//         [(Item::Resource(Resource::HealthFragment), 4).into()]
//     );

//     let settings = WorldSettings {
//         difficulty: Difficulty::Unsafe,
//         ..test_settings(Difficulty::Moki)
//     };
//     world.settings = &settings;
//     test!(
//         &world,

//         Requirement::And(vec![
//             Requirement::Damage(18.0),
//             Requirement::Damage(18.0),
//             Requirement::Damage(18.0)
//         ]),
//         [
//             (Item::Resource(Resource::HealthFragment), 11).into(),
//             [
//                 (Item::Shard(Shard::Resilience), 1),
//                 (Item::Resource(Resource::HealthFragment), 10)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Regenerate), 1),
//                 (Item::Resource(Resource::HealthFragment), 8),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Regenerate), 1),
//                 (Item::Resource(Resource::HealthFragment), 4),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Regenerate), 1),
//                 (Item::Shard(Shard::Resilience), 1),
//                 (Item::Resource(Resource::HealthFragment), 7),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Regenerate), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 4),
//                 (Item::Resource(Resource::EnergyFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );

//     let settings = WorldSettings {
//         difficulty: Difficulty::Moki,
//         ..test_settings(Difficulty::Moki)
//     };
//     world.settings = &settings;
//     test!(
//         &world,

//         Requirement::BreakWall(12.0),
//         [
//             Item::Skill(Skill::Sword).into(),
//             Item::Skill(Skill::Hammer).into(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Resource(Resource::EnergyFragment), 8)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );

//     let settings = WorldSettings {
//         difficulty: Difficulty::Unsafe,
//         ..test_settings(Difficulty::Moki)
//     };
//     world.settings = &settings;
//     test!(
//         &world,

//         Requirement::BreakWall(12.0),
//         [
//             Item::Skill(Skill::Sword).into(),
//             Item::Skill(Skill::Hammer).into(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 5)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 5)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Sentry), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );
//     world.store_skill(Skill::Bow, true, &[]);
//     test!(
//         &world,

//         Requirement::BreakWall(12.0),
//         [
//             Item::Skill(Skill::Sword).into(),
//             Item::Skill(Skill::Hammer).into(),
//             [(Item::Resource(Resource::EnergyFragment), 2)]
//                 .into_iter()
//                 .collect(),
//             [
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );

//     let settings = test_settings(Difficulty::Moki);
//     let mut world = test_world(&settings, DEFAULT_SPAWN);
//     test!(
//         &world,

//         Requirement::Combat(smallvec![(Enemy::Slug, 1)]),
//         [
//             Item::Skill(Skill::Sword).into(),
//             Item::Skill(Skill::Hammer).into(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Flash), 1),
//                 (Item::Resource(Resource::EnergyFragment), 8)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Resource(Resource::EnergyFragment), 8)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );
//     world.store_skill(Skill::Launch, true, &[]);
//     test!(
//         &world,

//         Requirement::Combat(smallvec![
//             (Enemy::Skeeto, 2),
//             (Enemy::EnergyRefill, 2),
//             (Enemy::Mantis, 1),
//             (Enemy::SmallSkeeto, 4),
//             (Enemy::EnergyRefill, 2),
//             (Enemy::Mantis, 1),
//             (Enemy::Skeeto, 1)
//         ]),
//         [
//             Item::Skill(Skill::Sword).into(),
//             Item::Skill(Skill::Hammer).into(),
//             [
//                 (Item::Skill(Skill::Bow), 1),
//                 (Item::Resource(Resource::EnergyFragment), 31)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Grenade), 1),
//                 (Item::Resource(Resource::EnergyFragment), 56)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Shuriken), 1),
//                 (Item::Resource(Resource::EnergyFragment), 46)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Blaze), 1),
//                 (Item::Resource(Resource::EnergyFragment), 56)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Flash), 1),
//                 (Item::Resource(Resource::EnergyFragment), 56)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Spear), 1),
//                 (Item::Resource(Resource::EnergyFragment), 80)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );
//     let settings = WorldSettings {
//         difficulty: Difficulty::Unsafe,
//         ..test_settings(Difficulty::Moki)
//     };
//     world.settings = &settings;
//     world.store_skill(Skill::Bow, true, &[]);
//     // 40 + 32 + (20 * 2) + 24 * 2 + 20 * 3 + 32
//     // 10 + 8 + (10) + 12 + 15 + 8 = 63
//     test!(
//         &world,

//         Requirement::Combat(smallvec![
//             (Enemy::Hornbug, 1),
//             (Enemy::Bat, 1),
//             (Enemy::Sandworm, 2),
//             (Enemy::Lizard, 2),
//             (Enemy::Skeeto, 3),
//             (Enemy::SneezeSlug, 1)
//         ]),
//         [
//             Item::Skill(Skill::Sword).into(),
//             Item::Skill(Skill::Hammer).into(),
//             [(Item::Resource(Resource::EnergyFragment), 32)]
//                 .into_iter()
//                 .collect(), // 15.75
//             [
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 16)
//             ]
//             .into_iter()
//             .collect(), // 7.875
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 31),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 30),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 29),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 28),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 27),
//                 (Item::Resource(Resource::HealthFragment), 5)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 26),
//                 (Item::Resource(Resource::HealthFragment), 6)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 25),
//                 (Item::Resource(Resource::HealthFragment), 7)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 24),
//                 (Item::Resource(Resource::HealthFragment), 8)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 23),
//                 (Item::Resource(Resource::HealthFragment), 9)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 22),
//                 (Item::Resource(Resource::HealthFragment), 10)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 21),
//                 (Item::Resource(Resource::HealthFragment), 11)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 20),
//                 (Item::Resource(Resource::HealthFragment), 12)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 19),
//                 (Item::Resource(Resource::HealthFragment), 13)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 18),
//                 (Item::Resource(Resource::HealthFragment), 14)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 17),
//                 (Item::Resource(Resource::HealthFragment), 15)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 16),
//                 (Item::Resource(Resource::HealthFragment), 16)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 15),
//                 (Item::Resource(Resource::HealthFragment), 17)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 14),
//                 (Item::Resource(Resource::HealthFragment), 18)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 13),
//                 (Item::Resource(Resource::HealthFragment), 19)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 12),
//                 (Item::Resource(Resource::HealthFragment), 20)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 11),
//                 (Item::Resource(Resource::HealthFragment), 21)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 10),
//                 (Item::Resource(Resource::HealthFragment), 22)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 9),
//                 (Item::Resource(Resource::HealthFragment), 23)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 8),
//                 (Item::Resource(Resource::HealthFragment), 24)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 7),
//                 (Item::Resource(Resource::HealthFragment), 25)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 6),
//                 (Item::Resource(Resource::HealthFragment), 26)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 5),
//                 (Item::Resource(Resource::HealthFragment), 27)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4),
//                 (Item::Resource(Resource::HealthFragment), 28)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3),
//                 (Item::Resource(Resource::HealthFragment), 29)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2),
//                 (Item::Resource(Resource::HealthFragment), 30)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 31)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 32)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 15),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 14),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 13),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 12),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 11),
//                 (Item::Resource(Resource::HealthFragment), 5)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 10),
//                 (Item::Resource(Resource::HealthFragment), 6)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 9),
//                 (Item::Resource(Resource::HealthFragment), 7)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 8),
//                 (Item::Resource(Resource::HealthFragment), 8)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 7),
//                 (Item::Resource(Resource::HealthFragment), 9)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 6),
//                 (Item::Resource(Resource::HealthFragment), 10)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 5),
//                 (Item::Resource(Resource::HealthFragment), 11)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4),
//                 (Item::Resource(Resource::HealthFragment), 12)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3),
//                 (Item::Resource(Resource::HealthFragment), 13)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2),
//                 (Item::Resource(Resource::HealthFragment), 14)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 15)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 16)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Resource(Resource::EnergyFragment), 27)
//             ]
//             .into_iter()
//             .collect(), // 13.25
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 14)
//             ]
//             .into_iter()
//             .collect(), // 6.625
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 26),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 25),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 24),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 23),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 22),
//                 (Item::Resource(Resource::HealthFragment), 5)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 21),
//                 (Item::Resource(Resource::HealthFragment), 6)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 20),
//                 (Item::Resource(Resource::HealthFragment), 7)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 19),
//                 (Item::Resource(Resource::HealthFragment), 8)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 18),
//                 (Item::Resource(Resource::HealthFragment), 9)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 17),
//                 (Item::Resource(Resource::HealthFragment), 10)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 16),
//                 (Item::Resource(Resource::HealthFragment), 11)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 15),
//                 (Item::Resource(Resource::HealthFragment), 12)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 14),
//                 (Item::Resource(Resource::HealthFragment), 13)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 13),
//                 (Item::Resource(Resource::HealthFragment), 14)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 12),
//                 (Item::Resource(Resource::HealthFragment), 15)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 11),
//                 (Item::Resource(Resource::HealthFragment), 16)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 10),
//                 (Item::Resource(Resource::HealthFragment), 17)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 9),
//                 (Item::Resource(Resource::HealthFragment), 18)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 8),
//                 (Item::Resource(Resource::HealthFragment), 19)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 7),
//                 (Item::Resource(Resource::HealthFragment), 20)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 6),
//                 (Item::Resource(Resource::HealthFragment), 21)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 5),
//                 (Item::Resource(Resource::HealthFragment), 22)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4),
//                 (Item::Resource(Resource::HealthFragment), 23)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3),
//                 (Item::Resource(Resource::HealthFragment), 24)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2),
//                 (Item::Resource(Resource::HealthFragment), 25)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 26)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Resource(Resource::HealthFragment), 27)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 13),
//                 (Item::Resource(Resource::HealthFragment), 1)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 12),
//                 (Item::Resource(Resource::HealthFragment), 2)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 11),
//                 (Item::Resource(Resource::HealthFragment), 3)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 10),
//                 (Item::Resource(Resource::HealthFragment), 4)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 9),
//                 (Item::Resource(Resource::HealthFragment), 5)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 8),
//                 (Item::Resource(Resource::HealthFragment), 6)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 7),
//                 (Item::Resource(Resource::HealthFragment), 7)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 6),
//                 (Item::Resource(Resource::HealthFragment), 8)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 5),
//                 (Item::Resource(Resource::HealthFragment), 9)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 4),
//                 (Item::Resource(Resource::HealthFragment), 10)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 3),
//                 (Item::Resource(Resource::HealthFragment), 11)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 2),
//                 (Item::Resource(Resource::HealthFragment), 12)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::EnergyFragment), 1),
//                 (Item::Resource(Resource::HealthFragment), 13)
//             ]
//             .into_iter()
//             .collect(),
//             [
//                 (Item::Skill(Skill::Burrow), 1),
//                 (Item::Shard(Shard::LifePact), 1),
//                 (Item::Shard(Shard::Overcharge), 1),
//                 (Item::Resource(Resource::HealthFragment), 14)
//             ]
//             .into_iter()
//             .collect(),
//         ]
//     );
// }
