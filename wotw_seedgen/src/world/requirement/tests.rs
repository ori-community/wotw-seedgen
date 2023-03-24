use super::*;
use crate::settings::WorldSettings;

use rustc_hash::FxHashSet;
use smallvec::smallvec;

use crate::inventory::Inventory;
use crate::item::Item;
use crate::util::orbs::{OrbVariants, Orbs};
use crate::world::player::Player;

#[test]
fn is_met() {
    macro_rules! test {
        ($player:expr, $states:expr, $req:expr, [...]) => {
            assert!(!$req.is_met($player, $states, smallvec![$player.max_orbs()]).is_empty());
        };
        ($player:expr, $states:expr, $req:expr, [$player_orbs:expr], [...]) => {
            assert!(!$req.is_met($player, $states, smallvec![$player_orbs]).is_empty());
        };
        ($player:expr, $states:expr, $req:expr, [$player_orbs:expr], [$($orbs:expr),* $(,)?]) => {
            {
                let sort = |mut orbs: OrbVariants| { orbs.sort_unstable_by(|a, b| a.health.partial_cmp(&b.health).expect("non-real orb value")); orbs };
                assert_eq!(sort($req.is_met($player, $states, smallvec![$player_orbs])), sort(smallvec![$($player_orbs + $orbs),*]));
            }
        };
        ($player:expr, $states:expr, $req:expr, [$($orbs:tt)*]) => {
            test!($player, $states, $req, [$player.max_orbs()], [$($orbs)*]);
        };
    }

    let world_settings = WorldSettings::default();
    let mut player = Player::new(&world_settings);

    let mut states = FxHashSet::default();
    let orbs = Orbs::default();

    test!(&player, &states, Requirement::Skill(Skill::Blaze), []);
    player.inventory.grant(Item::Skill(Skill::Blaze), 1);
    test!(&player, &states, Requirement::Skill(Skill::Blaze), [...]);

    test!(&player, &states, Requirement::And(vec![Requirement::Skill(Skill::Blaze), Requirement::Free]), [...]);
    test!(&player, &states, Requirement::Or(vec![Requirement::Skill(Skill::Blaze), Requirement::Impossible]), [...]);

    test!(
        &player,
        &states,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        []
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 2);
    test!(
        &player,
        &states,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        []
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs {
            energy: -1.0,
            ..orbs
        }]
    );
    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 2);
    test!(
        &player,
        &states,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player = Player::new(&world_settings);
    player.inventory.grant(Item::Skill(Skill::Blaze), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 3);
    player.inventory.grant(Item::Shard(Shard::LifePact), 1);
    test!(
        &player,
        &states,
        Requirement::EnergySkill(Skill::Blaze, 1.0),
        [Orbs {
            energy: -0.5,
            health: -5.0
        }]
    );
    test!(
        &player,
        &states,
        Requirement::NonConsumingEnergySkill(Skill::Blaze),
        [Orbs {
            health: -5.0,
            ..orbs
        }]
    );
    test!(
        &player,
        &states,
        Requirement::NonConsumingEnergySkill(Skill::Blaze),
        [Orbs {
            energy: 0.0,
            health: player.max_health()
        }],
        [Orbs {
            energy: 0.5,
            health: -10.0
        }]
    );

    test!(&player, &states, Requirement::State(34), []);
    states.insert(34);
    test!(&player, &states, Requirement::State(34), [...]);
    test!(&player, &states, Requirement::State(33), []);

    player = Player::new(&world_settings);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 4);
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 6);
    test!(&player, &states, Requirement::Damage(30.0), []);
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 1);
    test!(
        &player,
        &states,
        Requirement::Damage(30.0),
        [Orbs {
            health: -30.0,
            ..orbs
        }]
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 2);
    player.inventory.grant(Item::Skill(Skill::Regenerate), 1);
    test!(&player, &states, Requirement::Damage(60.0), []);
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 6);
    test!(
        &player,
        &states,
        Requirement::Damage(60.0),
        [Orbs {
            health: 30.0,
            energy: player.max_energy()
        }],
        [Orbs {
            health: -25.0,
            energy: -2.0
        }]
    );
    test!(
        &player,
        &states,
        Requirement::Danger(30.0),
        [Orbs {
            health: 30.0,
            energy: player.max_energy()
        }],
        [Orbs {
            health: 30.0,
            energy: -1.0
        }]
    );
    test!(
        &player,
        &states,
        Requirement::Danger(60.0),
        [Orbs {
            health: 30.0,
            energy: player.max_energy()
        }],
        [Orbs {
            health: 35.0,
            energy: -2.0
        }]
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player = Player::new(&world_settings);
    test!(&player, &states, Requirement::BreakWall(12.0), []);
    player.inventory.grant(Item::Skill(Skill::Sword), 1);
    test!(
        &player,
        &states,
        Requirement::BreakWall(12.0),
        [player.max_orbs()]
    );
    player = Player::new(&world_settings);
    player.inventory.grant(Item::Skill(Skill::Grenade), 1);
    test!(&player, &states, Requirement::BreakWall(12.0), []);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 3);
    test!(&player, &states, Requirement::BreakWall(12.0), []);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 1);
    test!(
        &player,
        &states,
        Requirement::BreakWall(12.0),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );
    player = Player::new(&world_settings);
    player.inventory.grant(Item::Skill(Skill::Grenade), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 2);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::BreakWall(16.0),
        [Orbs {
            energy: -1.0,
            ..orbs
        }]
    );
    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 1);
    test!(&player, &states, Requirement::BreakWall(12.0), []);

    player = Player::new(&world_settings);
    player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(&player, &states, Requirement::ShurikenBreak(12.0), []);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 4);
    test!(
        &player,
        &states,
        Requirement::ShurikenBreak(12.0),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 6);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(&player, &states, Requirement::ShurikenBreak(12.0), []);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 2);
    test!(
        &player,
        &states,
        Requirement::ShurikenBreak(12.0),
        [Orbs {
            energy: -6.0,
            ..orbs
        }]
    );

    player = Player::new(&world_settings);
    player.inventory.grant(Item::Skill(Skill::Bow), 1);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        []
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 7);
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        [Orbs {
            energy: -3.25,
            ..orbs
        }]
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 6);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        []
    );
    player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Slug, 2), (Enemy::Skeeto, 1)]),
        [Orbs {
            energy: -6.5,
            ..orbs
        }]
    );
    player = Player::new(&world_settings);
    let req = Requirement::Combat(smallvec![
        (Enemy::Sandworm, 1),
        (Enemy::Bat, 1),
        (Enemy::EnergyRefill, 99),
        (Enemy::ShieldMiner, 2),
        (Enemy::EnergyRefill, 1),
        (Enemy::Balloon, 4)
    ]);
    player.inventory.grant(Item::Skill(Skill::Shuriken), 1);
    player.inventory.grant(Item::Skill(Skill::Spear), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 27);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(&player, &states, &req, []);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 1);
    test!(
        &player,
        &states,
        &req,
        [Orbs {
            energy: -14.0,
            ..orbs
        }]
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 37);
    player.inventory.grant(Item::Skill(Skill::Bash), 1);
    player.inventory.grant(Item::Skill(Skill::Launch), 1);
    player.inventory.grant(Item::Skill(Skill::Burrow), 1);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(&player, &states, &req, []);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 1);
    test!(
        &player,
        &states,
        &req,
        [Orbs {
            energy: -33.0,
            ..orbs
        }]
    );
    player = Player::new(&world_settings);
    player.inventory.grant(Item::Skill(Skill::Spear), 1);
    player.inventory.grant(Item::Skill(Skill::DoubleJump), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 4);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Gorlek,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        [Orbs {
            energy: -2.0,
            ..orbs
        }]
    );
    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        []
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 11);
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        []
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 1);
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Tentacle, 1)]),
        [Orbs {
            energy: -8.0,
            ..orbs
        }]
    );

    player = Player::new(&world_settings);
    let a = Requirement::EnergySkill(Skill::Blaze, 2.0);
    let b = Requirement::Damage(20.0);
    let c = Requirement::EnergySkill(Skill::Blaze, 1.0);
    let d = Requirement::Damage(10.0);
    player.inventory.grant(Item::Skill(Skill::Blaze), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 4);
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 5);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::And(vec![c.clone(), d.clone()]),
        [Orbs {
            health: -10.0,
            energy: -1.0
        }]
    );
    test!(
        &player,
        &states,
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
        &player,
        &states,
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
        &player,
        &states,
        Requirement::And(vec![
            Requirement::Or(vec![a.clone(), d.clone()]),
            Requirement::Or(vec![b.clone(), c.clone()])
        ]),
        [Orbs {
            energy: -1.0,
            health: -10.0
        }]
    );
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 8);
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 8);
    test!(
        &player,
        &states,
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
        &player,
        &states,
        Requirement::Or(vec![Requirement::Free, b.clone()]),
        [Orbs::default()]
    );
    test!(
        &player,
        &states,
        Requirement::Or(vec![b.clone(), Requirement::Free]),
        [Orbs::default()]
    );

    player = Player::new(&world_settings);
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 7);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 2);
    test!(
        &player,
        &states,
        Requirement::And(vec![Requirement::Damage(30.0), Requirement::Damage(30.0)]),
        []
    );
    player.inventory.grant(Item::Skill(Skill::Regenerate), 1);
    test!(
        &player,
        &states,
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
    player.inventory.grant(Item::Skill(Skill::Blaze), 1);
    player
        .inventory
        .grant(Item::Resource(Resource::EnergyFragment), 2);
    test!(
        &player,
        &states,
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

#[test]
fn items_needed() {
    macro_rules! test {
        ($player:expr, $states:expr, $req:expr, [$player_orbs:expr], [$($solutions:expr),* $(,)?]) => {
            {
                fn sort(mut solutions: Vec<Inventory>) -> Vec<Inventory> {
                    solutions.sort_unstable_by_key(|inventory| {
                        let mut items = inventory.items.iter().map(|(item, amount)| format!("{item}{amount}")).collect::<Vec<_>>();
                        items.sort_unstable();
                        items.concat()
                    });  // dumb string based sort
                    solutions
                }
                assert_eq!(sort($req.solutions($player, $states, smallvec![$player_orbs], 1000, 1000)), sort(vec![$($solutions),*]));
            }
        };
        ($player:expr, $states:expr, $req:expr, [$($solutions:tt)*]) => {
            test!($player, $states, $req, [$player.max_orbs()], [$($solutions)*]);
        };
    }

    let world_settings = WorldSettings::default();
    let mut player = Player::new(&world_settings);
    let states = FxHashSet::default();

    test!(&player, &states, Requirement::Free, [Inventory::default()]);
    test!(&player, &states, Requirement::Impossible, []);
    test!(
        &player,
        &states,
        Requirement::Or(vec![Requirement::Free, Requirement::Impossible]),
        [Inventory::default()]
    );
    test!(
        &player,
        &states,
        Requirement::And(vec![Requirement::Free, Requirement::Impossible]),
        []
    );

    test!(
        &player,
        &states,
        Requirement::Skill(Skill::Dash),
        [Item::Skill(Skill::Dash).into()]
    );
    test!(
        &player,
        &states,
        Requirement::Or(vec![
            Requirement::Skill(Skill::Dash),
            Requirement::Skill(Skill::Bash)
        ]),
        [
            Item::Skill(Skill::Dash).into(),
            Item::Skill(Skill::Bash).into()
        ]
    );
    test!(
        &player,
        &states,
        Requirement::And(vec![
            Requirement::Skill(Skill::Dash),
            Requirement::Skill(Skill::Bash)
        ]),
        [[Item::Skill(Skill::Dash), Item::Skill(Skill::Bash)]
            .into_iter()
            .collect()]
    );

    test!(
        &player,
        &states,
        Requirement::EnergySkill(Skill::Grenade, 2.0),
        [[
            (Item::Skill(Skill::Grenade), 1),
            (Item::Resource(Resource::EnergyFragment), 8)
        ]
        .into_iter()
        .collect()]
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    player
        .inventory
        .grant(Item::Resource(Resource::HealthFragment), 8);
    // TODO this should really be equivalent to Requirement::EnergySkill(Skill::Grenade, 2.0)
    test!(
        &player,
        &states,
        Requirement::And(vec![
            Requirement::EnergySkill(Skill::Grenade, 1.0),
            Requirement::EnergySkill(Skill::Grenade, 1.0)
        ]),
        [Orbs::default()],
        [
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 3),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 2),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 5)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Resilience), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Resilience), 1),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Shard(Shard::Resilience), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Skill(Skill::Regenerate), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
        ]
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player = Player::new(&world_settings);
    test!(
        &player,
        &states,
        Requirement::Resource(Resource::ShardSlot, 3),
        [(Item::Resource(Resource::ShardSlot), 3).into()]
    );
    test!(
        &player,
        &states,
        Requirement::Shard(Shard::Overflow),
        [Item::Shard(Shard::Overflow).into()]
    );
    test!(
        &player,
        &states,
        Requirement::Teleporter(Teleporter::Glades),
        [Item::Teleporter(Teleporter::Glades).into()]
    );
    test!(&player, &states, Requirement::Water, [Item::Water.into()]);

    test!(
        &player,
        &states,
        Requirement::Damage(36.0),
        [(Item::Resource(Resource::HealthFragment), 8).into()]
    );
    test!(
        &player,
        &states,
        Requirement::And(vec![Requirement::Damage(18.0), Requirement::Damage(18.0)]),
        [
            (Item::Resource(Resource::HealthFragment), 8).into(),
            [
                (Item::Resource(Resource::HealthFragment), 4),
                (Item::Resource(Resource::EnergyFragment), 4),
                (Item::Skill(Skill::Regenerate), 1)
            ]
            .into_iter()
            .collect(),
        ]
    );
    test!(
        &player,
        &states,
        Requirement::Or(vec![Requirement::Damage(36.0), Requirement::Damage(18.0)]),
        [(Item::Resource(Resource::HealthFragment), 4).into()]
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::And(vec![
            Requirement::Damage(18.0),
            Requirement::Damage(18.0),
            Requirement::Damage(18.0)
        ]),
        [
            (Item::Resource(Resource::HealthFragment), 11).into(),
            [
                (Item::Shard(Shard::Resilience), 1),
                (Item::Resource(Resource::HealthFragment), 10)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Regenerate), 1),
                (Item::Resource(Resource::HealthFragment), 8),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Regenerate), 1),
                (Item::Resource(Resource::HealthFragment), 4),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Regenerate), 1),
                (Item::Shard(Shard::Resilience), 1),
                (Item::Resource(Resource::HealthFragment), 7),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Regenerate), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 4),
                (Item::Resource(Resource::EnergyFragment), 3)
            ]
            .into_iter()
            .collect(),
        ]
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Moki,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::BreakWall(12.0),
        [
            Item::Skill(Skill::Sword).into(),
            Item::Skill(Skill::Hammer).into(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Resource(Resource::EnergyFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Resource(Resource::EnergyFragment), 8)
            ]
            .into_iter()
            .collect(),
        ]
    );

    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    test!(
        &player,
        &states,
        Requirement::BreakWall(12.0),
        [
            Item::Skill(Skill::Sword).into(),
            Item::Skill(Skill::Hammer).into(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 3),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 2),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 5)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 3),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 2),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 5)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Sentry), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
        ]
    );
    player.inventory.grant(Item::Skill(Skill::Bow), 1);
    test!(
        &player,
        &states,
        Requirement::BreakWall(12.0),
        [
            Item::Skill(Skill::Sword).into(),
            Item::Skill(Skill::Hammer).into(),
            [(Item::Resource(Resource::EnergyFragment), 2)]
                .into_iter()
                .collect(),
            [
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
        ]
    );

    let world_settings = WorldSettings::default();
    let mut player = Player::new(&world_settings);
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![(Enemy::Slug, 1)]),
        [
            Item::Skill(Skill::Sword).into(),
            Item::Skill(Skill::Hammer).into(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Resource(Resource::EnergyFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Flash), 1),
                (Item::Resource(Resource::EnergyFragment), 8)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Resource(Resource::EnergyFragment), 8)
            ]
            .into_iter()
            .collect(),
        ]
    );
    player.inventory.grant(Item::Skill(Skill::Launch), 1);
    test!(
        &player,
        &states,
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
            Item::Skill(Skill::Sword).into(),
            Item::Skill(Skill::Hammer).into(),
            [
                (Item::Skill(Skill::Bow), 1),
                (Item::Resource(Resource::EnergyFragment), 31)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Grenade), 1),
                (Item::Resource(Resource::EnergyFragment), 56)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Shuriken), 1),
                (Item::Resource(Resource::EnergyFragment), 46)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Blaze), 1),
                (Item::Resource(Resource::EnergyFragment), 56)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Flash), 1),
                (Item::Resource(Resource::EnergyFragment), 56)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Spear), 1),
                (Item::Resource(Resource::EnergyFragment), 80)
            ]
            .into_iter()
            .collect(),
        ]
    );
    let world_settings = WorldSettings {
        difficulty: Difficulty::Unsafe,
        ..WorldSettings::default()
    };
    player.settings = &world_settings;
    player.inventory.grant(Item::Skill(Skill::Bow), 1);
    // 40 + 32 + (20 * 2) + 24 * 2 + 20 * 3 + 32
    // 10 + 8 + (10) + 12 + 15 + 8 = 63
    test!(
        &player,
        &states,
        Requirement::Combat(smallvec![
            (Enemy::Hornbug, 1),
            (Enemy::Bat, 1),
            (Enemy::Sandworm, 2),
            (Enemy::Lizard, 2),
            (Enemy::Skeeto, 3),
            (Enemy::SneezeSlug, 1)
        ]),
        [
            Item::Skill(Skill::Sword).into(),
            Item::Skill(Skill::Hammer).into(),
            [(Item::Resource(Resource::EnergyFragment), 32)]
                .into_iter()
                .collect(), // 15.75
            [
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 16)
            ]
            .into_iter()
            .collect(), // 7.875
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 31),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 30),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 29),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 28),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 27),
                (Item::Resource(Resource::HealthFragment), 5)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 26),
                (Item::Resource(Resource::HealthFragment), 6)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 25),
                (Item::Resource(Resource::HealthFragment), 7)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 24),
                (Item::Resource(Resource::HealthFragment), 8)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 23),
                (Item::Resource(Resource::HealthFragment), 9)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 22),
                (Item::Resource(Resource::HealthFragment), 10)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 21),
                (Item::Resource(Resource::HealthFragment), 11)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 20),
                (Item::Resource(Resource::HealthFragment), 12)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 19),
                (Item::Resource(Resource::HealthFragment), 13)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 18),
                (Item::Resource(Resource::HealthFragment), 14)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 17),
                (Item::Resource(Resource::HealthFragment), 15)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 16),
                (Item::Resource(Resource::HealthFragment), 16)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 15),
                (Item::Resource(Resource::HealthFragment), 17)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 14),
                (Item::Resource(Resource::HealthFragment), 18)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 13),
                (Item::Resource(Resource::HealthFragment), 19)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 12),
                (Item::Resource(Resource::HealthFragment), 20)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 11),
                (Item::Resource(Resource::HealthFragment), 21)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 10),
                (Item::Resource(Resource::HealthFragment), 22)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 9),
                (Item::Resource(Resource::HealthFragment), 23)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 8),
                (Item::Resource(Resource::HealthFragment), 24)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 7),
                (Item::Resource(Resource::HealthFragment), 25)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 6),
                (Item::Resource(Resource::HealthFragment), 26)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 5),
                (Item::Resource(Resource::HealthFragment), 27)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 4),
                (Item::Resource(Resource::HealthFragment), 28)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 3),
                (Item::Resource(Resource::HealthFragment), 29)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 2),
                (Item::Resource(Resource::HealthFragment), 30)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 31)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 32)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 15),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 14),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 13),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 12),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 11),
                (Item::Resource(Resource::HealthFragment), 5)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 10),
                (Item::Resource(Resource::HealthFragment), 6)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 9),
                (Item::Resource(Resource::HealthFragment), 7)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 8),
                (Item::Resource(Resource::HealthFragment), 8)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 7),
                (Item::Resource(Resource::HealthFragment), 9)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 6),
                (Item::Resource(Resource::HealthFragment), 10)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 5),
                (Item::Resource(Resource::HealthFragment), 11)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 4),
                (Item::Resource(Resource::HealthFragment), 12)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 3),
                (Item::Resource(Resource::HealthFragment), 13)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 2),
                (Item::Resource(Resource::HealthFragment), 14)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 15)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 16)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Resource(Resource::EnergyFragment), 27)
            ]
            .into_iter()
            .collect(), // 13.25
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 14)
            ]
            .into_iter()
            .collect(), // 6.625
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 26),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 25),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 24),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 23),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 22),
                (Item::Resource(Resource::HealthFragment), 5)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 21),
                (Item::Resource(Resource::HealthFragment), 6)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 20),
                (Item::Resource(Resource::HealthFragment), 7)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 19),
                (Item::Resource(Resource::HealthFragment), 8)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 18),
                (Item::Resource(Resource::HealthFragment), 9)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 17),
                (Item::Resource(Resource::HealthFragment), 10)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 16),
                (Item::Resource(Resource::HealthFragment), 11)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 15),
                (Item::Resource(Resource::HealthFragment), 12)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 14),
                (Item::Resource(Resource::HealthFragment), 13)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 13),
                (Item::Resource(Resource::HealthFragment), 14)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 12),
                (Item::Resource(Resource::HealthFragment), 15)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 11),
                (Item::Resource(Resource::HealthFragment), 16)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 10),
                (Item::Resource(Resource::HealthFragment), 17)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 9),
                (Item::Resource(Resource::HealthFragment), 18)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 8),
                (Item::Resource(Resource::HealthFragment), 19)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 7),
                (Item::Resource(Resource::HealthFragment), 20)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 6),
                (Item::Resource(Resource::HealthFragment), 21)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 5),
                (Item::Resource(Resource::HealthFragment), 22)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 4),
                (Item::Resource(Resource::HealthFragment), 23)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 3),
                (Item::Resource(Resource::HealthFragment), 24)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 2),
                (Item::Resource(Resource::HealthFragment), 25)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 26)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Resource(Resource::HealthFragment), 27)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 13),
                (Item::Resource(Resource::HealthFragment), 1)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 12),
                (Item::Resource(Resource::HealthFragment), 2)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 11),
                (Item::Resource(Resource::HealthFragment), 3)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 10),
                (Item::Resource(Resource::HealthFragment), 4)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 9),
                (Item::Resource(Resource::HealthFragment), 5)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 8),
                (Item::Resource(Resource::HealthFragment), 6)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 7),
                (Item::Resource(Resource::HealthFragment), 7)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 6),
                (Item::Resource(Resource::HealthFragment), 8)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 5),
                (Item::Resource(Resource::HealthFragment), 9)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 4),
                (Item::Resource(Resource::HealthFragment), 10)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 3),
                (Item::Resource(Resource::HealthFragment), 11)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 2),
                (Item::Resource(Resource::HealthFragment), 12)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::EnergyFragment), 1),
                (Item::Resource(Resource::HealthFragment), 13)
            ]
            .into_iter()
            .collect(),
            [
                (Item::Skill(Skill::Burrow), 1),
                (Item::Shard(Shard::LifePact), 1),
                (Item::Shard(Shard::Overcharge), 1),
                (Item::Resource(Resource::HealthFragment), 14)
            ]
            .into_iter()
            .collect(),
        ]
    );
}
