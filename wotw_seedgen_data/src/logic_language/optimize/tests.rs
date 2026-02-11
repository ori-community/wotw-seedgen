use log::trace;

use crate::{
    assets::TEST_ASSETS, logic_language::output::Requirement::*, test_logger, Shard::*, Skill::*,
};

macro_rules! test {
    ($before:expr, $after:expr) => {
        let mut graph = TEST_ASSETS.test_graph($before);
        graph.optimize();

        trace!("optimized graph: {graph:#?}");

        assert_eq!(graph.get_requirement(), &$after);
    };
}

#[test]
fn optimize_graph_region() {
    let _ = *TEST_ASSETS;
    test_logger();

    test!(
        Or(vec![
            Skill(Regenerate),
            And(vec![Skill(Regenerate), Danger(40.)]),
        ]),
        Skill(Regenerate)
    );
}

#[test]
fn optimize_graph_bashnade() {
    let _ = *TEST_ASSETS;
    test_logger();

    test!(
        Or(vec![
            And(vec![
                Skill(Bash),
                Or(vec![
                    EnergySkill(Grenade, 2.),
                    And(vec![EnergySkill(Grenade, 1.), Skill(DoubleJump)])
                ])
            ]),
            And(vec![Skill(Bash), EnergySkill(Grenade, 1.), Skill(Dash)])
        ]),
        And(vec![
            Skill(Bash),
            EnergySkill(Grenade, 1.),
            Or(vec![
                EnergySkill(Grenade, 1.),
                Skill(DoubleJump),
                Skill(Dash),
            ])
        ])
    );
}

// EastPools.TPArea -> EastPools.AboveDoorOre
#[test]
fn optimize_graph_above_door_ore() {
    let _ = *TEST_ASSETS;
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
        And(vec![
            Skill(Bash),
            EnergySkill(Grenade, 1.),
            Or(vec![
                Skill(Dash),
                Skill(Glide),
                Damage(20.),
                And(vec![
                    EnergySkill(Grenade, 1.),
                    Or(vec![Skill(Sword), Skill(Hammer)])
                ])
            ])
        ])
    );
}

// EastPools.TPArea -> EastPools.FishingPool
#[test]
fn optimize_graph_fishing_pool() {
    let _ = *TEST_ASSETS;
    test_logger();

    test!(
        And(vec![Or(vec![
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
        ]),]),
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
