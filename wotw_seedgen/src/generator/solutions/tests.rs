use std::{
    fmt::{self, Display},
    slice,
    sync::LazyLock,
};

use itertools::Itertools;
use rand_pcg::Pcg64Mcg;
use rustc_hash::FxHashMap;
use smallvec::smallvec;
use wotw_seedgen_data::{
    assets::TEST_ASSETS,
    logic_language::{
        ast::{Areas, Content},
        output::{Enemy, Graph, Requirement},
    },
    parse::SpannedOption,
    seed_language::{
        compile::{clean_water, energy_fragment, health_fragment, keystone, shard, skill},
        output::{CommandVoid, CommonItem, ContainedWrites},
        simulate::{Simulate, Simulation, UberStates},
    },
    test_logger, Difficulty,
    Shard::*,
    Skill::*,
    Teleporter, WorldSettings,
};

use crate::{
    item_pool::{ItemPool, ItemPoolBuilder},
    world::tests::test_world,
    World,
};

fn mock_world<'graph, 'settings>(
    graph: &'graph Graph,
    settings: &'settings WorldSettings,
    uber_states: UberStates,
) -> World<'graph, 'settings> {
    let mut world = World::new(graph, 0, settings, uber_states);
    world.store_max_health(0, &[]);
    world.store_max_energy((0.).into(), &[]);
    world.store_shard_slots(0, &[]);
    world.traverse_spawn(&[]);
    world
}

fn find_test_solutions(
    world: &mut World,
    item_pool: &ItemPool,
    slots: usize,
) -> Vec<Vec<(CommandVoid, u32)>> {
    sorted_test_solutions(
        world
            .find_solutions(&item_pool, &[], slots, 0, Some(u8::MAX))
            .into_iter()
            .map(|solution| {
                amounts_from_item_list(
                    solution
                        .items
                        .into_iter()
                        .map(|item| (*item_pool[item]).clone()),
                )
            })
            .collect(),
    )
}

fn amounts_from_item_list<I>(items: I) -> Vec<(CommandVoid, u32)>
where
    I: IntoIterator<Item = CommandVoid>,
{
    let mut map = FxHashMap::default();

    for item in items {
        *map.entry(item).or_default() += 1;
    }

    map.into_iter().collect()
}

fn sorted_test_solutions(
    mut solutions: Vec<Vec<(CommandVoid, u32)>>,
) -> Vec<Vec<(CommandVoid, u32)>> {
    sort_test_solutions(&mut solutions);
    solutions
}

fn sort_test_solutions(solutions: &mut Vec<Vec<(CommandVoid, u32)>>) {
    for solution in &mut *solutions {
        solution.sort_by_cached_key(solution_sort_key);
    }

    solutions
        .sort_by_cached_key(|solution| solution.iter().map(solution_sort_key).collect::<Vec<_>>());
}

fn solution_sort_key((item, _): &(CommandVoid, u32)) -> CommonItem {
    item.contained_common_items().next().unwrap()
}

fn display_test_solutions(solutions: &[Vec<(CommandVoid, u32)>]) -> DisplayTestSolutions<'_> {
    DisplayTestSolutions { solutions }
}

struct DisplayTestSolutions<'a> {
    solutions: &'a [Vec<(CommandVoid, u32)>],
}

impl Display for DisplayTestSolutions<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{solutions}]",
            solutions = self
                .solutions
                .iter()
                .format_with(", ", |items, f| f(&format_args!(
                    "[{items}]",
                    items = items
                        .iter()
                        .format_with(", ", |(item, amount), f| f(&format_args!(
                            "({item}, {amount})",
                            item = item.log_display()
                        )))
                )))
        )
    }
}

macro_rules! make_test_solutions {
    (@items [($item:expr, $amount:expr), $($more:tt)*] [$($done:tt)*]) => {
        make_test_solutions!(@items [$($more)*] [($item, $amount), $($done)*])
    };

    (@items [$item:expr, $($more:tt)*] [$($done:tt)*]) => {
        make_test_solutions!(@items [$($more)*] [($item, 1), $($done)*])
    };

    (@items [($item:expr, $amount:expr)] [$($done:tt)*]) => {
        FxHashMap::from_iter([($item, $amount), $($done)*]).into_iter().collect()
    };

    (@items [$item:expr] [$($done:tt)*]) => {
        FxHashMap::from_iter([($item, 1), $($done)*]).into_iter().collect()
    };

    ([$([$($items:tt)*]),* $(,)?]) => {
        sorted_test_solutions(vec![$(make_test_solutions!(@items [$($items)*] [])),*])
    };
}

macro_rules! assert_eq_solutions {
    ($solutions:expr, $expected:expr $(,)?) => {{
        let solutions = $solutions;
        let expected = $expected;

        if solutions != expected {
            panic!(
                "`solutions != expected`\n  solutions: {solutions}\n  expected:  {expected}",
                solutions = display_test_solutions(&solutions),
                expected = display_test_solutions(&expected),
            );
        }
    }};
}

#[test]
fn mock_solutions() {
    test_logger();

    let mut settings = WorldSettings::default();

    let mut item_pool = ItemPoolBuilder::new(&mut Pcg64Mcg::new(0xcafef00dd15ea5e5));
    item_pool.add_amount(health_fragment(), 99);
    item_pool.add_amount(energy_fragment(), 99);
    let item_pool = item_pool.finish();

    macro_rules! test {
        (@graph $requirement:expr) => {{
            let requirement = $requirement;
            eprintln!("testing {requirement}");
            TEST_ASSETS.test_graph(requirement)
        }};

        (@test_solutions $world:expr, [$([$($items:tt)*]),* $(,)?]) => {
            assert_eq_solutions!(
                find_test_solutions(&mut $world, &item_pool, usize::MAX),
                make_test_solutions!([$([$($items)*]),*])
            );
        };

        (spawn with [$($items:tt)*], $requirement:expr, $($more:tt)*) => {
            let graph = test!(@graph $requirement);
            let mut world = mock_world(&graph, &settings, TEST_ASSETS.uber_states.clone());

            for item in [$($items)*] {
                item.simulate(&mut world, &[]);
            }

            test!(@test_solutions world, $($more)*);
        };

        ($requirement:expr, $($more:tt)*) => {{
            let graph = test!(@graph $requirement);
            let mut world = mock_world(&graph, &settings, TEST_ASSETS.uber_states.clone());

            test!(@test_solutions world, $($more)*);
        }};
    }

    test!(Requirement::Impossible, []);

    test!(Requirement::Skill(Shuriken), [[skill(Shuriken)]]);

    test!(
        Requirement::Or(vec![
            Requirement::Skill(Sword),
            Requirement::Skill(Shuriken)
        ]),
        [[skill(Sword)], [skill(Shuriken)]]
    );

    test!(
        Requirement::And(vec![
            Requirement::Skill(Sword),
            Requirement::Skill(Shuriken)
        ]),
        [[skill(Sword), skill(Shuriken)]]
    );

    test!(
        Requirement::EnergySkill(Grenade, 2.),
        [[skill(Grenade), (energy_fragment(), 8)]]
    );

    test!(Requirement::Damage(36.), [[(health_fragment(), 8)]]);

    test!(
        Requirement::And(vec![Requirement::Damage(18.), Requirement::Damage(18.)]),
        [
            [(health_fragment(), 8)],
            [
                (health_fragment(), 4),
                (energy_fragment(), 4),
                skill(Regenerate)
            ]
        ]
    );

    test!(
        Requirement::Or(vec![Requirement::Damage(36.), Requirement::Damage(18.)]),
        [[(health_fragment(), 4)]]
    );

    test!(
        Requirement::BreakWall(12.),
        [
            [skill(Sword)],
            [skill(Hammer)],
            [skill(Bow), (energy_fragment(), 3)],
            [skill(Grenade), (energy_fragment(), 4)],
            [skill(Shuriken), (energy_fragment(), 4)],
            [skill(Blaze), (energy_fragment(), 4)],
            [skill(Spear), (energy_fragment(), 8)],
        ]
    );

    test!(
        Requirement::Combat(smallvec![(Enemy::Slug, 1)]),
        [
            [skill(Sword)],
            [skill(Hammer)],
            [skill(Bow), (energy_fragment(), 4)],
            [skill(Grenade), (energy_fragment(), 4)],
            [skill(Shuriken), (energy_fragment(), 4)],
            [skill(Blaze), (energy_fragment(), 4)],
            [skill(Flash), (energy_fragment(), 8)],
            [skill(Spear), (energy_fragment(), 8)],
        ]
    );

    test!(
        spawn with [skill(Launch)],
        Requirement::Combat(smallvec![
            (Enemy::Skeeto, 2),
            (Enemy::EnergyRefill, 2),
            (Enemy::Mantis, 1),
            (Enemy::SmallSkeeto, 4),
            (Enemy::EnergyRefill, 2),
            (Enemy::Mantis, 1),
            (Enemy::Skeeto, 1)
        ]),
        [
            [skill(Sword)],
            [skill(Hammer)],
            [skill(Bow), (energy_fragment(), 31)],
            [skill(Grenade), (energy_fragment(), 56)],
            [skill(Shuriken), (energy_fragment(), 46)],
            [skill(Blaze), (energy_fragment(), 56)],
            [skill(Flash), (energy_fragment(), 56)],
            [skill(Spear), (energy_fragment(), 80)],
        ]
    );

    settings.difficulty = Difficulty::Unsafe;

    // TODO unsafe level solutions

    // test!(
    //     Requirement::And(vec![
    //         Requirement::Damage(18.),
    //         Requirement::Damage(18.),
    //         Requirement::Damage(18.),
    //     ]),
    //     [
    //         [(health_fragment(), 11)],
    //         [shard(Resilience), (health_fragment(), 10)],
    //         [
    //             skill(Regenerate),
    //             (health_fragment(), 8),
    //             (energy_fragment(), 2)
    //         ],
    //         [
    //             skill(Regenerate),
    //             (health_fragment(), 4),
    //             (energy_fragment(), 4)
    //         ],
    //         [
    //             skill(Regenerate),
    //             shard(Resilience),
    //             (health_fragment(), 7),
    //             (energy_fragment(), 2)
    //         ],
    //         [
    //             skill(Regenerate),
    //             shard(Overcharge),
    //             (health_fragment(), 4),
    //             (energy_fragment(), 3)
    //         ],
    //     ]
    // );

    // test!(
    //     Requirement::BreakWall(12.),
    //     [
    //         [skill(Sword)],
    //         [skill(Hammer)],
    //         [skill(Bow), (energy_fragment(), 2)],
    //         [skill(Bow), shard(Overcharge), energy_fragment()],
    //         [
    //             skill(Bow),
    //             shard(LifePact),
    //             energy_fragment(),
    //             health_fragment()
    //         ],
    //         [skill(Bow), shard(LifePact), (health_fragment(), 2)],
    //         [
    //             skill(Bow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             health_fragment()
    //         ],
    //         [skill(Grenade), (energy_fragment(), 2)],
    //         [skill(Grenade), shard(Overcharge), energy_fragment()],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             energy_fragment(),
    //             (health_fragment(), 2)
    //         ],
    //         [skill(Grenade), shard(LifePact), (health_fragment(), 3)],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 2)
    //         ],
    //         [skill(Shuriken), (energy_fragment(), 2)],
    //         [skill(Shuriken), shard(Overcharge), energy_fragment()],
    //         [
    //             skill(Shuriken),
    //             shard(LifePact),
    //             energy_fragment(),
    //             (health_fragment(), 2)
    //         ],
    //         [skill(Shuriken), shard(LifePact), (health_fragment(), 3)],
    //         [
    //             skill(Shuriken),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 2)
    //         ],
    //         [skill(Blaze), (energy_fragment(), 2)],
    //         [skill(Blaze), shard(Overcharge), energy_fragment()],
    //         [
    //             skill(Blaze),
    //             shard(LifePact),
    //             energy_fragment(),
    //             (health_fragment(), 2)
    //         ],
    //         [skill(Blaze), shard(LifePact), (health_fragment(), 3)],
    //         [
    //             skill(Blaze),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 2)
    //         ],
    //         [skill(Spear), (energy_fragment(), 4)],
    //         [skill(Spear), shard(Overcharge), (energy_fragment(), 2)],
    //         [
    //             skill(Spear),
    //             shard(LifePact),
    //             (energy_fragment(), 3),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Spear),
    //             shard(LifePact),
    //             (energy_fragment(), 2),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             skill(Spear),
    //             shard(LifePact),
    //             energy_fragment(),
    //             (health_fragment(), 4)
    //         ],
    //         [skill(Spear), shard(LifePact), (health_fragment(), 5)],
    //         [
    //             skill(Spear),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             energy_fragment(),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Spear),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 3)
    //         ],
    //         [skill(Sentry), (energy_fragment(), 4)],
    //         [skill(Sentry), shard(Overcharge), (energy_fragment(), 2)],
    //         [
    //             skill(Sentry),
    //             shard(LifePact),
    //             (energy_fragment(), 3),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Sentry),
    //             shard(LifePact),
    //             (energy_fragment(), 2),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             skill(Sentry),
    //             shard(LifePact),
    //             energy_fragment(),
    //             (health_fragment(), 4)
    //         ],
    //         [skill(Sentry), shard(LifePact), (health_fragment(), 5)],
    //         [
    //             skill(Sentry),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             energy_fragment(),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Sentry),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 3)
    //         ],
    //     ]
    // );

    // test!(
    //     spawn with [skill(Bow)],
    //     Requirement::BreakWall(12.),
    //     [
    //        [skill(Sword)],
    //        [skill(Hammer)],
    //         [(energy_fragment(), 2)],
    //         [
    //             shard(Overcharge),
    //             (energy_fragment(), 1)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 1),
    //             (health_fragment(), 1)
    //         ],
    //         [
    //             shard(LifePact),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 1)
    //         ],
    //     ]
    // );

    // test!(
    //     // TODO this should really be equivalent to Requirement::EnergySkill(Grenade, 2.0)
    //     Requirement::And(vec![
    //         Requirement::EnergySkill(Grenade, 1.),
    //         Requirement::EnergySkill(Grenade, 1.)
    //     ]),
    //     [
    //         [skill(Grenade), (energy_fragment(), 4)],
    //         [skill(Grenade), shard(Overcharge), (energy_fragment(), 2)],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             (energy_fragment(), 3),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             (energy_fragment(), 2),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             energy_fragment(),
    //             (health_fragment(), 4)
    //         ],
    //         [skill(Grenade), shard(LifePact), (health_fragment(), 5)],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             energy_fragment(),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             shard(Resilience),
    //             energy_fragment(),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             shard(Resilience),
    //             (health_fragment(), 4)
    //         ],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             shard(Resilience),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Grenade),
    //             shard(LifePact),
    //             skill(Regenerate),
    //             (energy_fragment(), 2)
    //         ],
    //     ]
    // );

    // test!(
    //     spawn with [skill(Bow)],
    //     Requirement::Combat(smallvec![
    //         (Enemy::Hornbug, 1),
    //         (Enemy::Bat, 1),
    //         (Enemy::Sandworm, 2),
    //         (Enemy::Lizard, 2),
    //         (Enemy::Skeeto, 3),
    //         (Enemy::SneezeSlug, 1)
    //     ]),
    //     [
    //         [skill(Sword)],
    //         [skill(Hammer)],
    //         [(energy_fragment(), 32)],
    //         [shard(Overcharge), (energy_fragment(), 16)],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 31),
    //             (health_fragment(), 1)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 30),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 29),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 28),
    //             (health_fragment(), 4)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 27),
    //             (health_fragment(), 5)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 26),
    //             (health_fragment(), 6)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 25),
    //             (health_fragment(), 7)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 24),
    //             (health_fragment(), 8)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 23),
    //             (health_fragment(), 9)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 22),
    //             (health_fragment(), 10)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 21),
    //             (health_fragment(), 11)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 20),
    //             (health_fragment(), 12)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 19),
    //             (health_fragment(), 13)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 18),
    //             (health_fragment(), 14)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 17),
    //             (health_fragment(), 15)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 16),
    //             (health_fragment(), 16)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 15),
    //             (health_fragment(), 17)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 14),
    //             (health_fragment(), 18)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 13),
    //             (health_fragment(), 19)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 12),
    //             (health_fragment(), 20)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 11),
    //             (health_fragment(), 21)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 10),
    //             (health_fragment(), 22)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 9),
    //             (health_fragment(), 23)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 8),
    //             (health_fragment(), 24)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 7),
    //             (health_fragment(), 25)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 6),
    //             (health_fragment(), 26)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 5),
    //             (health_fragment(), 27)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 4),
    //             (health_fragment(), 28)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 3),
    //             (health_fragment(), 29)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 2),
    //             (health_fragment(), 30)
    //         ],
    //         [
    //             shard(LifePact),
    //             (energy_fragment(), 1),
    //             (health_fragment(), 31)
    //         ],
    //         [
    //             shard(LifePact),
    //             (health_fragment(), 32)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 15),
    //             (health_fragment(), 1)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 14),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 13),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 12),
    //             (health_fragment(), 4)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 11),
    //             (health_fragment(), 5)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 10),
    //             (health_fragment(), 6)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 9),
    //             (health_fragment(), 7)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 8),
    //             (health_fragment(), 8)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 7),
    //             (health_fragment(), 9)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 6),
    //             (health_fragment(), 10)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 5),
    //             (health_fragment(), 11)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 4),
    //             (health_fragment(), 12)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 3),
    //             (health_fragment(), 13)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 2),
    //             (health_fragment(), 14)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 1),
    //             (health_fragment(), 15)
    //         ],
    //         [
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 16)
    //         ],
    //         [
    //             skill(Burrow),
    //             (energy_fragment(), 27)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(Overcharge),
    //             (energy_fragment(), 14)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 26),
    //             (health_fragment(), 1)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 25),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 24),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 23),
    //             (health_fragment(), 4)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 22),
    //             (health_fragment(), 5)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 21),
    //             (health_fragment(), 6)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 20),
    //             (health_fragment(), 7)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 19),
    //             (health_fragment(), 8)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 18),
    //             (health_fragment(), 9)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 17),
    //             (health_fragment(), 10)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 16),
    //             (health_fragment(), 11)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 15),
    //             (health_fragment(), 12)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 14),
    //             (health_fragment(), 13)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 13),
    //             (health_fragment(), 14)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 12),
    //             (health_fragment(), 15)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 11),
    //             (health_fragment(), 16)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 10),
    //             (health_fragment(), 17)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 9),
    //             (health_fragment(), 18)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 8),
    //             (health_fragment(), 19)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 7),
    //             (health_fragment(), 20)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 6),
    //             (health_fragment(), 21)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 5),
    //             (health_fragment(), 22)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 4),
    //             (health_fragment(), 23)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 3),
    //             (health_fragment(), 24)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 2),
    //             (health_fragment(), 25)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (energy_fragment(), 1),
    //             (health_fragment(), 26)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             (health_fragment(), 27)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 13),
    //             (health_fragment(), 1)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 12),
    //             (health_fragment(), 2)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 11),
    //             (health_fragment(), 3)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 10),
    //             (health_fragment(), 4)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 9),
    //             (health_fragment(), 5)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 8),
    //             (health_fragment(), 6)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 7),
    //             (health_fragment(), 7)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 6),
    //             (health_fragment(), 8)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 5),
    //             (health_fragment(), 9)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 4),
    //             (health_fragment(), 10)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 3),
    //             (health_fragment(), 11)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 2),
    //             (health_fragment(), 12)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (energy_fragment(), 1),
    //             (health_fragment(), 13)
    //         ],
    //         [
    //             skill(Burrow),
    //             shard(LifePact),
    //             shard(Overcharge),
    //             (health_fragment(), 14)
    //         ],
    //     ]
    // );
}

// useful helpers when writing the expected spawn solutions

// const DPES: [(Skill, f32); 7] = [
//     (Bow, Bow.damage_per_energy(false)),
//     (Shuriken, Shuriken.damage_per_energy(false)),
//     (Blaze, Blaze.damage_per_energy(false)),
//     (Grenade, Grenade.damage_per_energy(false)),
//     (Flash, Flash.damage_per_energy(false)),
//     (Spear, Spear.damage_per_energy(false)),
//     (Sentry, Sentry.damage_per_energy(false)),
// ];

// const fn cost_to_destroy(skill: Skill, health: f32) -> f32 {
//     (health / skill.total_damage(false)).ceil() * 2. * skill.energy_cost()
// }

// const fn costs_to_destroy(health: f32) -> [(Skill, f32); 7] {
//     [
//         (Bow, cost_to_destroy(Bow, health)),
//         (Shuriken, cost_to_destroy(Shuriken, health)),
//         (Blaze, cost_to_destroy(Blaze, health)),
//         (Grenade, cost_to_destroy(Grenade, health)),
//         (Flash, cost_to_destroy(Flash, health)),
//         (Spear, cost_to_destroy(Spear, health)),
//         (Sentry, cost_to_destroy(Sentry, health)),
//     ]
// }

// const fn costs_to_defeat(enemy: Enemy) -> [(Skill, f32); 7] {
//     costs_to_destroy(enemy.health())
// }

static REGIONLESS_GRAPH: LazyLock<Graph> = LazyLock::new(|| {
    let mut areas = Areas::parse(&TEST_ASSETS.base.areas.content)
        .eprint_errors(&TEST_ASSETS.base.areas)
        .unwrap();

    areas
        .contents
        .more
        .retain(|(_, content)| !matches!(content.value, SpannedOption::Some(Content::Region(..))));

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    Graph::compile(
        areas,
        TEST_ASSETS.base.loc_data.clone(),
        TEST_ASSETS.base.state_data.clone(),
        slice::from_ref(&settings),
    )
    .eprint_errors(&TEST_ASSETS.base.areas)
    .unwrap()
});

static ITEM_POOL: LazyLock<ItemPool> = LazyLock::new(|| {
    let mut builder = ItemPoolBuilder::new(&mut Pcg64Mcg::new(0));
    // for simplicity
    builder.remove(&shard(Vitality));
    builder.remove(&shard(Energy));
    builder.finish()
});

#[test]
fn marsh_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "MarshSpawn.Main");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // MarshSpawn.BridgeEX
            [skill(DoubleJump)],
            [skill(Dash)],
            [skill(Glide)],
            [skill(Launch)],
            [skill(Sword)],
            [skill(Hammer)],
            // MarshSpawn.LongSwimEX
            [clean_water()],
            // MarshSpawn.BashEC
            [skill(Bash)],
            // MarshSpawn.ResilienceShard
            [skill(Bow)],
            [skill(Shuriken)],
            [skill(Blaze), (energy_fragment(), 2)],
            [skill(Grenade), (energy_fragment(), 2)],
            [skill(Spear), (energy_fragment(), 2)],
            // TODO should sentry be allowed? [skill(Sentry), (energy_fragment(), 2)],
            // MarshSpawn.RegenTree
            [(keystone(), 2)]
        ]),
    );
}

#[test]
fn den_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "HowlsDen.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // HowlsDen.LaserKS
            [skill(DoubleJump)],
            [skill(Bash), skill(Grenade)],
            [skill(Launch)],
            // HowlsDen.DoubleJumpTree
            [skill(Dash), skill(Glide)],
            [skill(Hammer)],
            // HowlsDen.AboveTPEX
            [skill(Bash), skill(Sword)],
            [skill(Bash), skill(Bow)],
            [skill(Bash), skill(Shuriken)],
            [skill(Bash), skill(Blaze)],
            [skill(Bash), skill(Spear), (energy_fragment(), 2)],
        ]),
    );
}

#[test]
fn hollow_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "EastHollow.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // EastHollow.BashTree via EastHollow.VoiceDoorPlatform
            [skill(Launch)],
            [skill(Bash), skill(DoubleJump)],
            [skill(Bash), skill(Dash)],
            [skill(Bash), skill(Glide)],
            [skill(Bash), skill(Sword)],
            // EastHollow.BashTree via EastHollow.BeetleFight
            [
                skill(Regenerate),
                (health_fragment(), 2),
                skill(Sword),
                skill(DoubleJump)
            ],
            [
                skill(Regenerate),
                (health_fragment(), 2),
                skill(Sword),
                skill(Dash)
            ],
            [
                skill(Regenerate),
                (health_fragment(), 2),
                skill(Hammer),
                skill(DoubleJump)
            ],
            [
                skill(Regenerate),
                (health_fragment(), 2),
                skill(Hammer),
                skill(Dash)
            ],
            // EastHollow.HornBeetleFightEX
            [skill(Bash), skill(Hammer), skill(Grenade)],
        ]),
    );
}

#[test]
fn glades_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "GladesTown.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // GladesTown.LupoSoupEX
            [clean_water()],
            [skill(Burrow)],
            // GladesTown.LowerOre
            [skill(Hammer)],
            [skill(Spear), (energy_fragment(), 2)],
            // GladesTown.UpdraftCeilingEX
            [skill(Launch)],
            [skill(Flap), skill(Glide)],
            [skill(DoubleJump), shard(TripleJump)],
            // anchor WestGlades.RightLowerPool
            // [skill(DoubleJump), skill(Bash)],
            // [skill(Bash), skill(Grenade), skill(Dash)],
            // [skill(Bash), skill(Grenade), skill(Glide)],
            // [skill(Bash), skill(Grenade), skill(Sword)],
            // [skill(DoubleJump), skill(Dash), skill(Sword)],
            // [skill(DoubleJump), skill(Dash), skill(Glide), skill(Bow)],
            // [skill(DoubleJump), skill(Dash), skill(Glide), skill(Shuriken), (energy_fragment(), 2)],
            // [skill(DoubleJump), skill(Dash), skill(Glide), skill(Blaze), (energy_fragment(), 2)],
            // [skill(DoubleJump), skill(Dash), skill(Glide), skill(Grenade), (energy_fragment(), 2)],
            // [skill(DoubleJump), skill(Dash), skill(Glide), skill(Flash), (energy_fragment(), 2)],
            // WestGlades.AbovePlantEX
            [skill(DoubleJump), skill(Bash)],
            [skill(Bash), skill(Grenade), skill(Dash)],
            [
                skill(Bash),
                skill(Grenade),
                skill(Glide),
                (energy_fragment(), 2)
            ],
            [
                skill(Bash),
                skill(Grenade),
                skill(Sword),
                (energy_fragment(), 2)
            ],
            [skill(Bash), skill(Grenade), skill(Glide), skill(Bow)],
            [skill(Bash), skill(Grenade), skill(Glide), skill(WaterDash)],
            [skill(Bash), skill(Grenade), skill(Sword), skill(WaterDash)],
            // WestGlades.GrappleEX
            [skill(Grapple), skill(Bash), skill(Grenade), skill(Glide)],
            [skill(Grapple), skill(Bash), skill(Grenade), skill(Sword)],
            [skill(Grapple), skill(DoubleJump), skill(Dash), skill(Sword)],
            [
                skill(Grapple),
                skill(DoubleJump),
                skill(Dash),
                skill(Glide),
                skill(Bow)
            ],
            [
                skill(Grapple),
                skill(DoubleJump),
                skill(Dash),
                skill(Glide),
                skill(Shuriken),
                (energy_fragment(), 2)
            ],
            [
                skill(Grapple),
                skill(DoubleJump),
                skill(Dash),
                skill(Glide),
                skill(Blaze),
                (energy_fragment(), 2)
            ],
            [
                skill(Grapple),
                skill(DoubleJump),
                skill(Dash),
                skill(Glide),
                skill(Grenade),
                (energy_fragment(), 2)
            ],
            [
                skill(Grapple),
                skill(DoubleJump),
                skill(Dash),
                skill(Glide),
                skill(Flash),
                (energy_fragment(), 2)
            ],
        ]),
    );
}

#[test]
fn wellspring_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "InnerWellspring.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // InnerWellspring.SwimOre
            [clean_water()],
            // OuterWellspring.RightWallOre
            [skill(DoubleJump)],
            [skill(Dash)],
            [skill(Glide)],
            [skill(Sword)],
            [skill(Hammer)],
            [skill(Grapple)],
            [skill(Launch)],
            [skill(Bash), skill(Grenade)],
        ]),
    );
}

#[test]
fn woods_entrance_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "WoodsEntry.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // WoodsEntry.LeafPileEX
            [skill(Flap)],
            // WoodsEntry.MudPitEX
            [skill(DoubleJump)],
            [skill(Dash)],
            [skill(Glide)],
            [skill(Launch)],
            [skill(Bash), skill(Grenade)],
            // EastHollow.ForestsVoice
            [clean_water(), skill(Bash)],
            // WoodsEntry.LowerKS
            [skill(Sword), skill(Bow)],
            [skill(Sword), skill(Shuriken)],
            [skill(Sword), skill(Grenade), (energy_fragment(), 2)],
            // [skill(Sword), skill(Sentry), (energy_fragment(), 2)],
            [skill(Hammer), skill(Bow)],
            [skill(Hammer), skill(Shuriken)],
            [skill(Hammer), skill(Grenade), (energy_fragment(), 2)],
            // [skill(Hammer), skill(Sentry), (energy_fragment(), 2)],
        ]),
    );
}

#[test]
fn woods_exit_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "WoodsMain.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // WoodsMain.ShrineEX
            [skill(Glide)],
            [skill(Launch)],
            // WoodsMain.LogBlobDestroyed + Combat=2xBalloon
            // [skill(Sword), skill(Bow)],
            // [skill(Sword), skill(Shuriken)],
            // [skill(Hammer), skill(Bow)],
            // [skill(Hammer), skill(Shuriken)],
            // [skill(Grenade), skill(Bow)],
            // [skill(Grenade), skill(Shuriken)],
            // [skill(Grenade), (energy_fragment(), 2)],
            // [skill(Blaze), skill(Bow)],
            // [skill(Blaze), skill(Shuriken)],
            // [skill(Sentry), skill(Bow)],
            // [skill(Sentry), skill(Shuriken)],
            // WoodsMain.HiddenEX
            [skill(DoubleJump), skill(Sword), skill(Bow)],
            [skill(DoubleJump), skill(Sword), skill(Shuriken)],
            [skill(DoubleJump), skill(Hammer), skill(Bow)],
            [skill(DoubleJump), skill(Hammer), skill(Shuriken)],
            [
                skill(DoubleJump),
                skill(Sword),
                skill(Grenade),
                (energy_fragment(), 2)
            ],
            [
                skill(DoubleJump),
                skill(Hammer),
                skill(Grenade),
                (energy_fragment(), 2)
            ],
            [
                skill(DoubleJump),
                shard(TripleJump),
                skill(Grenade),
                skill(Bow)
            ],
            [
                skill(DoubleJump),
                shard(TripleJump),
                skill(Grenade),
                skill(Shuriken)
            ],
            [
                skill(DoubleJump),
                shard(TripleJump),
                skill(Grenade),
                (energy_fragment(), 2)
            ],
            [
                skill(DoubleJump),
                shard(TripleJump),
                skill(Blaze),
                skill(Bow)
            ],
            [
                skill(DoubleJump),
                shard(TripleJump),
                skill(Blaze),
                skill(Shuriken)
            ],
            [
                skill(DoubleJump),
                shard(TripleJump),
                skill(Sentry),
                skill(Bow)
            ],
            [
                skill(DoubleJump),
                shard(TripleJump),
                skill(Sentry),
                skill(Shuriken)
            ],
            [skill(Bash), skill(Grenade), skill(Dash)],
            [skill(Bash), skill(Grenade), skill(Bow)],
            [
                skill(Bash),
                skill(Grenade),
                skill(Shuriken),
                (energy_fragment(), 2)
            ],
        ]),
    );
}

#[test]
fn reach_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "LowerReach.Teleporter");
    world.store_skill(Grenade, true, &[]);
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // TODO this is incorrect in areas.wotw, the ice wall next to the teleporter should be a state
            // if this was correct, LowerReach.MeltIceEX would be reachable on spawn and not considered here
            // some solution below are commented out because they become redundant as a consequence
            // LowerReach.MeltIceEX
            [(energy_fragment(), 2)],
            [skill(Sword)],
            [skill(Bow)],
            [skill(Shuriken)],
            // LowerReach.AboveDoorEX
            [skill(Bash)],
            [skill(Hammer)],
            // [skill(Spear), (energy_fragment(), 2)],
            [skill(Launch)],
            // LowerReach.BurrowEX
            [skill(Burrow)],
            // LowerReach.TPLeftEX
            [skill(Glide)],
            [skill(DoubleJump), shard(TripleJump)],
            [skill(DoubleJump), skill(Dash)],
            // [skill(DoubleJump), skill(Sword)],
            // [skill(Dash), skill(Sword)],
        ]),
    );
}

#[test]
fn depths_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "UpperDepths.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // TODO these three are invalid solutions, we don't have `BreakWall with` syntax yet so this cannot be written correctly in areas.wotw
            [skill(Spear), skill(Hammer)],
            [skill(Spear), skill(Sword)],
            [skill(Spear), skill(Blaze)],
            // UpperDepths.RightHealthKS
            [skill(Bow)],
            [skill(Shuriken)],
            [skill(Grenade)],
            [skill(Spear), (energy_fragment(), 2)],
            [skill(Flash), (energy_fragment(), 2)],
            [skill(Flash), skill(DoubleJump), skill(Sword)],
            [skill(Flash), skill(DoubleJump), skill(Hammer)],
            [skill(Flash), skill(Dash), health_fragment()],
            [skill(Launch), skill(Sword)],
            [skill(Launch), skill(Hammer)],
            [skill(Launch), skill(Blaze)],
            // UpperDepths.HiveEX
            [skill(Flash), skill(Bash)],
            [skill(Flash), skill(Launch)],
            [skill(Launch), skill(Dash)],
            [skill(Dash), health_fragment(), skill(Bash)],
            [
                skill(Dash),
                health_fragment(),
                skill(Grapple),
                skill(DoubleJump),
                skill(Sword)
            ],
            [
                skill(Dash),
                health_fragment(),
                skill(Grapple),
                skill(DoubleJump),
                skill(Hammer)
            ],
            [
                skill(Dash),
                health_fragment(),
                skill(Grapple),
                skill(DoubleJump),
                skill(Blaze),
                (energy_fragment(), 2)
            ],
        ]),
    );
}

#[test]
fn pools_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "EastPools.Teleporter");
    world.store_clean_water(true, &[]);
    world.store_boolean(Teleporter::CENTRAL_POOLS_ID, true, &[]);
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // EastPools.UltraBashShard
            [skill(Bash)],
            // EastPools.AboveTpEX
            [skill(WaterDash), skill(Grapple), skill(DoubleJump)],
            [skill(WaterDash), skill(Grapple), skill(Dash), skill(Sword)],
            [
                skill(WaterDash),
                skill(DoubleJump),
                shard(TripleJump),
                skill(Dash)
            ],
            [skill(Launch)],
        ]),
    );
}

#[test]
fn feeding_grounds_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "LowerWastes.WestTP");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // LowerWastes.WestTPOre
            [skill(Burrow)],
            // LowerWastes.SunsetViewEX
            [skill(DoubleJump), shard(TripleJump)],
            [skill(DoubleJump), skill(Grapple)],
            [skill(Bash), skill(Grenade), skill(DoubleJump)],
            [skill(Bash), skill(Grenade), skill(Dash)],
            [skill(Bash), skill(Grenade), health_fragment()],
            [skill(Grapple), skill(Dash)],
            [skill(Grapple), skill(Sword), health_fragment()],
            [skill(Launch)],
        ]),
    );
}

#[test]
fn central_wastes_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "LowerWastes.EastTP");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // LowerWastes.UpperPathHC
            [skill(Burrow)],
            // LowerWastes.EastTPOre
            [skill(DoubleJump), skill(Sword)],
            [skill(DoubleJump), skill(Hammer)],
            [skill(DoubleJump), skill(Bow)],
            [skill(DoubleJump), skill(Shuriken)],
            [skill(DoubleJump), skill(Blaze), (energy_fragment(), 2)],
            [skill(DoubleJump), skill(Grenade), (energy_fragment(), 2)],
            [skill(DoubleJump), skill(Spear), (energy_fragment(), 2)],
            [skill(Bash), skill(Grenade), skill(Sword)],
            [skill(Bash), skill(Grenade), skill(Hammer)],
            [skill(Launch), skill(Sword)],
            [skill(Launch), skill(Hammer)],
            [skill(Launch), skill(Bow)],
            [skill(Launch), skill(Shuriken)],
            [skill(Launch), skill(Blaze), (energy_fragment(), 2)],
            [skill(Launch), skill(Grenade), (energy_fragment(), 2)],
            [skill(Launch), skill(Spear), (energy_fragment(), 2)],
        ]),
    );
}

#[test]
fn outer_ruins_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "UpperWastes.NorthTP");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // UpperWastes.SpinLasersRightEX
            [skill(Burrow)],
            // UpperWastes.FlowersSeed
            [
                skill(Bash),
                skill(Grenade),
                skill(DoubleJump),
                shard(TripleJump),
                skill(Dash)
            ],
            [
                skill(Bash),
                skill(Grenade),
                skill(DoubleJump),
                shard(TripleJump),
                skill(Glide)
            ],
            [
                skill(Bash),
                skill(Grenade),
                skill(DoubleJump),
                shard(TripleJump),
                skill(Sword)
            ],
            [
                skill(Bash),
                skill(Grenade),
                skill(DoubleJump),
                shard(TripleJump),
                health_fragment()
            ],
            [
                skill(Bash),
                skill(Grenade),
                (energy_fragment(), 2),
                skill(DoubleJump),
                shard(TripleJump)
            ],
            [skill(Launch), skill(DoubleJump), skill(Sword)],
            [skill(Launch), skill(DoubleJump), skill(Hammer)],
            [skill(Launch), skill(DoubleJump), shard(TripleJump)],
            [skill(Launch), skill(DoubleJump), skill(Dash)],
            [skill(Launch), skill(Bash)],
        ]),
    );
}

// TODO outdated since the paths were analyzed when logic was rather incomplete around WillowsEnd.EntryEX, reanalyze
// #[test]
// fn willow_spawn_solutions() {
//     test_logger();

//     let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
//     let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "WillowsEnd.InnerTP");
//     world.traverse_spawn(&[]);

//     assert_eq_solutions!(
//         find_test_solutions(&mut world, &*ITEM_POOL, 7),
//         make_test_solutions!([
//             // anchor WillowsEnd.Entry
//             // [skill(Sword)],
//             // [skill(Hammer)],
//             // [skill(Bow)],
//             // [skill(Shuriken)],
//             // [skill(Blaze), (energy_fragment(), 2)],
//             // [skill(Grenade), (energy_fragment(), 2)],
//             // [skill(Spear), (energy_fragment(), 2)],
//             // [skill(DoubleJump), skill(Dash)],
//             // [skill(DoubleJump), shard(TripleJump), (health_fragment(), 3)],
//             // [skill(Glide)],
//             // [skill(Bash), (health_fragment(), 3), skill(DoubleJump)],
//             // [skill(Bash), (health_fragment(), 3), skill(Dash)],
//             // [skill(Launch)],
//             // WillowsEnd.EntryEX
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 (energy_fragment(), 2),
//                 skill(DoubleJump)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 (energy_fragment(), 2),
//                 skill(Dash)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 (energy_fragment(), 2),
//                 skill(Sword)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 (energy_fragment(), 2),
//                 skill(Hammer)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 skill(Sword)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 skill(Hammer)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 skill(Bow)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 skill(Shuriken)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 skill(Dash)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 skill(Glide)
//             ],
//             [
//                 skill(Bash),
//                 skill(Grenade),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 (health_fragment(), 3)
//             ],
//             [skill(Launch)],
//             // WillowsEnd.SpikesOre
//             [
//                 skill(Grapple),
//                 skill(DoubleJump),
//                 shard(TripleJump),
//                 skill(Glide)
//             ],
//         ]),
//     );
// }

#[test]
fn burrows_spawn_solutions() {
    test_logger();

    let settings = WorldSettings::difficulty_default(Difficulty::Gorlek);
    let mut world = test_world(&*REGIONLESS_GRAPH, &settings, "MidnightBurrows.Teleporter");
    world.traverse_spawn(&[]);

    assert_eq_solutions!(
        find_test_solutions(&mut world, &*ITEM_POOL, 7),
        make_test_solutions!([
            // MidnightBurrows.LeftKS
            [skill(DoubleJump)],
            [skill(Dash)],
            [skill(Glide)],
            [skill(Sword)],
            [skill(Hammer)],
            // TODO this doesn't get found because it provides better orbs for a
            // to a connection which has already been solved...
            // [skill(Bash)],
            [skill(Launch)],
            [health_fragment()],
            [skill(Regenerate)],
        ]),
    );
}
