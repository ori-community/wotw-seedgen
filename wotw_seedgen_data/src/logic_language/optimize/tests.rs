use smallvec::smallvec;
use wotw_seedgen_log_capture::NO_LOG_CAPTURE;

use crate::{
    assets::TEST_ASSETS,
    logic_language::output::{Enemy::*, Requirement::*},
    test_logger,
    Shard::*,
    Skill::*,
};

macro_rules! test {
    ($before:expr, $after:expr) => {
        let before = $before;

        let mut after = $after;
        // Order is too arbitrary and tedious, don't want to test it here
        after.improve_order();

        let mut graph = TEST_ASSETS.test_graph(before.clone());
        graph.optimize(&NO_LOG_CAPTURE);
        let optimized = graph.get_requirement();

        assert_eq!(
            optimized, &after,
            "{before} should've optimized to {after}, but optimized to {optimized} instead"
        );
    };
}

#[test]
fn optimize_graph_region() {
    test_logger();

    test!(
        Or(vec![
            Skill(Regenerate),
            And(vec![Skill(Regenerate), Danger(40.)]),
        ]),
        Skill(Regenerate)
    );
}

// TODO enable when pulling apart energy uses again
// #[test]
// fn optimize_graph_bashnade() {
//     test_logger();
//
//     test!(
//         Or(vec![
//             And(vec![
//                 Skill(Bash),
//                 Or(vec![
//                     EnergySkill(Grenade, (2.).into()),
//                     And(vec![EnergySkill(Grenade, (1.).into()), Skill(DoubleJump)])
//                 ])
//             ]),
//             And(vec![
//                 Skill(Bash),
//                 EnergySkill(Grenade, (1.).into()),
//                 Skill(Dash)
//             ])
//         ]),
//         And(vec![
//             Skill(Bash),
//             EnergySkill(Grenade, (1.).into()),
//             Or(vec![
//                 EnergySkill(Grenade, (1.).into()),
//                 Skill(DoubleJump),
//                 Skill(Dash),
//             ])
//         ])
//     );
// }

#[test]
fn optimize_graph_nested_redundancy() {
    test_logger();

    test!(
        Or(vec![
            And(vec![Skill(Dash), Or(vec![Skill(DoubleJump), Skill(Bash)])]),
            Skill(Bash)
        ]),
        Or(vec![Skill(Bash), And(vec![Skill(DoubleJump), Skill(Dash)])])
    );
}

// EastPools.TPArea -> EastPools.AboveDoorOre
#[test]
fn optimize_graph_above_door_ore() {
    test_logger();

    test!(
        Or(vec![
            And(vec![
                Skill(Bash),
                EnergySkill(Grenade, 1.0),
                Or(vec![Skill(Dash), Skill(Glide)]),
            ]),
            And(vec![Skill(Bash), EnergySkill(Grenade, 1.0), Damage(20.0)]),
            And(vec![
                Skill(Bash),
                EnergySkill(Grenade, 2.0),
                Or(vec![Skill(Sword), Skill(Hammer)]),
            ]),
        ]),
        // this is when pulling apart the grenade
        // And(vec![
        //     Skill(Bash),
        //     EnergySkill(Grenade, (1.).into()),
        //     Or(vec![
        //         Skill(Dash),
        //         Skill(Glide),
        //         Damage((20.).into()),
        //         And(vec![
        //             Or(vec![Skill(Sword), Skill(Hammer)]),
        //             EnergySkill(Grenade, (1.).into()),
        //         ])
        //     ])
        // ])
        And(vec![
            Skill(Bash),
            Or(vec![
                And(vec![
                    EnergySkill(Grenade, 1.),
                    Or(vec![Skill(Dash), Skill(Glide), Damage(20.)]),
                ]),
                And(vec![
                    EnergySkill(Grenade, 2.),
                    Or(vec![Skill(Sword), Skill(Hammer)]),
                ]),
            ]),
        ])
    );
}

// EastPools.TPArea -> EastPools.FishingPool
#[test]
fn optimize_graph_fishing_pool() {
    test_logger();

    test!(
        Or(vec![
            And(vec![
                Skill(Grapple),
                Or(vec![Skill(DoubleJump), Skill(Dash)]),
            ]),
            And(vec![
                Skill(DoubleJump),
                Skill(Bash),
                EnergySkill(Grenade, 1.0),
            ]),
            Skill(Launch),
            And(vec![
                Skill(Grapple),
                Or(vec![Skill(Glide), Skill(Sword), Skill(Hammer)]),
            ]),
            And(vec![
                Skill(Bash),
                EnergySkill(Grenade, 1.0),
                Or(vec![Skill(Dash), Skill(Sword), Skill(Hammer)]),
            ]),
            And(vec![Skill(DoubleJump), Shard(TripleJump), Skill(Dash)]),
        ]),
        Or(vec![
            And(vec![
                Skill(Grapple),
                Or(vec![
                    Skill(DoubleJump),
                    Skill(Dash),
                    Skill(Glide),
                    Skill(Sword),
                    Skill(Hammer),
                ])
            ]),
            And(vec![
                Skill(Bash),
                EnergySkill(Grenade, 1.),
                Or(vec![
                    Skill(DoubleJump),
                    Skill(Dash),
                    Skill(Sword),
                    Skill(Hammer),
                ])
            ]),
            Skill(Launch),
            And(vec![Skill(DoubleJump), Shard(TripleJump), Skill(Dash),]),
        ])
    );
}

// MarshSpawn.BurrowFightArena -> MarshSpawn.BurrowArena
#[test]
fn optimize_graph_burrow_arena() {
    test_logger();

    let combat = Combat(smallvec![
        (Hornbug, 1),
        (Bat, 1),
        (Sandworm, 2),
        (Lizard, 2),
        (Skeeto, 3),
        (SneezeSlug, 1),
    ]);

    test!(
        Or(vec![
            And(vec![Skill(Regenerate), Damage(40.), combat.clone()]),
            combat.clone()
        ]),
        combat
    );
}

// UpperDepths.RightKeystonePath -> UpperDepths.RightEntryKS
#[test]
fn optimize_graph_depths_right_entry_ks() {
    test_logger();

    let depths_light = Or(vec![
        State(0),
        EnergySkill(Flash, 1.),
        EnergySkill(Bow, 3.),
        NonConsumingEnergySkill(Bow),
    ]);
    let depths_light_optimized = Or(vec![
        State(0),
        EnergySkill(Flash, 1.),
        NonConsumingEnergySkill(Bow),
    ]);

    test!(
        Or(vec![
            And(vec![Skill(Grapple), Skill(DoubleJump)]),
            Skill(Launch),
            Skill(Grapple),
            And(vec![Skill(DoubleJump), Shard(TripleJump)]),
            And(vec![
                depths_light.clone(),
                Or(vec![
                    Skill(DoubleJump),
                    Skill(Dash),
                    Skill(Glide),
                    Skill(Sword),
                ]),
            ]),
            And(vec![
                depths_light.clone(),
                Skill(Bash),
                EnergySkill(Grenade, 1.),
            ]),
            Skill(Bash),
            And(vec![
                depths_light.clone(),
                Or(vec![
                    Skill(Hammer),
                    EnergySkill(Shuriken, 2.),
                    EnergySkill(Sentry, 2.),
                ]),
            ]),
        ]),
        Or(vec![
            Skill(Launch),
            Skill(Grapple),
            Skill(Bash),
            And(vec![Skill(DoubleJump), Shard(TripleJump)]),
            And(vec![
                depths_light_optimized.clone(),
                Or(vec![
                    Skill(DoubleJump),
                    Skill(Dash),
                    Skill(Glide),
                    Skill(Sword),
                    Skill(Hammer),
                    EnergySkill(Shuriken, 2.),
                    EnergySkill(Sentry, 2.),
                ]),
            ]),
        ])
    );
}

// PoolsApproach.OnTopOfWheel -> PoolsApproach.MillPath
#[test]
fn optimize_graph_mill_path() {
    test_logger();

    test!(
        Or(vec![
            And(vec![
                State(495),
                Or(vec![
                    And(vec![
                        Water,
                        Skill(Bash),
                        Damage(10.),
                        Skill(Grapple),
                        Skill(DoubleJump),
                        Or(vec![Skill(Dash), Skill(Glide)]),
                    ]),
                    And(vec![Water, Skill(Bash), Skill(WaterDash)]),
                    And(vec![Water, Skill(Bash), Damage(10.), Skill(Launch)]),
                ]),
            ]),
            And(vec![
                State(495),
                Or(vec![
                    And(vec![Water, Skill(Bash)]),
                    And(vec![Water, Damage(10.), Skill(Launch)]),
                    And(vec![
                        Water,
                        Skill(WaterDash),
                        Or(vec![
                            Skill(Launch),
                            Skill(DoubleJump),
                            Combat(smallvec![(SpinCrab, 1)]),
                        ]),
                    ]),
                ]),
            ])
        ]),
        And(vec![
            State(495),
            Water,
            Or(vec![
                Skill(Bash),
                And(vec![Skill(Launch), Damage(10.)]),
                And(vec![
                    Skill(WaterDash),
                    Or(vec![
                        Skill(Launch),
                        Skill(DoubleJump),
                        Combat(smallvec![(SpinCrab, 1)]),
                    ])
                ]),
            ]),
        ])
    );
}
