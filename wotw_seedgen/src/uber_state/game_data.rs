use super::UberIdentifier;

pub(super) const UBER_STATES: &[(&str, UberIdentifier)] = &[
    (
        "trees.bash",
        UberIdentifier::new(0, 0),
    ),
    (
        "trees.double_jump",
        UberIdentifier::new(0, 5),
    ),
    (
        "trees.launch",
        UberIdentifier::new(0, 8),
    ),
    (
        "trees.grenade",
        UberIdentifier::new(0, 51),
    ),
    (
        "trees.grapple",
        UberIdentifier::new(0, 57),
    ),
    (
        "trees.flash",
        UberIdentifier::new(0, 62),
    ),
    (
        "trees.regenerate",
        UberIdentifier::new(0, 77),
    ),
    (
        "trees.bow",
        UberIdentifier::new(0, 97),
    ),
    (
        "trees.sword",
        UberIdentifier::new(0, 100),
    ),
    (
        "trees.burrow",
        UberIdentifier::new(0, 101),
    ),
    (
        "trees.dash",
        UberIdentifier::new(0, 102),
    ),
    (
        "trees.water_dash",
        UberIdentifier::new(0, 104),
    ),
    (
        "trees.ancestral_light",
        UberIdentifier::new(0, 120),
    ),
    (
        "trees.ancestral_light_2",
        UberIdentifier::new(0, 121),
    ),
    (
        "opher_weapons.Water Breath",
        UberIdentifier::new(1, 23),
    ),
    (
        "opher_weapons.Spike",
        UberIdentifier::new(1, 74),
    ),
    (
        "opher_weapons.Spirit Smash",
        UberIdentifier::new(1, 98),
    ),
    (
        "opher_weapons.Fast Travel",
        UberIdentifier::new(1, 105),
    ),
    (
        "opher_weapons.Spirit Star",
        UberIdentifier::new(1, 106),
    ),
    (
        "opher_weapons.Blaze",
        UberIdentifier::new(1, 115),
    ),
    (
        "opher_weapons.Sentry",
        UberIdentifier::new(1, 116),
    ),
    (
        "opher_weapons.Exploding Spike",
        UberIdentifier::new(1, 1074),
    ),
    (
        "opher_weapons.Shock Smash",
        UberIdentifier::new(1, 1098),
    ),
    (
        "opher_weapons.Static Star",
        UberIdentifier::new(1, 1106),
    ),
    (
        "opher_weapons.Charge Blaze",
        UberIdentifier::new(1, 1115),
    ),
    (
        "opher_weapons.Rapid Sentry",
        UberIdentifier::new(1, 1116),
    ),
    (
        "opher_weapons.Has bought everything",
        UberIdentifier::new(1, 20000),
    ),
    (
        "opher_weapons.Water Breath cost",
        UberIdentifier::new(1, 10023),
    ),
    (
        "opher_weapons.Spike cost",
        UberIdentifier::new(1, 10074),
    ),
    (
        "opher_weapons.Spirit Smash cost",
        UberIdentifier::new(1, 10098),
    ),
    (
        "opher_weapons.Fast Travel cost",
        UberIdentifier::new(1, 10105),
    ),
    (
        "opher_weapons.Spirit Star cost",
        UberIdentifier::new(1, 10106),
    ),
    (
        "opher_weapons.Blaze cost",
        UberIdentifier::new(1, 10115),
    ),
    (
        "opher_weapons.Sentry cost",
        UberIdentifier::new(1, 10116),
    ),
    (
        "opher_weapons.Exploding Spike cost",
        UberIdentifier::new(1, 11074),
    ),
    (
        "opher_weapons.Shock Smash cost",
        UberIdentifier::new(1, 11098),
    ),
    (
        "opher_weapons.Static Star cost",
        UberIdentifier::new(1, 11106),
    ),
    (
        "opher_weapons.Charge Blaze cost",
        UberIdentifier::new(1, 11115),
    ),
    (
        "opher_weapons.Rapid Sentry cost",
        UberIdentifier::new(1, 11116),
    ),
    (
        "twillen_shards.Overcharge",
        UberIdentifier::new(2, 1),
    ),
    (
        "twillen_shards.TripleJump",
        UberIdentifier::new(2, 2),
    ),
    (
        "twillen_shards.Wingclip",
        UberIdentifier::new(2, 3),
    ),
    (
        "twillen_shards.Swap",
        UberIdentifier::new(2, 5),
    ),
    (
        "twillen_shards.LightHarvest",
        UberIdentifier::new(2, 19),
    ),
    (
        "twillen_shards.Vitality",
        UberIdentifier::new(2, 22),
    ),
    (
        "twillen_shards.Energy",
        UberIdentifier::new(2, 26),
    ),
    (
        "twillen_shards.Finesse",
        UberIdentifier::new(2, 40),
    ),
    (
        "twillen_shards.Has bought everything",
        UberIdentifier::new(2, 20000),
    ),
    (
        "twillen_shards.Overcharge cost",
        UberIdentifier::new(2, 101),
    ),
    (
        "twillen_shards.TripleJump cost",
        UberIdentifier::new(2, 102),
    ),
    (
        "twillen_shards.Wingclip cost",
        UberIdentifier::new(2, 103),
    ),
    (
        "twillen_shards.Swap cost",
        UberIdentifier::new(2, 105),
    ),
    (
        "twillen_shards.LightHarvest cost",
        UberIdentifier::new(2, 119),
    ),
    (
        "twillen_shards.Vitality cost",
        UberIdentifier::new(2, 122),
    ),
    (
        "twillen_shards.Energy cost",
        UberIdentifier::new(2, 126),
    ),
    (
        "twillen_shards.Finesse cost",
        UberIdentifier::new(2, 140),
    ),
    (
        "game_state.Spawn",
        UberIdentifier::new(3, 0),
    ),
    (
        "game_state.Goal Modes Complete",
        UberIdentifier::new(3, 11),
    ),
    (
        "game_state.On Teleport",
        UberIdentifier::new(3, 20),
    ),
    (
        "game_state.Reload",
        UberIdentifier::new(3, 1),
    ),
    (
        "game_state.Binding One",
        UberIdentifier::new(3, 2),
    ),
    (
        "game_state.Binding Two",
        UberIdentifier::new(3, 3),
    ),
    (
        "game_state.Binding Three",
        UberIdentifier::new(3, 4),
    ),
    (
        "game_state.Binding Four",
        UberIdentifier::new(3, 5),
    ),
    (
        "game_state.Binding Five",
        UberIdentifier::new(3, 6),
    ),
    (
        "game_state.Load",
        UberIdentifier::new(3, 7),
    ),
    (
        "rando_upgrades.Autoaim",
        UberIdentifier::new(4, 37),
    ),
    (
        "rando_upgrades.Grenades explode on collision",
        UberIdentifier::new(4, 41),
    ),
    (
        "rando_upgrades.Bashable uncharged Grenades",
        UberIdentifier::new(4, 42),
    ),
    (
        "rando_upgrades.Charged Air Grenades",
        UberIdentifier::new(4, 43),
    ),
    (
        "rando_upgrades.Bow as fire source",
        UberIdentifier::new(4, 70),
    ),
    (
        "rando_upgrades.Blaze as fire source",
        UberIdentifier::new(4, 71),
    ),
    (
        "rando_upgrades.Sword as fire source",
        UberIdentifier::new(4, 72),
    ),
    (
        "rando_upgrades.Hammer as fire source",
        UberIdentifier::new(4, 73),
    ),
    (
        "rando_upgrades.Spear as fire source",
        UberIdentifier::new(4, 74),
    ),
    (
        "rando_upgrades.Shuriken as fire source",
        UberIdentifier::new(4, 75),
    ),
    (
        "rando_upgrades.Hammer speed multiplier",
        UberIdentifier::new(4, 0),
    ),
    (
        "rando_upgrades.Sword speed multiplier",
        UberIdentifier::new(4, 1),
    ),
    (
        "rando_upgrades.Blaze cost multiplier",
        UberIdentifier::new(4, 2),
    ),
    (
        "rando_upgrades.Spike cost multiplier",
        UberIdentifier::new(4, 3),
    ),
    (
        "rando_upgrades.Shuriken cost multiplier",
        UberIdentifier::new(4, 4),
    ),
    (
        "rando_upgrades.Sentry cost multiplier",
        UberIdentifier::new(4, 5),
    ),
    (
        "rando_upgrades.Bow cost multiplier",
        UberIdentifier::new(4, 6),
    ),
    (
        "rando_upgrades.Regeneration cost multiplier",
        UberIdentifier::new(4, 7),
    ),
    (
        "rando_upgrades.Flash cost multiplier",
        UberIdentifier::new(4, 8),
    ),
    (
        "rando_upgrades.Light Burst cost multiplier",
        UberIdentifier::new(4, 9),
    ),
    (
        "rando_upgrades.Bow rapid fire multiplier",
        UberIdentifier::new(4, 10),
    ),
    (
        "rando_upgrades.Spear speed multiplier",
        UberIdentifier::new(4, 11),
    ),
    (
        "rando_upgrades.Grenade charge time modifier",
        UberIdentifier::new(4, 44),
    ),
    (
        "rando_upgrades.Launch Speed",
        UberIdentifier::new(4, 80),
    ),
    (
        "rando_upgrades.Dash Distance",
        UberIdentifier::new(4, 81),
    ),
    (
        "rando_upgrades.Bash Speed",
        UberIdentifier::new(4, 82),
    ),
    (
        "rando_upgrades.Burrow Speed",
        UberIdentifier::new(4, 83),
    ),
    (
        "rando_upgrades.Burrow Dash Speed",
        UberIdentifier::new(4, 84),
    ),
    (
        "rando_upgrades.Swim Speed",
        UberIdentifier::new(4, 85),
    ),
    (
        "rando_upgrades.Swim Dash Speed",
        UberIdentifier::new(4, 86),
    ),
    (
        "rando_upgrades.Jump Height",
        UberIdentifier::new(4, 87),
    ),
    (
        "rando_upgrades.Relic",
        UberIdentifier::new(4, 20),
    ),
    (
        "rando_upgrades.Health Regeneration",
        UberIdentifier::new(4, 30),
    ),
    (
        "rando_upgrades.Energy Regeneration",
        UberIdentifier::new(4, 31),
    ),
    (
        "rando_upgrades.Extra Double Jumps",
        UberIdentifier::new(4, 35),
    ),
    (
        "rando_upgrades.Extra Dashes",
        UberIdentifier::new(4, 36),
    ),
    (
        "rando_upgrades.Extra Grenades",
        UberIdentifier::new(4, 40),
    ),
    (
        "rando_upgrades.Grenade multishot",
        UberIdentifier::new(4, 45),
    ),
    (
        "rando_upgrades.Hammer Speed",
        UberIdentifier::new(4, 50),
    ),
    (
        "rando_upgrades.Sword Speed",
        UberIdentifier::new(4, 51),
    ),
    (
        "rando_upgrades.Blaze Efficiency",
        UberIdentifier::new(4, 52),
    ),
    (
        "rando_upgrades.Spike Efficiency",
        UberIdentifier::new(4, 53),
    ),
    (
        "rando_upgrades.Shuriken Efficiency",
        UberIdentifier::new(4, 54),
    ),
    (
        "rando_upgrades.Sentry Efficiency",
        UberIdentifier::new(4, 55),
    ),
    (
        "rando_upgrades.Bow Efficiency",
        UberIdentifier::new(4, 56),
    ),
    (
        "rando_upgrades.Regenerate Efficiency",
        UberIdentifier::new(4, 57),
    ),
    (
        "rando_upgrades.Flash Efficiency",
        UberIdentifier::new(4, 58),
    ),
    (
        "rando_upgrades.Light Burst Efficiency",
        UberIdentifier::new(4, 59),
    ),
    (
        "rando_upgrades.Exploding Spike",
        UberIdentifier::new(4, 95),
    ),
    (
        "rando_upgrades.Shock Smash",
        UberIdentifier::new(4, 96),
    ),
    (
        "rando_upgrades.Static Star",
        UberIdentifier::new(4, 97),
    ),
    (
        "rando_upgrades.Charge Blaze",
        UberIdentifier::new(4, 98),
    ),
    (
        "rando_upgrades.Rapid Sentry",
        UberIdentifier::new(4, 99),
    ),
    (
        "rando_upgrades.Marsh Relic",
        UberIdentifier::new(4, 100),
    ),
    (
        "rando_upgrades.Hollow Relic",
        UberIdentifier::new(4, 101),
    ),
    (
        "rando_upgrades.Glades Relic",
        UberIdentifier::new(4, 102),
    ),
    (
        "rando_upgrades.Wellspring Relic",
        UberIdentifier::new(4, 103),
    ),
    (
        "rando_upgrades.Burrows Relic",
        UberIdentifier::new(4, 104),
    ),
    (
        "rando_upgrades.Woods Relic",
        UberIdentifier::new(4, 105),
    ),
    (
        "rando_upgrades.Reach Relic",
        UberIdentifier::new(4, 106),
    ),
    (
        "rando_upgrades.Pools Relic",
        UberIdentifier::new(4, 107),
    ),
    (
        "rando_upgrades.Depths Relic",
        UberIdentifier::new(4, 108),
    ),
    (
        "rando_upgrades.Wastes Relic",
        UberIdentifier::new(4, 109),
    ),
    (
        "rando_upgrades.Willow Relic",
        UberIdentifier::new(4, 111),
    ),
    (
        "rando_state.Checkable Item Hint 1",
        UberIdentifier::new(6, 10),
    ),
    (
        "rando_state.Checkable Item Hint 2",
        UberIdentifier::new(6, 11),
    ),
    (
        "rando_state.Checkable Item Hint 3",
        UberIdentifier::new(6, 12),
    ),
    (
        "rando_state.Checkable Item Hint 4",
        UberIdentifier::new(6, 13),
    ),
    (
        "rando_state.Checkable Item Hint 5",
        UberIdentifier::new(6, 14),
    ),
    (
        "rando_state.Checkable Item Hint 6",
        UberIdentifier::new(6, 15),
    ),
    (
        "rando_state.Checkable Item Hint 7",
        UberIdentifier::new(6, 16),
    ),
    (
        "rando_state.Checkable Item Hint 8",
        UberIdentifier::new(6, 17),
    ),
    (
        "rando_state.Checkable Item Hint 9",
        UberIdentifier::new(6, 18),
    ),
    (
        "rando_state.Checkable Item Hint 10",
        UberIdentifier::new(6, 19),
    ),
    (
        "rando_state.HollowTP",
        UberIdentifier::new(6, 106),
    ),
    (
        "rando_state.Bash",
        UberIdentifier::new(6, 1000),
    ),
    (
        "rando_state.WallJump",
        UberIdentifier::new(6, 1003),
    ),
    (
        "rando_state.DoubleJump",
        UberIdentifier::new(6, 1005),
    ),
    (
        "rando_state.Launch",
        UberIdentifier::new(6, 1008),
    ),
    (
        "rando_state.Feather",
        UberIdentifier::new(6, 1014),
    ),
    (
        "rando_state.Spirit Flame",
        UberIdentifier::new(6, 1015),
    ),
    (
        "rando_state.WaterBreath",
        UberIdentifier::new(6, 1023),
    ),
    (
        "rando_state.Resilience",
        UberIdentifier::new(6, 1031),
    ),
    (
        "rando_state.Health Efficiency",
        UberIdentifier::new(6, 1032),
    ),
    (
        "rando_state.Energy Efficiency",
        UberIdentifier::new(6, 1039),
    ),
    (
        "rando_state.LightBurst",
        UberIdentifier::new(6, 1051),
    ),
    (
        "rando_state.Grapple",
        UberIdentifier::new(6, 1057),
    ),
    (
        "rando_state.Flash",
        UberIdentifier::new(6, 1062),
    ),
    (
        "rando_state.Spike",
        UberIdentifier::new(6, 1074),
    ),
    (
        "rando_state.Regenerate",
        UberIdentifier::new(6, 1077),
    ),
    (
        "rando_state.SpiritArc",
        UberIdentifier::new(6, 1097),
    ),
    (
        "rando_state.SpiritSmash",
        UberIdentifier::new(6, 1098),
    ),
    (
        "rando_state.Torch",
        UberIdentifier::new(6, 1099),
    ),
    (
        "rando_state.SpiritEdge",
        UberIdentifier::new(6, 1100),
    ),
    (
        "rando_state.Burrow",
        UberIdentifier::new(6, 1101),
    ),
    (
        "rando_state.Dash",
        UberIdentifier::new(6, 1102),
    ),
    (
        "rando_state.WaterDash",
        UberIdentifier::new(6, 1104),
    ),
    (
        "rando_state.SpiritStar",
        UberIdentifier::new(6, 1106),
    ),
    (
        "rando_state.Seir",
        UberIdentifier::new(6, 1108),
    ),
    (
        "rando_state.Bow Charge",
        UberIdentifier::new(6, 1109),
    ),
    (
        "rando_state.Spirit Magnet",
        UberIdentifier::new(6, 1112),
    ),
    (
        "rando_state.Blaze",
        UberIdentifier::new(6, 1115),
    ),
    (
        "rando_state.Sentry",
        UberIdentifier::new(6, 1116),
    ),
    (
        "rando_state.Flap",
        UberIdentifier::new(6, 1118),
    ),
    (
        "rando_state.Weapon Charge",
        UberIdentifier::new(6, 1119),
    ),
    (
        "rando_state.DamageUpgrade1",
        UberIdentifier::new(6, 1120),
    ),
    (
        "rando_state.DamageUpgrade2",
        UberIdentifier::new(6, 1121),
    ),
    (
        "rando_state.Clean Water",
        UberIdentifier::new(6, 2000),
    ),
    (
        "rando_state.Collected Keystones",
        UberIdentifier::new(6, 0),
    ),
    (
        "rando_state.Purchased Keystones",
        UberIdentifier::new(6, 1),
    ),
    (
        "rando_state.Pickups Collected",
        UberIdentifier::new(6, 2),
    ),
    (
        "rando_state.Spirit Light Collected",
        UberIdentifier::new(6, 3),
    ),
    (
        "rando_state.Spirit Light Spent",
        UberIdentifier::new(6, 4),
    ),
    (
        "rando_state.Ore Collected",
        UberIdentifier::new(6, 5),
    ),
    (
        "rando_state.Ore Spent",
        UberIdentifier::new(6, 6),
    ),
    (
        "rando_state.Marsh Key Item Hint",
        UberIdentifier::new(6, 10000),
    ),
    (
        "rando_state.Hollow Key Item Hint",
        UberIdentifier::new(6, 10001),
    ),
    (
        "rando_state.Glades Key Item Hint",
        UberIdentifier::new(6, 10002),
    ),
    (
        "rando_state.Wellspring Key Item Hint",
        UberIdentifier::new(6, 10003),
    ),
    (
        "rando_state.Burrows Key Item Hint",
        UberIdentifier::new(6, 10004),
    ),
    (
        "rando_state.Woods Key Item Hint",
        UberIdentifier::new(6, 10005),
    ),
    (
        "rando_state.Reach Key Item Hint",
        UberIdentifier::new(6, 10006),
    ),
    (
        "rando_state.Pools Key Item Hint",
        UberIdentifier::new(6, 10007),
    ),
    (
        "rando_state.Depths Key Item Hint",
        UberIdentifier::new(6, 10008),
    ),
    (
        "rando_state.Wastes Key Item Hint",
        UberIdentifier::new(6, 10009),
    ),
    (
        "rando_state.Willow Key Item Hint",
        UberIdentifier::new(6, 10011),
    ),
    (
        "rando_config.glades_tp_fix",
        UberIdentifier::new(7, 0),
    ),
    (
        "rando_config.prevent_map_reactivate_tps",
        UberIdentifier::new(7, 1),
    ),
    (
        "rando_config.marsh_starts_sunny",
        UberIdentifier::new(7, 2),
    ),
    (
        "rando_config.howl_starts_dead",
        UberIdentifier::new(7, 3),
    ),
    (
        "rando_config.enable_vanilla_regen_tree",
        UberIdentifier::new(7, 4),
    ),
    (
        "rando_config.disable_tree_check_for_rain",
        UberIdentifier::new(7, 5),
    ),
    (
        "map_filter.show_spoiler",
        UberIdentifier::new(8, 70),
    ),
    (
        "plando_vars.100_bool",
        UberIdentifier::new(9, 100),
    ),
    (
        "plando_vars.101_bool",
        UberIdentifier::new(9, 101),
    ),
    (
        "plando_vars.102_bool",
        UberIdentifier::new(9, 102),
    ),
    (
        "plando_vars.103_bool",
        UberIdentifier::new(9, 103),
    ),
    (
        "plando_vars.104_bool",
        UberIdentifier::new(9, 104),
    ),
    (
        "plando_vars.105_bool",
        UberIdentifier::new(9, 105),
    ),
    (
        "plando_vars.106_bool",
        UberIdentifier::new(9, 106),
    ),
    (
        "plando_vars.107_bool",
        UberIdentifier::new(9, 107),
    ),
    (
        "plando_vars.108_bool",
        UberIdentifier::new(9, 108),
    ),
    (
        "plando_vars.109_bool",
        UberIdentifier::new(9, 109),
    ),
    (
        "plando_vars.110_bool",
        UberIdentifier::new(9, 110),
    ),
    (
        "plando_vars.111_bool",
        UberIdentifier::new(9, 111),
    ),
    (
        "plando_vars.112_bool",
        UberIdentifier::new(9, 112),
    ),
    (
        "plando_vars.113_bool",
        UberIdentifier::new(9, 113),
    ),
    (
        "plando_vars.114_bool",
        UberIdentifier::new(9, 114),
    ),
    (
        "plando_vars.115_bool",
        UberIdentifier::new(9, 115),
    ),
    (
        "plando_vars.116_bool",
        UberIdentifier::new(9, 116),
    ),
    (
        "plando_vars.117_bool",
        UberIdentifier::new(9, 117),
    ),
    (
        "plando_vars.118_bool",
        UberIdentifier::new(9, 118),
    ),
    (
        "plando_vars.119_bool",
        UberIdentifier::new(9, 119),
    ),
    (
        "plando_vars.120_bool",
        UberIdentifier::new(9, 120),
    ),
    (
        "plando_vars.121_bool",
        UberIdentifier::new(9, 121),
    ),
    (
        "plando_vars.122_bool",
        UberIdentifier::new(9, 122),
    ),
    (
        "plando_vars.123_bool",
        UberIdentifier::new(9, 123),
    ),
    (
        "plando_vars.124_bool",
        UberIdentifier::new(9, 124),
    ),
    (
        "plando_vars.125_bool",
        UberIdentifier::new(9, 125),
    ),
    (
        "plando_vars.126_bool",
        UberIdentifier::new(9, 126),
    ),
    (
        "plando_vars.127_bool",
        UberIdentifier::new(9, 127),
    ),
    (
        "plando_vars.128_bool",
        UberIdentifier::new(9, 128),
    ),
    (
        "plando_vars.129_bool",
        UberIdentifier::new(9, 129),
    ),
    (
        "plando_vars.130_bool",
        UberIdentifier::new(9, 130),
    ),
    (
        "plando_vars.131_bool",
        UberIdentifier::new(9, 131),
    ),
    (
        "plando_vars.132_bool",
        UberIdentifier::new(9, 132),
    ),
    (
        "plando_vars.133_bool",
        UberIdentifier::new(9, 133),
    ),
    (
        "plando_vars.134_bool",
        UberIdentifier::new(9, 134),
    ),
    (
        "plando_vars.135_bool",
        UberIdentifier::new(9, 135),
    ),
    (
        "plando_vars.136_bool",
        UberIdentifier::new(9, 136),
    ),
    (
        "plando_vars.137_bool",
        UberIdentifier::new(9, 137),
    ),
    (
        "plando_vars.138_bool",
        UberIdentifier::new(9, 138),
    ),
    (
        "plando_vars.139_bool",
        UberIdentifier::new(9, 139),
    ),
    (
        "plando_vars.140_bool",
        UberIdentifier::new(9, 140),
    ),
    (
        "plando_vars.141_bool",
        UberIdentifier::new(9, 141),
    ),
    (
        "plando_vars.142_bool",
        UberIdentifier::new(9, 142),
    ),
    (
        "plando_vars.143_bool",
        UberIdentifier::new(9, 143),
    ),
    (
        "plando_vars.144_bool",
        UberIdentifier::new(9, 144),
    ),
    (
        "plando_vars.145_bool",
        UberIdentifier::new(9, 145),
    ),
    (
        "plando_vars.146_bool",
        UberIdentifier::new(9, 146),
    ),
    (
        "plando_vars.147_bool",
        UberIdentifier::new(9, 147),
    ),
    (
        "plando_vars.148_bool",
        UberIdentifier::new(9, 148),
    ),
    (
        "plando_vars.149_bool",
        UberIdentifier::new(9, 149),
    ),
    (
        "plando_vars.0_int",
        UberIdentifier::new(9, 0),
    ),
    (
        "plando_vars.1_int",
        UberIdentifier::new(9, 1),
    ),
    (
        "plando_vars.2_int",
        UberIdentifier::new(9, 2),
    ),
    (
        "plando_vars.3_int",
        UberIdentifier::new(9, 3),
    ),
    (
        "plando_vars.4_int",
        UberIdentifier::new(9, 4),
    ),
    (
        "plando_vars.5_int",
        UberIdentifier::new(9, 5),
    ),
    (
        "plando_vars.6_int",
        UberIdentifier::new(9, 6),
    ),
    (
        "plando_vars.7_int",
        UberIdentifier::new(9, 7),
    ),
    (
        "plando_vars.8_int",
        UberIdentifier::new(9, 8),
    ),
    (
        "plando_vars.9_int",
        UberIdentifier::new(9, 9),
    ),
    (
        "plando_vars.10_int",
        UberIdentifier::new(9, 10),
    ),
    (
        "plando_vars.11_int",
        UberIdentifier::new(9, 11),
    ),
    (
        "plando_vars.12_int",
        UberIdentifier::new(9, 12),
    ),
    (
        "plando_vars.13_int",
        UberIdentifier::new(9, 13),
    ),
    (
        "plando_vars.14_int",
        UberIdentifier::new(9, 14),
    ),
    (
        "plando_vars.15_int",
        UberIdentifier::new(9, 15),
    ),
    (
        "plando_vars.16_int",
        UberIdentifier::new(9, 16),
    ),
    (
        "plando_vars.17_int",
        UberIdentifier::new(9, 17),
    ),
    (
        "plando_vars.18_int",
        UberIdentifier::new(9, 18),
    ),
    (
        "plando_vars.19_int",
        UberIdentifier::new(9, 19),
    ),
    (
        "plando_vars.20_int",
        UberIdentifier::new(9, 20),
    ),
    (
        "plando_vars.21_int",
        UberIdentifier::new(9, 21),
    ),
    (
        "plando_vars.22_int",
        UberIdentifier::new(9, 22),
    ),
    (
        "plando_vars.23_int",
        UberIdentifier::new(9, 23),
    ),
    (
        "plando_vars.24_int",
        UberIdentifier::new(9, 24),
    ),
    (
        "plando_vars.25_int",
        UberIdentifier::new(9, 25),
    ),
    (
        "plando_vars.26_int",
        UberIdentifier::new(9, 26),
    ),
    (
        "plando_vars.27_int",
        UberIdentifier::new(9, 27),
    ),
    (
        "plando_vars.28_int",
        UberIdentifier::new(9, 28),
    ),
    (
        "plando_vars.29_int",
        UberIdentifier::new(9, 29),
    ),
    (
        "plando_vars.30_int",
        UberIdentifier::new(9, 30),
    ),
    (
        "plando_vars.31_int",
        UberIdentifier::new(9, 31),
    ),
    (
        "plando_vars.32_int",
        UberIdentifier::new(9, 32),
    ),
    (
        "plando_vars.33_int",
        UberIdentifier::new(9, 33),
    ),
    (
        "plando_vars.34_int",
        UberIdentifier::new(9, 34),
    ),
    (
        "plando_vars.35_int",
        UberIdentifier::new(9, 35),
    ),
    (
        "plando_vars.36_int",
        UberIdentifier::new(9, 36),
    ),
    (
        "plando_vars.37_int",
        UberIdentifier::new(9, 37),
    ),
    (
        "plando_vars.38_int",
        UberIdentifier::new(9, 38),
    ),
    (
        "plando_vars.39_int",
        UberIdentifier::new(9, 39),
    ),
    (
        "plando_vars.40_int",
        UberIdentifier::new(9, 40),
    ),
    (
        "plando_vars.41_int",
        UberIdentifier::new(9, 41),
    ),
    (
        "plando_vars.42_int",
        UberIdentifier::new(9, 42),
    ),
    (
        "plando_vars.43_int",
        UberIdentifier::new(9, 43),
    ),
    (
        "plando_vars.44_int",
        UberIdentifier::new(9, 44),
    ),
    (
        "plando_vars.45_int",
        UberIdentifier::new(9, 45),
    ),
    (
        "plando_vars.46_int",
        UberIdentifier::new(9, 46),
    ),
    (
        "plando_vars.47_int",
        UberIdentifier::new(9, 47),
    ),
    (
        "plando_vars.48_int",
        UberIdentifier::new(9, 48),
    ),
    (
        "plando_vars.49_int",
        UberIdentifier::new(9, 49),
    ),
    (
        "plando_vars.50_int",
        UberIdentifier::new(9, 50),
    ),
    (
        "plando_vars.51_int",
        UberIdentifier::new(9, 51),
    ),
    (
        "plando_vars.52_int",
        UberIdentifier::new(9, 52),
    ),
    (
        "plando_vars.53_int",
        UberIdentifier::new(9, 53),
    ),
    (
        "plando_vars.54_int",
        UberIdentifier::new(9, 54),
    ),
    (
        "plando_vars.55_int",
        UberIdentifier::new(9, 55),
    ),
    (
        "plando_vars.56_int",
        UberIdentifier::new(9, 56),
    ),
    (
        "plando_vars.57_int",
        UberIdentifier::new(9, 57),
    ),
    (
        "plando_vars.58_int",
        UberIdentifier::new(9, 58),
    ),
    (
        "plando_vars.59_int",
        UberIdentifier::new(9, 59),
    ),
    (
        "plando_vars.60_int",
        UberIdentifier::new(9, 60),
    ),
    (
        "plando_vars.61_int",
        UberIdentifier::new(9, 61),
    ),
    (
        "plando_vars.62_int",
        UberIdentifier::new(9, 62),
    ),
    (
        "plando_vars.63_int",
        UberIdentifier::new(9, 63),
    ),
    (
        "plando_vars.64_int",
        UberIdentifier::new(9, 64),
    ),
    (
        "plando_vars.65_int",
        UberIdentifier::new(9, 65),
    ),
    (
        "plando_vars.66_int",
        UberIdentifier::new(9, 66),
    ),
    (
        "plando_vars.67_int",
        UberIdentifier::new(9, 67),
    ),
    (
        "plando_vars.68_int",
        UberIdentifier::new(9, 68),
    ),
    (
        "plando_vars.69_int",
        UberIdentifier::new(9, 69),
    ),
    (
        "plando_vars.70_int",
        UberIdentifier::new(9, 70),
    ),
    (
        "plando_vars.71_int",
        UberIdentifier::new(9, 71),
    ),
    (
        "plando_vars.72_int",
        UberIdentifier::new(9, 72),
    ),
    (
        "plando_vars.73_int",
        UberIdentifier::new(9, 73),
    ),
    (
        "plando_vars.74_int",
        UberIdentifier::new(9, 74),
    ),
    (
        "plando_vars.75_int",
        UberIdentifier::new(9, 75),
    ),
    (
        "plando_vars.76_int",
        UberIdentifier::new(9, 76),
    ),
    (
        "plando_vars.77_int",
        UberIdentifier::new(9, 77),
    ),
    (
        "plando_vars.78_int",
        UberIdentifier::new(9, 78),
    ),
    (
        "plando_vars.79_int",
        UberIdentifier::new(9, 79),
    ),
    (
        "plando_vars.80_int",
        UberIdentifier::new(9, 80),
    ),
    (
        "plando_vars.81_int",
        UberIdentifier::new(9, 81),
    ),
    (
        "plando_vars.82_int",
        UberIdentifier::new(9, 82),
    ),
    (
        "plando_vars.83_int",
        UberIdentifier::new(9, 83),
    ),
    (
        "plando_vars.84_int",
        UberIdentifier::new(9, 84),
    ),
    (
        "plando_vars.85_int",
        UberIdentifier::new(9, 85),
    ),
    (
        "plando_vars.86_int",
        UberIdentifier::new(9, 86),
    ),
    (
        "plando_vars.87_int",
        UberIdentifier::new(9, 87),
    ),
    (
        "plando_vars.88_int",
        UberIdentifier::new(9, 88),
    ),
    (
        "plando_vars.89_int",
        UberIdentifier::new(9, 89),
    ),
    (
        "plando_vars.90_int",
        UberIdentifier::new(9, 90),
    ),
    (
        "plando_vars.91_int",
        UberIdentifier::new(9, 91),
    ),
    (
        "plando_vars.92_int",
        UberIdentifier::new(9, 92),
    ),
    (
        "plando_vars.93_int",
        UberIdentifier::new(9, 93),
    ),
    (
        "plando_vars.94_int",
        UberIdentifier::new(9, 94),
    ),
    (
        "plando_vars.95_int",
        UberIdentifier::new(9, 95),
    ),
    (
        "plando_vars.96_int",
        UberIdentifier::new(9, 96),
    ),
    (
        "plando_vars.97_int",
        UberIdentifier::new(9, 97),
    ),
    (
        "plando_vars.98_int",
        UberIdentifier::new(9, 98),
    ),
    (
        "plando_vars.99_int",
        UberIdentifier::new(9, 99),
    ),
    (
        "plando_vars.150_float",
        UberIdentifier::new(9, 150),
    ),
    (
        "plando_vars.151_float",
        UberIdentifier::new(9, 151),
    ),
    (
        "plando_vars.152_float",
        UberIdentifier::new(9, 152),
    ),
    (
        "plando_vars.153_float",
        UberIdentifier::new(9, 153),
    ),
    (
        "plando_vars.154_float",
        UberIdentifier::new(9, 154),
    ),
    (
        "plando_vars.155_float",
        UberIdentifier::new(9, 155),
    ),
    (
        "plando_vars.156_float",
        UberIdentifier::new(9, 156),
    ),
    (
        "plando_vars.157_float",
        UberIdentifier::new(9, 157),
    ),
    (
        "plando_vars.158_float",
        UberIdentifier::new(9, 158),
    ),
    (
        "plando_vars.159_float",
        UberIdentifier::new(9, 159),
    ),
    (
        "plando_vars.160_float",
        UberIdentifier::new(9, 160),
    ),
    (
        "plando_vars.161_float",
        UberIdentifier::new(9, 161),
    ),
    (
        "plando_vars.162_float",
        UberIdentifier::new(9, 162),
    ),
    (
        "plando_vars.163_float",
        UberIdentifier::new(9, 163),
    ),
    (
        "plando_vars.164_float",
        UberIdentifier::new(9, 164),
    ),
    (
        "plando_vars.165_float",
        UberIdentifier::new(9, 165),
    ),
    (
        "plando_vars.166_float",
        UberIdentifier::new(9, 166),
    ),
    (
        "plando_vars.167_float",
        UberIdentifier::new(9, 167),
    ),
    (
        "plando_vars.168_float",
        UberIdentifier::new(9, 168),
    ),
    (
        "plando_vars.169_float",
        UberIdentifier::new(9, 169),
    ),
    (
        "plando_vars.170_float",
        UberIdentifier::new(9, 170),
    ),
    (
        "plando_vars.171_float",
        UberIdentifier::new(9, 171),
    ),
    (
        "plando_vars.172_float",
        UberIdentifier::new(9, 172),
    ),
    (
        "plando_vars.173_float",
        UberIdentifier::new(9, 173),
    ),
    (
        "plando_vars.174_float",
        UberIdentifier::new(9, 174),
    ),
    (
        "bingo_state.Squares",
        UberIdentifier::new(10, 0),
    ),
    (
        "bingo_state.Lines",
        UberIdentifier::new(10, 1),
    ),
    (
        "bingo_state.Rank",
        UberIdentifier::new(10, 2),
    ),
    (
        "bingo_state.Kills",
        UberIdentifier::new(10, 10),
    ),
    (
        "bingo_state.SwordKills",
        UberIdentifier::new(10, 11),
    ),
    (
        "bingo_state.HammerKills",
        UberIdentifier::new(10, 12),
    ),
    (
        "bingo_state.BowKills",
        UberIdentifier::new(10, 13),
    ),
    (
        "bingo_state.SpearKills",
        UberIdentifier::new(10, 14),
    ),
    (
        "bingo_state.SentryKills",
        UberIdentifier::new(10, 15),
    ),
    (
        "bingo_state.BlazeKills",
        UberIdentifier::new(10, 16),
    ),
    (
        "bingo_state.GrenadeKills",
        UberIdentifier::new(10, 17),
    ),
    (
        "bingo_state.BurnDoTKills",
        UberIdentifier::new(10, 18),
    ),
    (
        "bingo_state.ShurikenKills",
        UberIdentifier::new(10, 19),
    ),
    (
        "bingo_state.LaunchKills",
        UberIdentifier::new(10, 20),
    ),
    (
        "bingo_state.FlashKills",
        UberIdentifier::new(10, 21),
    ),
    (
        "bingo_state.BashKills",
        UberIdentifier::new(10, 22),
    ),
    (
        "bingo_state.DrownedEnemies",
        UberIdentifier::new(10, 23),
    ),
    (
        "bingo_state.MinerKills",
        UberIdentifier::new(10, 40),
    ),
    (
        "bingo_state.FlierKills",
        UberIdentifier::new(10, 41),
    ),
    (
        "bingo_state.TentaKills",
        UberIdentifier::new(10, 42),
    ),
    (
        "bingo_state.SlimeKills",
        UberIdentifier::new(10, 43),
    ),
    (
        "bingo_state.FishKills",
        UberIdentifier::new(10, 44),
    ),
    (
        "bingo_state.ExploderKills",
        UberIdentifier::new(10, 45),
    ),
    (
        "appliers_serialization.0_id",
        UberIdentifier::new(11, 0),
    ),
    (
        "appliers_serialization.1_value",
        UberIdentifier::new(11, 1),
    ),
    (
        "appliers_serialization.2_id",
        UberIdentifier::new(11, 2),
    ),
    (
        "appliers_serialization.3_value",
        UberIdentifier::new(11, 3),
    ),
    (
        "appliers_serialization.4_id",
        UberIdentifier::new(11, 4),
    ),
    (
        "appliers_serialization.5_value",
        UberIdentifier::new(11, 5),
    ),
    (
        "appliers_serialization.6_id",
        UberIdentifier::new(11, 6),
    ),
    (
        "appliers_serialization.7_value",
        UberIdentifier::new(11, 7),
    ),
    (
        "appliers_serialization.8_id",
        UberIdentifier::new(11, 8),
    ),
    (
        "appliers_serialization.9_value",
        UberIdentifier::new(11, 9),
    ),
    (
        "appliers_serialization.10_id",
        UberIdentifier::new(11, 10),
    ),
    (
        "appliers_serialization.11_value",
        UberIdentifier::new(11, 11),
    ),
    (
        "appliers_serialization.12_id",
        UberIdentifier::new(11, 12),
    ),
    (
        "appliers_serialization.13_value",
        UberIdentifier::new(11, 13),
    ),
    (
        "appliers_serialization.14_id",
        UberIdentifier::new(11, 14),
    ),
    (
        "appliers_serialization.15_value",
        UberIdentifier::new(11, 15),
    ),
    (
        "appliers_serialization.16_id",
        UberIdentifier::new(11, 16),
    ),
    (
        "appliers_serialization.17_value",
        UberIdentifier::new(11, 17),
    ),
    (
        "appliers_serialization.18_id",
        UberIdentifier::new(11, 18),
    ),
    (
        "appliers_serialization.19_value",
        UberIdentifier::new(11, 19),
    ),
    (
        "appliers_serialization.20_id",
        UberIdentifier::new(11, 20),
    ),
    (
        "appliers_serialization.21_value",
        UberIdentifier::new(11, 21),
    ),
    (
        "appliers_serialization.22_id",
        UberIdentifier::new(11, 22),
    ),
    (
        "appliers_serialization.23_value",
        UberIdentifier::new(11, 23),
    ),
    (
        "appliers_serialization.24_id",
        UberIdentifier::new(11, 24),
    ),
    (
        "appliers_serialization.25_value",
        UberIdentifier::new(11, 25),
    ),
    (
        "appliers_serialization.26_id",
        UberIdentifier::new(11, 26),
    ),
    (
        "appliers_serialization.27_value",
        UberIdentifier::new(11, 27),
    ),
    (
        "appliers_serialization.28_id",
        UberIdentifier::new(11, 28),
    ),
    (
        "appliers_serialization.29_value",
        UberIdentifier::new(11, 29),
    ),
    (
        "appliers_serialization.30_id",
        UberIdentifier::new(11, 30),
    ),
    (
        "appliers_serialization.31_value",
        UberIdentifier::new(11, 31),
    ),
    (
        "appliers_serialization.32_id",
        UberIdentifier::new(11, 32),
    ),
    (
        "appliers_serialization.33_value",
        UberIdentifier::new(11, 33),
    ),
    (
        "appliers_serialization.34_id",
        UberIdentifier::new(11, 34),
    ),
    (
        "appliers_serialization.35_value",
        UberIdentifier::new(11, 35),
    ),
    (
        "appliers_serialization.36_id",
        UberIdentifier::new(11, 36),
    ),
    (
        "appliers_serialization.37_value",
        UberIdentifier::new(11, 37),
    ),
    (
        "appliers_serialization.38_id",
        UberIdentifier::new(11, 38),
    ),
    (
        "appliers_serialization.39_value",
        UberIdentifier::new(11, 39),
    ),
    (
        "appliers_serialization.40_id",
        UberIdentifier::new(11, 40),
    ),
    (
        "appliers_serialization.41_value",
        UberIdentifier::new(11, 41),
    ),
    (
        "appliers_serialization.42_id",
        UberIdentifier::new(11, 42),
    ),
    (
        "appliers_serialization.43_value",
        UberIdentifier::new(11, 43),
    ),
    (
        "appliers_serialization.44_id",
        UberIdentifier::new(11, 44),
    ),
    (
        "appliers_serialization.45_value",
        UberIdentifier::new(11, 45),
    ),
    (
        "appliers_serialization.46_id",
        UberIdentifier::new(11, 46),
    ),
    (
        "appliers_serialization.47_value",
        UberIdentifier::new(11, 47),
    ),
    (
        "appliers_serialization.48_id",
        UberIdentifier::new(11, 48),
    ),
    (
        "appliers_serialization.49_value",
        UberIdentifier::new(11, 49),
    ),
    (
        "appliers_serialization.50_id",
        UberIdentifier::new(11, 50),
    ),
    (
        "appliers_serialization.51_value",
        UberIdentifier::new(11, 51),
    ),
    (
        "appliers_serialization.52_id",
        UberIdentifier::new(11, 52),
    ),
    (
        "appliers_serialization.53_value",
        UberIdentifier::new(11, 53),
    ),
    (
        "appliers_serialization.54_id",
        UberIdentifier::new(11, 54),
    ),
    (
        "appliers_serialization.55_value",
        UberIdentifier::new(11, 55),
    ),
    (
        "appliers_serialization.56_id",
        UberIdentifier::new(11, 56),
    ),
    (
        "appliers_serialization.57_value",
        UberIdentifier::new(11, 57),
    ),
    (
        "appliers_serialization.58_id",
        UberIdentifier::new(11, 58),
    ),
    (
        "appliers_serialization.59_value",
        UberIdentifier::new(11, 59),
    ),
    (
        "appliers_serialization.60_id",
        UberIdentifier::new(11, 60),
    ),
    (
        "appliers_serialization.61_value",
        UberIdentifier::new(11, 61),
    ),
    (
        "appliers_serialization.62_id",
        UberIdentifier::new(11, 62),
    ),
    (
        "appliers_serialization.63_value",
        UberIdentifier::new(11, 63),
    ),
    (
        "appliers_serialization.64_id",
        UberIdentifier::new(11, 64),
    ),
    (
        "appliers_serialization.65_value",
        UberIdentifier::new(11, 65),
    ),
    (
        "appliers_serialization.66_id",
        UberIdentifier::new(11, 66),
    ),
    (
        "appliers_serialization.67_value",
        UberIdentifier::new(11, 67),
    ),
    (
        "appliers_serialization.68_id",
        UberIdentifier::new(11, 68),
    ),
    (
        "appliers_serialization.69_value",
        UberIdentifier::new(11, 69),
    ),
    (
        "appliers_serialization.70_id",
        UberIdentifier::new(11, 70),
    ),
    (
        "appliers_serialization.71_value",
        UberIdentifier::new(11, 71),
    ),
    (
        "appliers_serialization.72_id",
        UberIdentifier::new(11, 72),
    ),
    (
        "appliers_serialization.73_value",
        UberIdentifier::new(11, 73),
    ),
    (
        "appliers_serialization.74_id",
        UberIdentifier::new(11, 74),
    ),
    (
        "appliers_serialization.75_value",
        UberIdentifier::new(11, 75),
    ),
    (
        "appliers_serialization.76_id",
        UberIdentifier::new(11, 76),
    ),
    (
        "appliers_serialization.77_value",
        UberIdentifier::new(11, 77),
    ),
    (
        "appliers_serialization.78_id",
        UberIdentifier::new(11, 78),
    ),
    (
        "appliers_serialization.79_value",
        UberIdentifier::new(11, 79),
    ),
    (
        "appliers_serialization.80_id",
        UberIdentifier::new(11, 80),
    ),
    (
        "appliers_serialization.81_value",
        UberIdentifier::new(11, 81),
    ),
    (
        "appliers_serialization.82_id",
        UberIdentifier::new(11, 82),
    ),
    (
        "appliers_serialization.83_value",
        UberIdentifier::new(11, 83),
    ),
    (
        "appliers_serialization.84_id",
        UberIdentifier::new(11, 84),
    ),
    (
        "appliers_serialization.85_value",
        UberIdentifier::new(11, 85),
    ),
    (
        "appliers_serialization.86_id",
        UberIdentifier::new(11, 86),
    ),
    (
        "appliers_serialization.87_value",
        UberIdentifier::new(11, 87),
    ),
    (
        "appliers_serialization.88_id",
        UberIdentifier::new(11, 88),
    ),
    (
        "appliers_serialization.89_value",
        UberIdentifier::new(11, 89),
    ),
    (
        "appliers_serialization.90_id",
        UberIdentifier::new(11, 90),
    ),
    (
        "appliers_serialization.91_value",
        UberIdentifier::new(11, 91),
    ),
    (
        "appliers_serialization.92_id",
        UberIdentifier::new(11, 92),
    ),
    (
        "appliers_serialization.93_value",
        UberIdentifier::new(11, 93),
    ),
    (
        "appliers_serialization.94_id",
        UberIdentifier::new(11, 94),
    ),
    (
        "appliers_serialization.95_value",
        UberIdentifier::new(11, 95),
    ),
    (
        "appliers_serialization.96_id",
        UberIdentifier::new(11, 96),
    ),
    (
        "appliers_serialization.97_value",
        UberIdentifier::new(11, 97),
    ),
    (
        "appliers_serialization.98_id",
        UberIdentifier::new(11, 98),
    ),
    (
        "appliers_serialization.99_value",
        UberIdentifier::new(11, 99),
    ),
    (
        "multi_vars.0_multi",
        UberIdentifier::new(12, 0),
    ),
    (
        "multi_vars.1_multi",
        UberIdentifier::new(12, 1),
    ),
    (
        "multi_vars.2_multi",
        UberIdentifier::new(12, 2),
    ),
    (
        "multi_vars.3_multi",
        UberIdentifier::new(12, 3),
    ),
    (
        "multi_vars.4_multi",
        UberIdentifier::new(12, 4),
    ),
    (
        "multi_vars.5_multi",
        UberIdentifier::new(12, 5),
    ),
    (
        "multi_vars.6_multi",
        UberIdentifier::new(12, 6),
    ),
    (
        "multi_vars.7_multi",
        UberIdentifier::new(12, 7),
    ),
    (
        "multi_vars.8_multi",
        UberIdentifier::new(12, 8),
    ),
    (
        "multi_vars.9_multi",
        UberIdentifier::new(12, 9),
    ),
    (
        "multi_vars.10_multi",
        UberIdentifier::new(12, 10),
    ),
    (
        "multi_vars.11_multi",
        UberIdentifier::new(12, 11),
    ),
    (
        "multi_vars.12_multi",
        UberIdentifier::new(12, 12),
    ),
    (
        "multi_vars.13_multi",
        UberIdentifier::new(12, 13),
    ),
    (
        "multi_vars.14_multi",
        UberIdentifier::new(12, 14),
    ),
    (
        "multi_vars.15_multi",
        UberIdentifier::new(12, 15),
    ),
    (
        "multi_vars.16_multi",
        UberIdentifier::new(12, 16),
    ),
    (
        "multi_vars.17_multi",
        UberIdentifier::new(12, 17),
    ),
    (
        "multi_vars.18_multi",
        UberIdentifier::new(12, 18),
    ),
    (
        "multi_vars.19_multi",
        UberIdentifier::new(12, 19),
    ),
    (
        "multi_vars.20_multi",
        UberIdentifier::new(12, 20),
    ),
    (
        "multi_vars.21_multi",
        UberIdentifier::new(12, 21),
    ),
    (
        "multi_vars.22_multi",
        UberIdentifier::new(12, 22),
    ),
    (
        "multi_vars.23_multi",
        UberIdentifier::new(12, 23),
    ),
    (
        "multi_vars.24_multi",
        UberIdentifier::new(12, 24),
    ),
    (
        "multi_vars.25_multi",
        UberIdentifier::new(12, 25),
    ),
    (
        "multi_vars.26_multi",
        UberIdentifier::new(12, 26),
    ),
    (
        "multi_vars.27_multi",
        UberIdentifier::new(12, 27),
    ),
    (
        "multi_vars.28_multi",
        UberIdentifier::new(12, 28),
    ),
    (
        "multi_vars.29_multi",
        UberIdentifier::new(12, 29),
    ),
    (
        "multi_vars.30_multi",
        UberIdentifier::new(12, 30),
    ),
    (
        "multi_vars.31_multi",
        UberIdentifier::new(12, 31),
    ),
    (
        "multi_vars.32_multi",
        UberIdentifier::new(12, 32),
    ),
    (
        "multi_vars.33_multi",
        UberIdentifier::new(12, 33),
    ),
    (
        "multi_vars.34_multi",
        UberIdentifier::new(12, 34),
    ),
    (
        "multi_vars.35_multi",
        UberIdentifier::new(12, 35),
    ),
    (
        "multi_vars.36_multi",
        UberIdentifier::new(12, 36),
    ),
    (
        "multi_vars.37_multi",
        UberIdentifier::new(12, 37),
    ),
    (
        "multi_vars.38_multi",
        UberIdentifier::new(12, 38),
    ),
    (
        "multi_vars.39_multi",
        UberIdentifier::new(12, 39),
    ),
    (
        "multi_vars.40_multi",
        UberIdentifier::new(12, 40),
    ),
    (
        "multi_vars.41_multi",
        UberIdentifier::new(12, 41),
    ),
    (
        "multi_vars.42_multi",
        UberIdentifier::new(12, 42),
    ),
    (
        "multi_vars.43_multi",
        UberIdentifier::new(12, 43),
    ),
    (
        "multi_vars.44_multi",
        UberIdentifier::new(12, 44),
    ),
    (
        "multi_vars.45_multi",
        UberIdentifier::new(12, 45),
    ),
    (
        "multi_vars.46_multi",
        UberIdentifier::new(12, 46),
    ),
    (
        "multi_vars.47_multi",
        UberIdentifier::new(12, 47),
    ),
    (
        "multi_vars.48_multi",
        UberIdentifier::new(12, 48),
    ),
    (
        "multi_vars.49_multi",
        UberIdentifier::new(12, 49),
    ),
    (
        "multi_vars.50_multi",
        UberIdentifier::new(12, 50),
    ),
    (
        "multi_vars.51_multi",
        UberIdentifier::new(12, 51),
    ),
    (
        "multi_vars.52_multi",
        UberIdentifier::new(12, 52),
    ),
    (
        "multi_vars.53_multi",
        UberIdentifier::new(12, 53),
    ),
    (
        "multi_vars.54_multi",
        UberIdentifier::new(12, 54),
    ),
    (
        "multi_vars.55_multi",
        UberIdentifier::new(12, 55),
    ),
    (
        "multi_vars.56_multi",
        UberIdentifier::new(12, 56),
    ),
    (
        "multi_vars.57_multi",
        UberIdentifier::new(12, 57),
    ),
    (
        "multi_vars.58_multi",
        UberIdentifier::new(12, 58),
    ),
    (
        "multi_vars.59_multi",
        UberIdentifier::new(12, 59),
    ),
    (
        "multi_vars.60_multi",
        UberIdentifier::new(12, 60),
    ),
    (
        "multi_vars.61_multi",
        UberIdentifier::new(12, 61),
    ),
    (
        "multi_vars.62_multi",
        UberIdentifier::new(12, 62),
    ),
    (
        "multi_vars.63_multi",
        UberIdentifier::new(12, 63),
    ),
    (
        "multi_vars.64_multi",
        UberIdentifier::new(12, 64),
    ),
    (
        "multi_vars.65_multi",
        UberIdentifier::new(12, 65),
    ),
    (
        "multi_vars.66_multi",
        UberIdentifier::new(12, 66),
    ),
    (
        "multi_vars.67_multi",
        UberIdentifier::new(12, 67),
    ),
    (
        "multi_vars.68_multi",
        UberIdentifier::new(12, 68),
    ),
    (
        "multi_vars.69_multi",
        UberIdentifier::new(12, 69),
    ),
    (
        "multi_vars.70_multi",
        UberIdentifier::new(12, 70),
    ),
    (
        "multi_vars.71_multi",
        UberIdentifier::new(12, 71),
    ),
    (
        "multi_vars.72_multi",
        UberIdentifier::new(12, 72),
    ),
    (
        "multi_vars.73_multi",
        UberIdentifier::new(12, 73),
    ),
    (
        "multi_vars.74_multi",
        UberIdentifier::new(12, 74),
    ),
    (
        "multi_vars.75_multi",
        UberIdentifier::new(12, 75),
    ),
    (
        "multi_vars.76_multi",
        UberIdentifier::new(12, 76),
    ),
    (
        "multi_vars.77_multi",
        UberIdentifier::new(12, 77),
    ),
    (
        "multi_vars.78_multi",
        UberIdentifier::new(12, 78),
    ),
    (
        "multi_vars.79_multi",
        UberIdentifier::new(12, 79),
    ),
    (
        "multi_vars.80_multi",
        UberIdentifier::new(12, 80),
    ),
    (
        "multi_vars.81_multi",
        UberIdentifier::new(12, 81),
    ),
    (
        "multi_vars.82_multi",
        UberIdentifier::new(12, 82),
    ),
    (
        "multi_vars.83_multi",
        UberIdentifier::new(12, 83),
    ),
    (
        "multi_vars.84_multi",
        UberIdentifier::new(12, 84),
    ),
    (
        "multi_vars.85_multi",
        UberIdentifier::new(12, 85),
    ),
    (
        "multi_vars.86_multi",
        UberIdentifier::new(12, 86),
    ),
    (
        "multi_vars.87_multi",
        UberIdentifier::new(12, 87),
    ),
    (
        "multi_vars.88_multi",
        UberIdentifier::new(12, 88),
    ),
    (
        "multi_vars.89_multi",
        UberIdentifier::new(12, 89),
    ),
    (
        "multi_vars.90_multi",
        UberIdentifier::new(12, 90),
    ),
    (
        "multi_vars.91_multi",
        UberIdentifier::new(12, 91),
    ),
    (
        "multi_vars.92_multi",
        UberIdentifier::new(12, 92),
    ),
    (
        "multi_vars.93_multi",
        UberIdentifier::new(12, 93),
    ),
    (
        "multi_vars.94_multi",
        UberIdentifier::new(12, 94),
    ),
    (
        "multi_vars.95_multi",
        UberIdentifier::new(12, 95),
    ),
    (
        "multi_vars.96_multi",
        UberIdentifier::new(12, 96),
    ),
    (
        "multi_vars.97_multi",
        UberIdentifier::new(12, 97),
    ),
    (
        "multi_vars.98_multi",
        UberIdentifier::new(12, 98),
    ),
    (
        "multi_vars.99_multi",
        UberIdentifier::new(12, 99),
    ),
    (
        "rando_stats.Deaths",
        UberIdentifier::new(14, 101),
    ),
    (
        "rando_stats.warps used",
        UberIdentifier::new(14, 106),
    ),
    (
        "rando_stats.Peak PPM count",
        UberIdentifier::new(14, 108),
    ),
    (
        "rando_stats.Marsh Time",
        UberIdentifier::new(14, 0),
    ),
    (
        "rando_stats.Hollow Time",
        UberIdentifier::new(14, 1),
    ),
    (
        "rando_stats.Glades Time",
        UberIdentifier::new(14, 2),
    ),
    (
        "rando_stats.Wellspring Time",
        UberIdentifier::new(14, 3),
    ),
    (
        "rando_stats.Burrows Time",
        UberIdentifier::new(14, 4),
    ),
    (
        "rando_stats.Woods Time",
        UberIdentifier::new(14, 5),
    ),
    (
        "rando_stats.Reach Time",
        UberIdentifier::new(14, 6),
    ),
    (
        "rando_stats.Pools Time",
        UberIdentifier::new(14, 7),
    ),
    (
        "rando_stats.Depths Time",
        UberIdentifier::new(14, 8),
    ),
    (
        "rando_stats.Wastes Time",
        UberIdentifier::new(14, 9),
    ),
    (
        "rando_stats.Ruins Time",
        UberIdentifier::new(14, 10),
    ),
    (
        "rando_stats.Willow Time",
        UberIdentifier::new(14, 11),
    ),
    (
        "rando_stats.Void Time",
        UberIdentifier::new(14, 12),
    ),
    (
        "rando_stats.Time",
        UberIdentifier::new(14, 100),
    ),
    (
        "rando_stats.Current Drought",
        UberIdentifier::new(14, 102),
    ),
    (
        "rando_stats.Longest Drought",
        UberIdentifier::new(14, 103),
    ),
    (
        "rando_stats.Time since last checkpoint",
        UberIdentifier::new(14, 104),
    ),
    (
        "rando_stats.Time lost to deaths",
        UberIdentifier::new(14, 105),
    ),
    (
        "rando_stats.Peak PPM time",
        UberIdentifier::new(14, 107),
    ),
    (
        "rando_stats.Marsh Deaths",
        UberIdentifier::new(14, 20),
    ),
    (
        "rando_stats.Hollow Deaths",
        UberIdentifier::new(14, 21),
    ),
    (
        "rando_stats.Glades Deaths",
        UberIdentifier::new(14, 22),
    ),
    (
        "rando_stats.Wellspring Deaths",
        UberIdentifier::new(14, 23),
    ),
    (
        "rando_stats.Burrows Deaths",
        UberIdentifier::new(14, 24),
    ),
    (
        "rando_stats.Woods Deaths",
        UberIdentifier::new(14, 25),
    ),
    (
        "rando_stats.Reach Deaths",
        UberIdentifier::new(14, 26),
    ),
    (
        "rando_stats.Pools Deaths",
        UberIdentifier::new(14, 27),
    ),
    (
        "rando_stats.Depths Deaths",
        UberIdentifier::new(14, 28),
    ),
    (
        "rando_stats.Wastes Deaths",
        UberIdentifier::new(14, 29),
    ),
    (
        "rando_stats.Ruins Deaths",
        UberIdentifier::new(14, 30),
    ),
    (
        "rando_stats.Willow Deaths",
        UberIdentifier::new(14, 31),
    ),
    (
        "rando_stats.Void Deaths",
        UberIdentifier::new(14, 32),
    ),
    (
        "rando_stats.Marsh Pickups",
        UberIdentifier::new(14, 40),
    ),
    (
        "rando_stats.Hollow Pickups",
        UberIdentifier::new(14, 41),
    ),
    (
        "rando_stats.Glades Pickups",
        UberIdentifier::new(14, 42),
    ),
    (
        "rando_stats.Wellspring Pickups",
        UberIdentifier::new(14, 43),
    ),
    (
        "rando_stats.Burrows Pickups",
        UberIdentifier::new(14, 44),
    ),
    (
        "rando_stats.Woods Pickups",
        UberIdentifier::new(14, 45),
    ),
    (
        "rando_stats.Reach Pickups",
        UberIdentifier::new(14, 46),
    ),
    (
        "rando_stats.Pools Pickups",
        UberIdentifier::new(14, 47),
    ),
    (
        "rando_stats.Depths Pickups",
        UberIdentifier::new(14, 48),
    ),
    (
        "rando_stats.Wastes Pickups",
        UberIdentifier::new(14, 49),
    ),
    (
        "rando_stats.Ruins Pickups",
        UberIdentifier::new(14, 50),
    ),
    (
        "rando_stats.Willow Pickups",
        UberIdentifier::new(14, 51),
    ),
    (
        "rando_stats.Void Pickups",
        UberIdentifier::new(14, 52),
    ),
    (
        "animalCutsceneGroupDescriptor.animalCutsceneDoneUberState",
        UberIdentifier::new(192, 80),
    ),
    (
        "kwoloksGroupDescriptor.leafPileB",
        UberIdentifier::new(195, 56127),
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorBOpen",
        UberIdentifier::new(229, 2),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalOBurning",
        UberIdentifier::new(229, 27),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalFBurning",
        UberIdentifier::new(229, 30),
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorAOpen",
        UberIdentifier::new(229, 35),
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorFOpen",
        UberIdentifier::new(229, 49),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalEBurning",
        UberIdentifier::new(229, 52),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalABurning",
        UberIdentifier::new(229, 62),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalIBurning",
        UberIdentifier::new(229, 66),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalNBurning",
        UberIdentifier::new(229, 71),
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorEOpen",
        UberIdentifier::new(229, 76),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalGBurning",
        UberIdentifier::new(229, 108),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalMBurning",
        UberIdentifier::new(229, 119),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalCBurning",
        UberIdentifier::new(229, 129),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalKBurning",
        UberIdentifier::new(229, 174),
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorCOpen",
        UberIdentifier::new(229, 185),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalHBurning",
        UberIdentifier::new(229, 210),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalJBurning",
        UberIdentifier::new(229, 222),
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorDOpen",
        UberIdentifier::new(229, 226),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalDBurning",
        UberIdentifier::new(229, 231),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalBBurning",
        UberIdentifier::new(229, 233),
    ),
    (
        "_riverlandsGroup.riverlands_pedestalLBurning",
        UberIdentifier::new(229, 237),
    ),
    (
        "_riverlandsGroup.savePedestalUberState",
        UberIdentifier::new(229, 41675),
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier::new(937, 109),
    ),
    (
        "kwolokGroupDescriptor.watermillDoor",
        UberIdentifier::new(937, 749),
    ),
    (
        "kwolokGroupDescriptor.cavernGLeverAndDoor",
        UberIdentifier::new(937, 1174),
    ),
    (
        "kwolokGroupDescriptor.halfHealthCell",
        UberIdentifier::new(937, 2463),
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier::new(937, 2538),
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop04",
        UberIdentifier::new(937, 4057),
    ),
    (
        "kwolokGroupDescriptor.xpOrbB",
        UberIdentifier::new(937, 5568),
    ),
    (
        "kwolokGroupDescriptor.energyContainerA",
        UberIdentifier::new(937, 5668),
    ),
    (
        "kwolokGroupDescriptor.orePickup",
        UberIdentifier::new(937, 6703),
    ),
    (
        "kwolokGroupDescriptor.mokiGateOpened",
        UberIdentifier::new(937, 6778),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokF",
        UberIdentifier::new(937, 7119),
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier::new(937, 7153),
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop01",
        UberIdentifier::new(937, 7941),
    ),
    (
        "kwolokGroupDescriptor.energyHalfCell",
        UberIdentifier::new(937, 8518),
    ),
    (
        "kwolokGroupDescriptor.secretWallA",
        UberIdentifier::new(937, 10140),
    ),
    (
        "kwolokGroupDescriptor.gromOreA",
        UberIdentifier::new(937, 10729),
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier::new(937, 10877),
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier::new(937, 11430),
    ),
    (
        "kwolokGroupDescriptor.bombableWallAkwoloksCavernEa",
        UberIdentifier::new(937, 11610),
    ),
    (
        "kwolokGroupDescriptor.orePickupB",
        UberIdentifier::new(937, 11846),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokH",
        UberIdentifier::new(937, 12458),
    ),
    (
        "kwolokGroupDescriptor.stepsRisen",
        UberIdentifier::new(937, 13273),
    ),
    (
        "kwolokGroupDescriptor.xpOrbA",
        UberIdentifier::new(937, 13413),
    ),
    (
        "kwolokGroupDescriptor.smallExpA",
        UberIdentifier::new(937, 15993),
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier::new(937, 16163),
    ),
    (
        "kwolokGroupDescriptor.energyContainerA",
        UberIdentifier::new(937, 17761),
    ),
    (
        "kwolokGroupDescriptor.temp_WispQuestStandIn",
        UberIdentifier::new(937, 18035),
    ),
    (
        "kwolokGroupDescriptor.smallExpA",
        UberIdentifier::new(937, 18103),
    ),
    (
        "kwolokGroupDescriptor.mediumExpC",
        UberIdentifier::new(937, 19529),
    ),
    (
        "kwolokGroupDescriptor.smallExpB",
        UberIdentifier::new(937, 20219),
    ),
    (
        "kwolokGroupDescriptor.bombableWallAkwoloksCavernE",
        UberIdentifier::new(937, 20294),
    ),
    (
        "kwolokGroupDescriptor.kwolokCavernsGate",
        UberIdentifier::new(937, 21165),
    ),
    (
        "kwolokGroupDescriptor.kwolokShrineBreakableWall",
        UberIdentifier::new(937, 22302),
    ),
    (
        "kwolokGroupDescriptor.leverDoor",
        UberIdentifier::new(937, 22419),
    ),
    (
        "kwolokGroupDescriptor.pressurePlatePuzzle",
        UberIdentifier::new(937, 22716),
    ),
    (
        "kwolokGroupDescriptor.lifeCellA",
        UberIdentifier::new(937, 23486),
    ),
    (
        "kwolokGroupDescriptor.energyHalfCell",
        UberIdentifier::new(937, 23772),
    ),
    (
        "kwolokGroupDescriptor.spiritShardA",
        UberIdentifier::new(937, 24039),
    ),
    (
        "kwolokGroupDescriptor.energyHalfContainer",
        UberIdentifier::new(937, 24175),
    ),
    (
        "kwolokGroupDescriptor.spiritShardMagnet",
        UberIdentifier::new(937, 25413),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokC",
        UberIdentifier::new(937, 27481),
    ),
    (
        "kwolokGroupDescriptor.hornbugWallBroken",
        UberIdentifier::new(937, 27671),
    ),
    (
        "kwolokGroupDescriptor.mediumExpB",
        UberIdentifier::new(937, 30182),
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop01",
        UberIdentifier::new(937, 30594),
    ),
    (
        "kwolokGroupDescriptor.breakableWallA",
        UberIdentifier::new(937, 31026),
    ),
    (
        "kwolokGroupDescriptor.mediumExpC",
        UberIdentifier::new(937, 31036),
    ),
    (
        "kwolokGroupDescriptor.switchTop",
        UberIdentifier::new(937, 31222),
    ),
    (
        "kwolokGroupDescriptor.door",
        UberIdentifier::new(937, 32165),
    ),
    (
        "kwolokGroupDescriptor.areaText",
        UberIdentifier::new(937, 32175),
    ),
    (
        "kwolokGroupDescriptor.drillZoneA",
        UberIdentifier::new(937, 32452),
    ),
    (
        "kwolokGroupDescriptor.keyStoneD",
        UberIdentifier::new(937, 33763),
    ),
    (
        "kwolokGroupDescriptor.winterForestDoor",
        UberIdentifier::new(937, 33773),
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop03",
        UberIdentifier::new(937, 34340),
    ),
    (
        "kwolokGroupDescriptor.airDashHint",
        UberIdentifier::new(937, 34343),
    ),
    (
        "kwolokGroupDescriptor.leverDoorKwoloksHollowEntrance",
        UberIdentifier::new(937, 34396),
    ),
    (
        "kwolokGroupDescriptor.haveSpokenToOtters",
        UberIdentifier::new(937, 34516),
    ),
    (
        "kwolokGroupDescriptor.frogTongueA",
        UberIdentifier::new(937, 34849),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokI",
        UberIdentifier::new(937, 37823),
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier::new(937, 37926),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokD",
        UberIdentifier::new(937, 39338),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokE",
        UberIdentifier::new(937, 39661),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokG",
        UberIdentifier::new(937, 39715),
    ),
    (
        "kwolokGroupDescriptor.secretWallB",
        UberIdentifier::new(937, 40042),
    ),
    (
        "kwolokGroupDescriptor.stompableFloor",
        UberIdentifier::new(937, 40225),
    ),
    (
        "kwolokGroupDescriptor.mediumExpD",
        UberIdentifier::new(937, 40298),
    ),
    (
        "kwolokGroupDescriptor.secretWallKwolok",
        UberIdentifier::new(937, 40466),
    ),
    (
        "kwolokGroupDescriptor.smallExpA",
        UberIdentifier::new(937, 40657),
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholderB",
        UberIdentifier::new(937, 42333),
    ),
    (
        "kwolokGroupDescriptor.interactedWithTokk",
        UberIdentifier::new(937, 42585),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolok",
        UberIdentifier::new(937, 44594),
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop03",
        UberIdentifier::new(937, 44861),
    ),
    (
        "kwolokGroupDescriptor.stompableFloorB",
        UberIdentifier::new(937, 45349),
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier::new(937, 45625),
    ),
    (
        "kwolokGroupDescriptor.mediumExpB",
        UberIdentifier::new(937, 45744),
    ),
    (
        "kwolokGroupDescriptor.secretWallA",
        UberIdentifier::new(937, 45811),
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier::new(937, 45987),
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop02",
        UberIdentifier::new(937, 47364),
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholderA",
        UberIdentifier::new(937, 48192),
    ),
    (
        "kwolokGroupDescriptor.desertBombableWall",
        UberIdentifier::new(937, 49545),
    ),
    (
        "kwolokGroupDescriptor.xpOrbA",
        UberIdentifier::new(937, 50176),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokB",
        UberIdentifier::new(937, 50357),
    ),
    (
        "kwolokGroupDescriptor.secretWallA",
        UberIdentifier::new(937, 50474),
    ),
    (
        "kwolokGroupDescriptor.energyContainerPlaceholder",
        UberIdentifier::new(937, 50615),
    ),
    (
        "kwolokGroupDescriptor.breakableWallA",
        UberIdentifier::new(937, 51878),
    ),
    (
        "kwolokGroupDescriptor.kwolokBossBridgeBroken",
        UberIdentifier::new(937, 51919),
    ),
    (
        "kwolokGroupDescriptor.spiritShardPickupPlaceholder",
        UberIdentifier::new(937, 52258),
    ),
    (
        "kwolokGroupDescriptor.frogDoor",
        UberIdentifier::new(937, 52652),
    ),
    (
        "kwolokGroupDescriptor.hornBugBossDefeatedState",
        UberIdentifier::new(937, 53122),
    ),
    (
        "kwolokGroupDescriptor.bombableWallAkwoloksCavernEb",
        UberIdentifier::new(937, 53969),
    ),
    (
        "kwolokGroupDescriptor.keyStoneC",
        UberIdentifier::new(937, 54102),
    ),
    (
        "kwolokGroupDescriptor.brokenWallA",
        UberIdentifier::new(937, 54236),
    ),
    (
        "kwolokGroupDescriptor.risingPedestals",
        UberIdentifier::new(937, 54318),
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop02",
        UberIdentifier::new(937, 55341),
    ),
    (
        "kwolokGroupDescriptor.dashHint",
        UberIdentifier::new(937, 55538),
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokJ",
        UberIdentifier::new(937, 56352),
    ),
    (
        "kwolokGroupDescriptor.frogTongueC",
        UberIdentifier::new(937, 56795),
    ),
    (
        "kwolokGroupDescriptor.leverDoorA",
        UberIdentifier::new(937, 57028),
    ),
    (
        "kwolokGroupDescriptor.healthHalfCell",
        UberIdentifier::new(937, 58598),
    ),
    (
        "kwolokGroupDescriptor.shootableTargetDoor",
        UberIdentifier::new(937, 58747),
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop05",
        UberIdentifier::new(937, 59404),
    ),
    (
        "kwolokGroupDescriptor.serializedBooleanUberState",
        UberIdentifier::new(937, 59515),
    ),
    (
        "kwolokGroupDescriptor.doorA",
        UberIdentifier::new(937, 59850),
    ),
    (
        "kwolokGroupDescriptor.switchDoorUberState",
        UberIdentifier::new(937, 59920),
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop04",
        UberIdentifier::new(937, 61099),
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier::new(937, 61460),
    ),
    (
        "kwolokGroupDescriptor.hornbugIntroArenaUberState",
        UberIdentifier::new(937, 61633),
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholderC",
        UberIdentifier::new(937, 61744),
    ),
    (
        "kwolokGroupDescriptor.xpOrbC",
        UberIdentifier::new(937, 61783),
    ),
    (
        "kwolokGroupDescriptor.healthHalfCell",
        UberIdentifier::new(937, 61897),
    ),
    (
        "kwolokGroupDescriptor.entranceStatueOpened",
        UberIdentifier::new(937, 64003),
    ),
    (
        "kwolokGroupDescriptor.spiritShardPickupPlaceholderB",
        UberIdentifier::new(937, 64146),
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier::new(937, 65195),
    ),
    (
        "kwolokGroupDescriptor.savePedestal",
        UberIdentifier::new(937, 5281),
    ),
    (
        "kwolokGroupDescriptor.savePedestal",
        UberIdentifier::new(937, 26601),
    ),
    (
        "kwolokGroupDescriptor.kwolokNpcState",
        UberIdentifier::new(937, 10071),
    ),
    (
        "kwolokGroupDescriptor.cleanseWellspringQuestUberState",
        UberIdentifier::new(937, 34641),
    ),
    (
        "kwolokGroupDescriptor.recedingWaterSetup",
        UberIdentifier::new(937, 42245),
    ),
    (
        "kwolokGroupDescriptor.shardTraderState",
        UberIdentifier::new(937, 47836),
    ),
    (
        "kwolokGroupDescriptor.hornBugBossState",
        UberIdentifier::new(937, 48534),
    ),
    (
        "kwolokGroupDescriptor.recedingWaterSetupJordi",
        UberIdentifier::new(937, 52814),
    ),
    (
        "kwolokGroupDescriptor.healthPlantTimer",
        UberIdentifier::new(937, 14501),
    ),
    (
        "kwolokGroupDescriptor.healthPlant",
        UberIdentifier::new(937, 15130),
    ),
    (
        "kwolokGroupDescriptor.eyesPlacedIntoStatue",
        UberIdentifier::new(937, 1038),
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomBottom",
        UberIdentifier::new(937, 6040),
    ),
    (
        "kwolokGroupDescriptor.frogTongueB",
        UberIdentifier::new(937, 12557),
    ),
    (
        "kwolokGroupDescriptor.retractTongue",
        UberIdentifier::new(937, 13557),
    ),
    (
        "kwolokGroupDescriptor.frogTongueE",
        UberIdentifier::new(937, 14026),
    ),
    (
        "kwolokGroupDescriptor.frogCavernGPuzzleLeftUp",
        UberIdentifier::new(937, 19495),
    ),
    (
        "kwolokGroupDescriptor.frogCavernBRight",
        UberIdentifier::new(937, 24510),
    ),
    (
        "kwolokGroupDescriptor.frogTop01",
        UberIdentifier::new(937, 28504),
    ),
    (
        "kwolokGroupDescriptor.attackableFrogByteUberState",
        UberIdentifier::new(937, 30661),
    ),
    (
        "kwolokGroupDescriptor.kwolokCavernsAttackableToad",
        UberIdentifier::new(937, 32948),
    ),
    (
        "kwolokGroupDescriptor.frogBottom03",
        UberIdentifier::new(937, 37928),
    ),
    (
        "kwolokGroupDescriptor.frogCavernFBottom",
        UberIdentifier::new(937, 38183),
    ),
    (
        "kwolokGroupDescriptor.frogCavernGPuzzleRight",
        UberIdentifier::new(937, 40810),
    ),
    (
        "kwolokGroupDescriptor.frogCavernBLeft",
        UberIdentifier::new(937, 41587),
    ),
    (
        "kwolokGroupDescriptor.frogCavernELeft",
        UberIdentifier::new(937, 44452),
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomTopC",
        UberIdentifier::new(937, 45423),
    ),
    (
        "kwolokGroupDescriptor.frogBottom02",
        UberIdentifier::new(937, 49392),
    ),
    (
        "kwolokGroupDescriptor.kwolokCavernsAttackableToadB",
        UberIdentifier::new(937, 49874),
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomTopA",
        UberIdentifier::new(937, 50411),
    ),
    (
        "kwolokGroupDescriptor.frogCavernGPuzzleLeft",
        UberIdentifier::new(937, 50803),
    ),
    (
        "kwolokGroupDescriptor.frogTongueD",
        UberIdentifier::new(937, 51234),
    ),
    (
        "kwolokGroupDescriptor.frogCavernERight",
        UberIdentifier::new(937, 53749),
    ),
    (
        "kwolokGroupDescriptor.frogCavernGBottom",
        UberIdentifier::new(937, 56395),
    ),
    (
        "kwolokGroupDescriptor.frogTop02",
        UberIdentifier::new(937, 57711),
    ),
    (
        "kwolokGroupDescriptor.frogTop03",
        UberIdentifier::new(937, 59288),
    ),
    (
        "kwolokGroupDescriptor.frogBottom04",
        UberIdentifier::new(937, 62300),
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomTopB",
        UberIdentifier::new(937, 63347),
    ),
    (
        "kwolokGroupDescriptor.frogCavernBTopRight",
        UberIdentifier::new(937, 63834),
    ),
    (
        "kwolokGroupDescriptor.frogBottom01",
        UberIdentifier::new(937, 64257),
    ),
    (
        "lagoonStateGroup.secretWallA",
        UberIdentifier::new(945, 3487),
    ),
    (
        "lagoonStateGroup.canShowGlideHint",
        UberIdentifier::new(945, 3659),
    ),
    (
        "lagoonStateGroup.mediumExpA",
        UberIdentifier::new(945, 7031),
    ),
    (
        "lagoonStateGroup.breakableWallB",
        UberIdentifier::new(945, 7465),
    ),
    (
        "lagoonStateGroup.kwolokBossBridgeBreak",
        UberIdentifier::new(945, 9034),
    ),
    (
        "lagoonStateGroup.wispSequencePlayedOut",
        UberIdentifier::new(945, 9367),
    ),
    (
        "lagoonStateGroup.mediumExpB",
        UberIdentifier::new(945, 10682),
    ),
    (
        "lagoonStateGroup.largeExpC",
        UberIdentifier::new(945, 10833),
    ),
    (
        "lagoonStateGroup.tentacleKilled",
        UberIdentifier::new(945, 12852),
    ),
    (
        "lagoonStateGroup.mediumExpA",
        UberIdentifier::new(945, 14530),
    ),
    (
        "lagoonStateGroup.energyCellA",
        UberIdentifier::new(945, 21334),
    ),
    (
        "lagoonStateGroup.memoriesPlayed",
        UberIdentifier::new(945, 25182),
    ),
    (
        "lagoonStateGroup.energyContainerA",
        UberIdentifier::new(945, 25520),
    ),
    (
        "lagoonStateGroup.breakableWallA",
        UberIdentifier::new(945, 28631),
    ),
    (
        "lagoonStateGroup.mediumExpA",
        UberIdentifier::new(945, 32890),
    ),
    (
        "lagoonStateGroup.displayedGlideHint",
        UberIdentifier::new(945, 33930),
    ),
    (
        "lagoonStateGroup.lagoonMillTransitionHealthCell",
        UberIdentifier::new(945, 37243),
    ),
    (
        "lagoonStateGroup.mediumExpB",
        UberIdentifier::new(945, 38319),
    ),
    (
        "lagoonStateGroup.breakableWallB",
        UberIdentifier::new(945, 39004),
    ),
    (
        "lagoonStateGroup.secretWallB",
        UberIdentifier::new(945, 43451),
    ),
    (
        "lagoonStateGroup.bossReward",
        UberIdentifier::new(945, 49747),
    ),
    (
        "lagoonStateGroup.breakableWallA",
        UberIdentifier::new(945, 55795),
    ),
    (
        "lagoonStateGroup.medExpA",
        UberIdentifier::new(945, 58723),
    ),
    (
        "lagoonStateGroup.savePedestalUberState",
        UberIdentifier::new(945, 1370),
    ),
    (
        "lagoonStateGroup.savePedestalUberState",
        UberIdentifier::new(945, 58183),
    ),
    (
        "lagoonStateGroup.healthPlantA",
        UberIdentifier::new(945, 23296),
    ),
    (
        "lagoonStateGroup.kwolokBossState",
        UberIdentifier::new(945, 58403),
    ),
    (
        "playerUberStateGroupDescriptor.playerPurchasedWeaponMasterUpgrade",
        UberIdentifier::new(3440, 20131),
    ),
    (
        "playerUberStateGroupDescriptor.playerOnTandemUberState",
        UberIdentifier::new(3440, 54402),
    ),
    (
        "playerUberStateGroupDescriptor.playerWeaponDamageUpgradeLevel",
        UberIdentifier::new(3440, 34448),
    ),
    (
        "playerUberStateGroupDescriptor.hammerSpeedUpgradeLevel",
        UberIdentifier::new(3440, 1157),
    ),
    (
        "playerUberStateGroupDescriptor.chargeWeaponsUpgradeLevel",
        UberIdentifier::new(3440, 2234),
    ),
    (
        "playerUberStateGroupDescriptor.spikeExplosiveUpgradeLevel",
        UberIdentifier::new(3440, 5687),
    ),
    (
        "playerUberStateGroupDescriptor.spellMeditateUpgradeLevel",
        UberIdentifier::new(3440, 9670),
    ),
    (
        "playerUberStateGroupDescriptor.waterBreathUpgradeLevel",
        UberIdentifier::new(3440, 10233),
    ),
    (
        "playerUberStateGroupDescriptor.chakramSpinUpgradeLevel",
        UberIdentifier::new(3440, 10776),
    ),
    (
        "playerUberStateGroupDescriptor.bashSplitUpgradeLevel",
        UberIdentifier::new(3440, 10928),
    ),
    (
        "playerUberStateGroupDescriptor.grenadeDamageUpgradeLevel",
        UberIdentifier::new(3440, 16155),
    ),
    (
        "playerUberStateGroupDescriptor.spellChakramUpgradeLevel",
        UberIdentifier::new(3440, 17265),
    ),
    (
        "playerUberStateGroupDescriptor.missilesDamageUpgradeLevel",
        UberIdentifier::new(3440, 18770),
    ),
    (
        "playerUberStateGroupDescriptor.spellSpikeUpgradeLevel",
        UberIdentifier::new(3440, 24142),
    ),
    (
        "playerUberStateGroupDescriptor.missilesAmountUpgradeLevel",
        UberIdentifier::new(3440, 26998),
    ),
    (
        "playerUberStateGroupDescriptor.bowDamageUpgradeLevel",
        UberIdentifier::new(3440, 29503),
    ),
    (
        "playerUberStateGroupDescriptor.swordComboUpgradeLevel",
        UberIdentifier::new(3440, 30415),
    ),
    (
        "playerUberStateGroupDescriptor.healEfficiencyUpgradeLevel",
        UberIdentifier::new(3440, 31259),
    ),
    (
        "playerUberStateGroupDescriptor.spikeDamageUpgradeLevel",
        UberIdentifier::new(3440, 33963),
    ),
    (
        "playerUberStateGroupDescriptor.spellSentryUpgradeLevel",
        UberIdentifier::new(3440, 38929),
    ),
    (
        "playerUberStateGroupDescriptor.swordDamageUpgradeLevel",
        UberIdentifier::new(3440, 39658),
    ),
    (
        "playerUberStateGroupDescriptor.chakramMagnetUpgradeLevel",
        UberIdentifier::new(3440, 40954),
    ),
    (
        "playerUberStateGroupDescriptor.chakramDamageUpgradeLevel",
        UberIdentifier::new(3440, 42913),
    ),
    (
        "playerUberStateGroupDescriptor.invisibilityDurationUpgradeLevel",
        UberIdentifier::new(3440, 45208),
    ),
    (
        "playerUberStateGroupDescriptor.hammerStompUpgradeLevel",
        UberIdentifier::new(3440, 46488),
    ),
    (
        "playerUberStateGroupDescriptor.sentryAmountUpgradeLevel",
        UberIdentifier::new(3440, 48877),
    ),
    (
        "playerUberStateGroupDescriptor.hammerDamageUpgradeLevel",
        UberIdentifier::new(3440, 53415),
    ),
    (
        "playerUberStateGroupDescriptor.sentrySpeedUpgradeLevel",
        UberIdentifier::new(3440, 57376),
    ),
    (
        "playerUberStateGroupDescriptor.spellBlazeUpgradeLevel",
        UberIdentifier::new(3440, 58703),
    ),
    (
        "playerUberStateGroupDescriptor.blazeChargeUpgradeLevel",
        UberIdentifier::new(3440, 61898),
    ),
    (
        "playerUberStateGroupDescriptor.chakramAmountUpgradeLevel",
        UberIdentifier::new(3440, 62563),
    ),
    (
        "playerUberStateGroupDescriptor.spellHammerUpgradeLevel",
        UberIdentifier::new(3440, 64152),
    ),
    (
        "lumaPoolsStateGroup.largeExpOrbPlaceholderA",
        UberIdentifier::new(5377, 628),
    ),
    (
        "lumaPoolsStateGroup.energyCellFragmentA",
        UberIdentifier::new(5377, 1600),
    ),
    (
        "lumaPoolsStateGroup.waterRaised",
        UberIdentifier::new(5377, 2286),
    ),
    (
        "lumaPoolsStateGroup.pullWallLeft",
        UberIdentifier::new(5377, 2518),
    ),
    (
        "lumaPoolsStateGroup.breakableSecretWallA",
        UberIdentifier::new(5377, 3831),
    ),
    (
        "lumaPoolsStateGroup.breakableWallB",
        UberIdentifier::new(5377, 4463),
    ),
    (
        "lumaPoolsStateGroup.leverAndDoor",
        UberIdentifier::new(5377, 6398),
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier::new(5377, 6857),
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholder",
        UberIdentifier::new(5377, 7381),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 7540),
    ),
    (
        "lumaPoolsStateGroup.breakRockDState",
        UberIdentifier::new(5377, 8019),
    ),
    (
        "lumaPoolsStateGroup.trunkState",
        UberIdentifier::new(5377, 8294),
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerBlocked",
        UberIdentifier::new(5377, 8440),
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier::new(5377, 8451),
    ),
    (
        "lumaPoolsStateGroup.expOrb",
        UberIdentifier::new(5377, 8939),
    ),
    (
        "lumaPoolsStateGroup.expOrbB",
        UberIdentifier::new(5377, 9812),
    ),
    (
        "lumaPoolsStateGroup.creepD",
        UberIdentifier::new(5377, 10291),
    ),
    (
        "lumaPoolsStateGroup.breakableWallB",
        UberIdentifier::new(5377, 10782),
    ),
    (
        "lumaPoolsStateGroup.bombableWallA",
        UberIdentifier::new(5377, 11049),
    ),
    (
        "lumaPoolsStateGroup.gorlekOreA",
        UberIdentifier::new(5377, 12235),
    ),
    (
        "lumaPoolsStateGroup.pressurePlateGate",
        UberIdentifier::new(5377, 12826),
    ),
    (
        "lumaPoolsStateGroup.xpOrbD",
        UberIdentifier::new(5377, 13832),
    ),
    (
        "lumaPoolsStateGroup.leverAndDoor",
        UberIdentifier::new(5377, 14488),
    ),
    (
        "lumaPoolsStateGroup.spiritShard",
        UberIdentifier::new(5377, 14664),
    ),
    (
        "lumaPoolsStateGroup.drillableWall",
        UberIdentifier::new(5377, 15383),
    ),
    (
        "lumaPoolsStateGroup.dashDoor",
        UberIdentifier::new(5377, 15402),
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedA",
        UberIdentifier::new(5377, 15754),
    ),
    (
        "lumaPoolsStateGroup.keystoneB",
        UberIdentifier::new(5377, 16426),
    ),
    (
        "lumaPoolsStateGroup.breakRockFState",
        UberIdentifier::new(5377, 16607),
    ),
    (
        "lumaPoolsStateGroup.xpOrbC",
        UberIdentifier::new(5377, 17396),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 18345),
    ),
    (
        "lumaPoolsStateGroup.areaText",
        UberIdentifier::new(5377, 19132),
    ),
    (
        "lumaPoolsStateGroup.pickupA",
        UberIdentifier::new(5377, 19694),
    ),
    (
        "lumaPoolsStateGroup.talkedToKwolok",
        UberIdentifier::new(5377, 21700),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 21860),
    ),
    (
        "lumaPoolsStateGroup.breakRockEState",
        UberIdentifier::new(5377, 22047),
    ),
    (
        "lumaPoolsStateGroup.breakableWallB",
        UberIdentifier::new(5377, 22978),
    ),
    (
        "lumaPoolsStateGroup.hintZones",
        UberIdentifier::new(5377, 24015),
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedA",
        UberIdentifier::new(5377, 24765),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 25391),
    ),
    (
        "lumaPoolsStateGroup.bridgeState",
        UberIdentifier::new(5377, 25612),
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderB",
        UberIdentifier::new(5377, 25633),
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier::new(5377, 26170),
    ),
    (
        "lumaPoolsStateGroup.lagoonDoor",
        UberIdentifier::new(5377, 26987),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 27204),
    ),
    (
        "lumaPoolsStateGroup.bombableWall",
        UberIdentifier::new(5377, 27558),
    ),
    (
        "lumaPoolsStateGroup.breakRockCState",
        UberIdentifier::new(5377, 29662),
    ),
    (
        "lumaPoolsStateGroup.loweringWaterState",
        UberIdentifier::new(5377, 29911),
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderB",
        UberIdentifier::new(5377, 30860),
    ),
    (
        "lumaPoolsStateGroup.treeFallen",
        UberIdentifier::new(5377, 31093),
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier::new(5377, 31145),
    ),
    (
        "lumaPoolsStateGroup.gorlekOreA",
        UberIdentifier::new(5377, 31434),
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerBlocked",
        UberIdentifier::new(5377, 32210),
    ),
    (
        "lumaPoolsStateGroup.creepB",
        UberIdentifier::new(5377, 32685),
    ),
    (
        "lumaPoolsStateGroup.energyContainerA",
        UberIdentifier::new(5377, 32750),
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderC",
        UberIdentifier::new(5377, 33110),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 33180),
    ),
    (
        "lumaPoolsStateGroup.breakRockAState",
        UberIdentifier::new(5377, 33730),
    ),
    (
        "lumaPoolsStateGroup.orePickupA",
        UberIdentifier::new(5377, 34852),
    ),
    (
        "lumaPoolsStateGroup.playedMokiVignette",
        UberIdentifier::new(5377, 35023),
    ),
    (
        "lumaPoolsStateGroup.keystoneA",
        UberIdentifier::new(5377, 35091),
    ),
    (
        "lumaPoolsStateGroup.xpOrbB",
        UberIdentifier::new(5377, 35440),
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedB",
        UberIdentifier::new(5377, 35751),
    ),
    (
        "lumaPoolsStateGroup.expOrb",
        UberIdentifier::new(5377, 35971),
    ),
    (
        "lumaPoolsStateGroup.breakableFloorA",
        UberIdentifier::new(5377, 36511),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 38515),
    ),
    (
        "lumaPoolsStateGroup.spiritShard",
        UberIdentifier::new(5377, 40328),
    ),
    (
        "lumaPoolsStateGroup.keystoneD",
        UberIdentifier::new(5377, 41881),
    ),
    (
        "lumaPoolsStateGroup.mainPickup",
        UberIdentifier::new(5377, 42145),
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderA",
        UberIdentifier::new(5377, 42553),
    ),
    (
        "lumaPoolsStateGroup.creepA",
        UberIdentifier::new(5377, 43134),
    ),
    (
        "lumaPoolsStateGroup.optionalPickup",
        UberIdentifier::new(5377, 43859),
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier::new(5377, 44122),
    ),
    (
        "lumaPoolsStateGroup.expOrbA",
        UberIdentifier::new(5377, 44777),
    ),
    (
        "lumaPoolsStateGroup.switchesActivated",
        UberIdentifier::new(5377, 45765),
    ),
    (
        "lumaPoolsStateGroup.healthContainerA",
        UberIdentifier::new(5377, 45774),
    ),
    (
        "lumaPoolsStateGroup.fallingRockState",
        UberIdentifier::new(5377, 46040),
    ),
    (
        "lumaPoolsStateGroup.keystoneC",
        UberIdentifier::new(5377, 46926),
    ),
    (
        "lumaPoolsStateGroup.keystoneGate",
        UberIdentifier::new(5377, 47621),
    ),
    (
        "lumaPoolsStateGroup.splitPlatformState",
        UberIdentifier::new(5377, 49394),
    ),
    (
        "lumaPoolsStateGroup.pullWallRight",
        UberIdentifier::new(5377, 49826),
    ),
    (
        "lumaPoolsStateGroup.doorState",
        UberIdentifier::new(5377, 52062),
    ),
    (
        "lumaPoolsStateGroup.breakableLogA",
        UberIdentifier::new(5377, 52133),
    ),
    (
        "lumaPoolsStateGroup.xpOrbB",
        UberIdentifier::new(5377, 52791),
    ),
    (
        "lumaPoolsStateGroup.bombableWallB",
        UberIdentifier::new(5377, 53532),
    ),
    (
        "lumaPoolsStateGroup.spiritShard",
        UberIdentifier::new(5377, 56199),
    ),
    (
        "lumaPoolsStateGroup.secretWallA",
        UberIdentifier::new(5377, 56302),
    ),
    (
        "lumaPoolsStateGroup.creepE",
        UberIdentifier::new(5377, 57334),
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblocked",
        UberIdentifier::new(5377, 57453),
    ),
    (
        "lumaPoolsStateGroup.kwolokChaseDoorState",
        UberIdentifier::new(5377, 57929),
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedB",
        UberIdentifier::new(5377, 58278),
    ),
    (
        "lumaPoolsStateGroup.doorState",
        UberIdentifier::new(5377, 59514),
    ),
    (
        "lumaPoolsStateGroup.pickupA",
        UberIdentifier::new(5377, 61475),
    ),
    (
        "lumaPoolsStateGroup.xpOrbB",
        UberIdentifier::new(5377, 62180),
    ),
    (
        "lumaPoolsStateGroup.waterLowered",
        UberIdentifier::new(5377, 63173),
    ),
    (
        "lumaPoolsStateGroup.healthContainerA",
        UberIdentifier::new(5377, 63201),
    ),
    (
        "lumaPoolsStateGroup.doorState",
        UberIdentifier::new(5377, 63513),
    ),
    (
        "lumaPoolsStateGroup.secretWallA",
        UberIdentifier::new(5377, 63922),
    ),
    (
        "lumaPoolsStateGroup.breakableWall",
        UberIdentifier::new(5377, 64337),
    ),
    (
        "lumaPoolsStateGroup.creepC",
        UberIdentifier::new(5377, 64761),
    ),
    (
        "lumaPoolsStateGroup.breakRockBState",
        UberIdentifier::new(5377, 64827),
    ),
    (
        "lumaPoolsStateGroup.gorlekOreA",
        UberIdentifier::new(5377, 65019),
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier::new(5377, 65413),
    ),
    (
        "lumaPoolsStateGroup.healthPlantA",
        UberIdentifier::new(5377, 47557),
    ),
    (
        "lumaPoolsStateGroup.healthPlantA",
        UberIdentifier::new(5377, 63230),
    ),
    (
        "lumaPoolsStateGroup.arenaByteStateSerialized",
        UberIdentifier::new(5377, 1373),
    ),
    (
        "lumaPoolsStateGroup.arenaBByteStateSerialized",
        UberIdentifier::new(5377, 53480),
    ),
    (
        "testUberStateGroup.firePedestalBooleanUberState",
        UberIdentifier::new(6837, 5475),
    ),
    (
        "testUberStateGroup.kwolokCavernDoor2",
        UberIdentifier::new(6837, 7403),
    ),
    (
        "testUberStateGroup.desertShortcutWall",
        UberIdentifier::new(6837, 10235),
    ),
    (
        "testUberStateGroup.testDoorTwoSlotsBooleanUberState",
        UberIdentifier::new(6837, 19173),
    ),
    (
        "testUberStateGroup.testShrineUberStateDescriptor",
        UberIdentifier::new(6837, 19701),
    ),
    (
        "testUberStateGroup.arenaCompletedState",
        UberIdentifier::new(6837, 31278),
    ),
    (
        "testUberStateGroup.lianaHealLantern",
        UberIdentifier::new(6837, 31353),
    ),
    (
        "testUberStateGroup.willowsEndShortcutWall",
        UberIdentifier::new(6837, 38771),
    ),
    (
        "testUberStateGroup.swampShortcutWall",
        UberIdentifier::new(6837, 40492),
    ),
    (
        "testUberStateGroup.winterForestEnemyDoor",
        UberIdentifier::new(6837, 44762),
    ),
    (
        "testUberStateGroup.lagoonContactSwitch",
        UberIdentifier::new(6837, 47735),
    ),
    (
        "testUberStateGroup.watermillShortcutWall",
        UberIdentifier::new(6837, 51086),
    ),
    (
        "testUberStateGroup.kwolokCavernDoor",
        UberIdentifier::new(6837, 54316),
    ),
    (
        "testUberStateGroup.testLeverDescriptorDesertC",
        UberIdentifier::new(6837, 54999),
    ),
    (
        "testUberStateGroup.oneSideBreakableWall",
        UberIdentifier::new(6837, 55663),
    ),
    (
        "testUberStateGroup.testSecret",
        UberIdentifier::new(6837, 60688),
    ),
    (
        "testUberStateGroup.testBooleanUberStateDescriptor",
        UberIdentifier::new(6837, 60823),
    ),
    (
        "testUberStateGroup.cordycepsShortcutWall",
        UberIdentifier::new(6837, 61703),
    ),
    (
        "testUberStateGroup.kwolokCavernsPressurePlate",
        UberIdentifier::new(6837, 62194),
    ),
    (
        "testUberStateGroup.kwolokCavernsAttackableSwitch",
        UberIdentifier::new(6837, 62909),
    ),
    (
        "testUberStateGroup.lagoonShortcutWall",
        UberIdentifier::new(6837, 64646),
    ),
    (
        "testUberStateGroup.testBreakableWallInt",
        UberIdentifier::new(6837, 37967),
    ),
    (
        "testUberStateGroup.testBreakableWallIntB",
        UberIdentifier::new(6837, 61358),
    ),
    (
        "testUberStateGroup.serializedInt",
        UberIdentifier::new(6837, 63967),
    ),
    (
        "testUberStateGroup.landOnAndSpawnOrbs",
        UberIdentifier::new(6837, 39815),
    ),
    (
        "testUberStateGroup.testSerializedFloatUberState",
        UberIdentifier::new(6837, 61561),
    ),
    (
        "desertAGroup.collectableHDesertA",
        UberIdentifier::new(7228, 1781),
    ),
    (
        "desertAGroup.collectableEDesertA",
        UberIdentifier::new(7228, 2996),
    ),
    (
        "desertAGroup.secretWall",
        UberIdentifier::new(7228, 4034),
    ),
    (
        "desertAGroup.gorlekOre",
        UberIdentifier::new(7228, 8370),
    ),
    (
        "desertAGroup.keystoneAUberState",
        UberIdentifier::new(7228, 20282),
    ),
    (
        "desertAGroup.collectableFDesertA",
        UberIdentifier::new(7228, 32434),
    ),
    (
        "desertAGroup.expOrb",
        UberIdentifier::new(7228, 35329),
    ),
    (
        "desertAGroup.collectableDesertA",
        UberIdentifier::new(7228, 36579),
    ),
    (
        "desertAGroup.lifeCellBooleanUberState",
        UberIdentifier::new(7228, 37885),
    ),
    (
        "desertAGroup.xpOrbUberState",
        UberIdentifier::new(7228, 45954),
    ),
    (
        "desertAGroup.xpOrbBUberState",
        UberIdentifier::new(7228, 48993),
    ),
    (
        "desertAGroup.collectableCDesertA",
        UberIdentifier::new(7228, 52086),
    ),
    (
        "desertAGroup.xpOrbB",
        UberIdentifier::new(7228, 54275),
    ),
    (
        "desertAGroup.gorlekOre",
        UberIdentifier::new(7228, 54494),
    ),
    (
        "desertAGroup.collectableADesertA",
        UberIdentifier::new(7228, 56821),
    ),
    (
        "desertAGroup.collectableGDesertA",
        UberIdentifier::new(7228, 60605),
    ),
    (
        "desertAGroup.xpOrbAUberState",
        UberIdentifier::new(7228, 61548),
    ),
    (
        "desertAGroup.keystoneBUberState",
        UberIdentifier::new(7228, 62117),
    ),
    (
        "statsUberStateGroup.totalSpiritLightCollectedSerializedIntUberState",
        UberIdentifier::new(8246, 5144),
    ),
    (
        "statsUberStateGroup.fastTravelCountIntUberState",
        UberIdentifier::new(8246, 7909),
    ),
    (
        "statsUberStateGroup.enemiesPiercedAtOnceStatSettingSerializedUberState",
        UberIdentifier::new(8246, 7927),
    ),
    (
        "statsUberStateGroup.deathFromEnemiesStatSettingSerializedUberState",
        UberIdentifier::new(8246, 12323),
    ),
    (
        "statsUberStateGroup.npcsInHubStatSettingSerializedUberState",
        UberIdentifier::new(8246, 15506),
    ),
    (
        "statsUberStateGroup.bashesStatSettingSerializedUberState",
        UberIdentifier::new(8246, 17772),
    ),
    (
        "statsUberStateGroup.shardSlotUpgradesCollectedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 18554),
    ),
    (
        "statsUberStateGroup.mostDefeatedEnemyEnumStatSettingSerializedUberState",
        UberIdentifier::new(8246, 26498),
    ),
    (
        "statsUberStateGroup.totalDamageTakenStatSettingSerializedUberState",
        UberIdentifier::new(8246, 28073),
    ),
    (
        "statsUberStateGroup.wallJumpsStatSettingSerializedUberState",
        UberIdentifier::new(8246, 30164),
    ),
    (
        "statsUberStateGroup.spiritLightCollectedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 30251),
    ),
    (
        "statsUberStateGroup.sideQuestsCompletedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 31056),
    ),
    (
        "statsUberStateGroup.enemyVsEnemyKillsStatSettingSerializedUberState",
        UberIdentifier::new(8246, 31216),
    ),
    (
        "statsUberStateGroup.enemiesDefeatedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 32860),
    ),
    (
        "statsUberStateGroup.deathsStatSettingSerializedUberState",
        UberIdentifier::new(8246, 36466),
    ),
    (
        "statsUberStateGroup.totalSpiritLightSpentSerializedIntUberState",
        UberIdentifier::new(8246, 37583),
    ),
    (
        "statsUberStateGroup.highestAmountOfDamageSerializedIntUberState",
        UberIdentifier::new(8246, 40254),
    ),
    (
        "statsUberStateGroup.totalHealthRegeneratedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 42772),
    ),
    (
        "statsUberStateGroup.gardenerSeedsCollectedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 44318),
    ),
    (
        "statsUberStateGroup.racesCompletedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 49162),
    ),
    (
        "statsUberStateGroup.favoriteSkillEnumStatSettingSerializedUberState",
        UberIdentifier::new(8246, 49721),
    ),
    (
        "statsUberStateGroup.shrinesDiscoveredStatSettingSerializedUberState",
        UberIdentifier::new(8246, 50096),
    ),
    (
        "statsUberStateGroup.spiritLightSpentStatSettingSerializedUberState",
        UberIdentifier::new(8246, 50669),
    ),
    (
        "statsUberStateGroup.dashesStatSettingSerializedUberState",
        UberIdentifier::new(8246, 50952),
    ),
    (
        "statsUberStateGroup.racePedestalsActivatedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 54110),
    ),
    (
        "statsUberStateGroup.deathsEnvironmentalStatSettingSerializedUberState",
        UberIdentifier::new(8246, 57639),
    ),
    (
        "statsUberStateGroup.drowningDeathsStatSettingSerializedUberState",
        UberIdentifier::new(8246, 58048),
    ),
    (
        "statsUberStateGroup.jumpsStatSettingSerializedUberState",
        UberIdentifier::new(8246, 58908),
    ),
    (
        "statsUberStateGroup.shardsCollectedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 59865),
    ),
    (
        "statsUberStateGroup.spiritWellsDiscoveredStatSettingSerializedUberState",
        UberIdentifier::new(8246, 60852),
    ),
    (
        "statsUberStateGroup.mostDefeatedByEnemyEnumStatSettingSerializedUberState",
        UberIdentifier::new(8246, 62287),
    ),
    (
        "statsUberStateGroup.shrinesCompletedStatSettingSerializedUberState",
        UberIdentifier::new(8246, 63037),
    ),
    (
        "statsUberStateGroup.leashesStatSettingSerializedUberState",
        UberIdentifier::new(8246, 64519),
    ),
    (
        "statsUberStateGroup.teleportCountStatSettingSerializedUberState",
        UberIdentifier::new(8246, 64778),
    ),
    (
        "statsUberStateGroup.timeAirborneStatSettingSerializedUberState",
        UberIdentifier::new(8246, 3307),
    ),
    (
        "statsUberStateGroup.timeGlowingStatSettingSerializedUberState",
        UberIdentifier::new(8246, 7293),
    ),
    (
        "statsUberStateGroup.distanceSwamStatSettingSerializedFloatUberState",
        UberIdentifier::new(8246, 8682),
    ),
    (
        "statsUberStateGroup.distanceGlidedStatSettingSerializedFloatUberState",
        UberIdentifier::new(8246, 16123),
    ),
    (
        "statsUberStateGroup.distanceBurrowedStatSettingSerializedFloatUberState",
        UberIdentifier::new(8246, 40261),
    ),
    (
        "statsUberStateGroup.timeTotalPlaytimeStatSettingSerializedUberState",
        UberIdentifier::new(8246, 43418),
    ),
    (
        "statsUberStateGroup.distanceFallingStatSettingSerializedFloatUberState",
        UberIdentifier::new(8246, 44439),
    ),
    (
        "statsUberStateGroup.timeAliveUberState",
        UberIdentifier::new(8246, 47477),
    ),
    (
        "statsUberStateGroup.timeLongestSingleAirborneStatSettingSerializedUberState",
        UberIdentifier::new(8246, 49364),
    ),
    (
        "statsUberStateGroup.energySpentSerializedFloatUberState",
        UberIdentifier::new(8246, 60940),
    ),
    (
        "statsUberStateGroup.distanceTravelledStatSettingSerializedFloatUberState",
        UberIdentifier::new(8246, 62310),
    ),
    (
        "inkwaterMarshStateGroup.mokiTorchPlayed",
        UberIdentifier::new(9593, 3621),
    ),
    (
        "inkwaterMarshStateGroup.expOrbA",
        UberIdentifier::new(9593, 5253),
    ),
    (
        "inkwaterMarshStateGroup.xpOrbA",
        UberIdentifier::new(9593, 5929),
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier::new(9593, 7849),
    ),
    (
        "inkwaterMarshStateGroup.lanternAndCreepA",
        UberIdentifier::new(9593, 9229),
    ),
    (
        "inkwaterMarshStateGroup.breakableLogB",
        UberIdentifier::new(9593, 14616),
    ),
    (
        "inkwaterMarshStateGroup.climbHintShown",
        UberIdentifier::new(9593, 15672),
    ),
    (
        "inkwaterMarshStateGroup.stompableFloor",
        UberIdentifier::new(9593, 17659),
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier::new(9593, 17818),
    ),
    (
        "inkwaterMarshStateGroup.lasersDiscovered",
        UberIdentifier::new(9593, 17991),
    ),
    (
        "inkwaterMarshStateGroup.gorlekOreA",
        UberIdentifier::new(9593, 20382),
    ),
    (
        "inkwaterMarshStateGroup.energyVessel",
        UberIdentifier::new(9593, 22802),
    ),
    (
        "inkwaterMarshStateGroup.breakableWall",
        UberIdentifier::new(9593, 23319),
    ),
    (
        "inkwaterMarshStateGroup.gorlekOreA",
        UberIdentifier::new(9593, 23858),
    ),
    (
        "inkwaterMarshStateGroup.gorlekOreA",
        UberIdentifier::new(9593, 25989),
    ),
    (
        "inkwaterMarshStateGroup.lanternAndCreepB",
        UberIdentifier::new(9593, 26238),
    ),
    (
        "inkwaterMarshStateGroup.energyContainer",
        UberIdentifier::new(9593, 26457),
    ),
    (
        "inkwaterMarshStateGroup.halfEnergyCellA",
        UberIdentifier::new(9593, 27562),
    ),
    (
        "inkwaterMarshStateGroup.secretWallA",
        UberIdentifier::new(9593, 34704),
    ),
    (
        "inkwaterMarshStateGroup.xpOrb",
        UberIdentifier::new(9593, 42047),
    ),
    (
        "inkwaterMarshStateGroup.xpOrbB",
        UberIdentifier::new(9593, 45321),
    ),
    (
        "inkwaterMarshStateGroup.secretWallA",
        UberIdentifier::new(9593, 47420),
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier::new(9593, 53947),
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier::new(9593, 59344),
    ),
    (
        "inkwaterMarshStateGroup.enemyRoom",
        UberIdentifier::new(9593, 59418),
    ),
    (
        "inkwaterMarshStateGroup.healthContainer",
        UberIdentifier::new(9593, 61304),
    ),
    (
        "inkwaterMarshStateGroup.lizardMultiWaveArenaInt",
        UberIdentifier::new(9593, 25130),
    ),
    (
        "inkwaterMarshStateGroup.swampArenaAInt",
        UberIdentifier::new(9593, 31687),
    ),
    (
        "inkwaterMarshStateGroup.swampArenaA",
        UberIdentifier::new(9593, 45142),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 94),
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierD",
        UberIdentifier::new(10289, 1620),
    ),
    (
        "windtornRuinsGroup.ruinsVisited",
        UberIdentifier::new(10289, 3621),
    ),
    (
        "windtornRuinsGroup.openedDesertRuins",
        UberIdentifier::new(10289, 3804),
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierB",
        UberIdentifier::new(10289, 4154),
    ),
    (
        "windtornRuinsGroup.baseKillzoneState",
        UberIdentifier::new(10289, 7638),
    ),
    (
        "windtornRuinsGroup.bombableWallDesertC",
        UberIdentifier::new(10289, 8436),
    ),
    (
        "windtornRuinsGroup.drillZoneD",
        UberIdentifier::new(10289, 8533),
    ),
    (
        "windtornRuinsGroup.drillZoneF",
        UberIdentifier::new(10289, 10093),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 12859),
    ),
    (
        "windtornRuinsGroup.sandwormActiveA",
        UberIdentifier::new(10289, 13021),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 15867),
    ),
    (
        "windtornRuinsGroup.escapeBridgeB",
        UberIdentifier::new(10289, 16802),
    ),
    (
        "windtornRuinsGroup.wispRewardPickup",
        UberIdentifier::new(10289, 22102),
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierC",
        UberIdentifier::new(10289, 23922),
    ),
    (
        "windtornRuinsGroup.collapseSequenceB",
        UberIdentifier::new(10289, 27089),
    ),
    (
        "windtornRuinsGroup.escapeRockI",
        UberIdentifier::new(10289, 27929),
    ),
    (
        "windtornRuinsGroup.escapeRockC",
        UberIdentifier::new(10289, 28779),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 29069),
    ),
    (
        "windtornRuinsGroup.fallingPillars",
        UberIdentifier::new(10289, 29425),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 30540),
    ),
    (
        "windtornRuinsGroup.windsweptWastesRuinsDoorCannotOpen",
        UberIdentifier::new(10289, 31524),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 31750),
    ),
    (
        "windtornRuinsGroup.escapeRockG",
        UberIdentifier::new(10289, 32483),
    ),
    (
        "windtornRuinsGroup.escapeRockE",
        UberIdentifier::new(10289, 32833),
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier::new(10289, 36274),
    ),
    (
        "windtornRuinsGroup.windtornRuinsAKeystoneDoor",
        UberIdentifier::new(10289, 37849),
    ),
    (
        "windtornRuinsGroup.drillZoneC",
        UberIdentifier::new(10289, 38171),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 38721),
    ),
    (
        "windtornRuinsGroup.escapeRockF",
        UberIdentifier::new(10289, 40310),
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierA",
        UberIdentifier::new(10289, 40790),
    ),
    (
        "windtornRuinsGroup.lever",
        UberIdentifier::new(10289, 41277),
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier::new(10289, 41902),
    ),
    (
        "windtornRuinsGroup.rootBreakPillarFall",
        UberIdentifier::new(10289, 43103),
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier::new(10289, 44426),
    ),
    (
        "windtornRuinsGroup.energyHalfCell",
        UberIdentifier::new(10289, 44555),
    ),
    (
        "windtornRuinsGroup.escapeEndRocks",
        UberIdentifier::new(10289, 45179),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 45766),
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier::new(10289, 46316),
    ),
    (
        "windtornRuinsGroup.healthHalfCell",
        UberIdentifier::new(10289, 48372),
    ),
    (
        "windtornRuinsGroup.keystoneDoor",
        UberIdentifier::new(10289, 48604),
    ),
    (
        "windtornRuinsGroup.escapeRockJ",
        UberIdentifier::new(10289, 50961),
    ),
    (
        "windtornRuinsGroup.wormBreakFloor",
        UberIdentifier::new(10289, 52478),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 55317),
    ),
    (
        "windtornRuinsGroup.drillZoneE",
        UberIdentifier::new(10289, 55672),
    ),
    (
        "windtornRuinsGroup.escapeRockH",
        UberIdentifier::new(10289, 55692),
    ),
    (
        "windtornRuinsGroup.bombableWall",
        UberIdentifier::new(10289, 55787),
    ),
    (
        "windtornRuinsGroup.escapeRockA",
        UberIdentifier::new(10289, 57325),
    ),
    (
        "windtornRuinsGroup.areaText",
        UberIdentifier::new(10289, 61217),
    ),
    (
        "windtornRuinsGroup.xpOrbA",
        UberIdentifier::new(10289, 61615),
    ),
    (
        "windtornRuinsGroup.drillZoneC",
        UberIdentifier::new(10289, 62291),
    ),
    (
        "windtornRuinsGroup.collapseSequenceA",
        UberIdentifier::new(10289, 62926),
    ),
    (
        "windtornRuinsGroup.escapeRockD",
        UberIdentifier::new(10289, 63154),
    ),
    (
        "windtornRuinsGroup.desertSruinsChaseSandWall",
        UberIdentifier::new(10289, 63700),
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier::new(10289, 64240),
    ),
    (
        "windtornRuinsGroup.escapeRockB",
        UberIdentifier::new(10289, 65145),
    ),
    (
        "windtornRuinsGroup.savePedestalUberState",
        UberIdentifier::new(10289, 4928),
    ),
    (
        "windtornRuinsGroup.DesertSavePedestal",
        UberIdentifier::new(10289, 13937),
    ),
    (
        "windtornRuinsGroup.savePedestalUberState",
        UberIdentifier::new(10289, 40484),
    ),
    (
        "windtornRuinsGroup.rotatingBlockSetupRotation",
        UberIdentifier::new(10289, 93),
    ),
    (
        "windtornRuinsGroup.powerLineIntUberStateA",
        UberIdentifier::new(10289, 312),
    ),
    (
        "windtornRuinsGroup.powerLineIntUberState",
        UberIdentifier::new(10289, 3217),
    ),
    (
        "windtornRuinsGroup.powerLineIntUberStateC",
        UberIdentifier::new(10289, 3682),
    ),
    (
        "windtornRuinsGroup.wormNodeStateB",
        UberIdentifier::new(10289, 6414),
    ),
    (
        "windtornRuinsGroup.wormNodeStateC",
        UberIdentifier::new(10289, 12614),
    ),
    (
        "windtornRuinsGroup.wormNodeStateG",
        UberIdentifier::new(10289, 16886),
    ),
    (
        "windtornRuinsGroup.desertRuinsEscape",
        UberIdentifier::new(10289, 19890),
    ),
    (
        "windtornRuinsGroup.wormNodeStateF",
        UberIdentifier::new(10289, 23855),
    ),
    (
        "windtornRuinsGroup.wormNodeStateA",
        UberIdentifier::new(10289, 27997),
    ),
    (
        "windtornRuinsGroup.powerLineIntUberState",
        UberIdentifier::new(10289, 35130),
    ),
    (
        "windtornRuinsGroup.wormNodeStateE",
        UberIdentifier::new(10289, 45821),
    ),
    (
        "windtornRuinsGroup.wormNodeState",
        UberIdentifier::new(10289, 47857),
    ),
    (
        "windtornRuinsGroup.wormNodeStateD",
        UberIdentifier::new(10289, 50264),
    ),
    (
        "windtornRuinsGroup.wormNodeStateH",
        UberIdentifier::new(10289, 56515),
    ),
    (
        "windtornRuinsGroup.powerLineIntUberStateB",
        UberIdentifier::new(10289, 58350),
    ),
    (
        "windtornRuinsGroup.desertRuinsWispSequencePlayed",
        UberIdentifier::new(10289, 60565),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateA",
        UberIdentifier::new(10289, 5546),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateD",
        UberIdentifier::new(10289, 5814),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateC",
        UberIdentifier::new(10289, 10828),
    ),
    (
        "windtornRuinsGroup.wormDistanceToNextNodeState",
        UberIdentifier::new(10289, 35190),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateH",
        UberIdentifier::new(10289, 36008),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateB",
        UberIdentifier::new(10289, 51149),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateE",
        UberIdentifier::new(10289, 52211),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateF",
        UberIdentifier::new(10289, 58175),
    ),
    (
        "windtornRuinsGroup.wormDistanceStateG",
        UberIdentifier::new(10289, 63894),
    ),
    (
        "howlsDenGRoup.hasOriUsedSavePedestal",
        UberIdentifier::new(11666, 4220),
    ),
    (
        "howlsDenGRoup.saveRoomDoor",
        UberIdentifier::new(11666, 4932),
    ),
    (
        "howlsDenGRoup.howlsDenLargeXPOrbA",
        UberIdentifier::new(11666, 24943),
    ),
    (
        "howlsDenGRoup.areaText",
        UberIdentifier::new(11666, 42038),
    ),
    (
        "howlsDenGRoup.savePedestalUberState",
        UberIdentifier::new(11666, 16542),
    ),
    (
        "howlsDenGRoup.savePedestalUberState",
        UberIdentifier::new(11666, 20829),
    ),
    (
        "howlsDenGRoup.savePedestal",
        UberIdentifier::new(11666, 61594),
    ),
    (
        "leaderboardsUberStateGroup.baursReachLeaderboardNotificationState",
        UberIdentifier::new(13298, 54921),
    ),
    (
        "leaderboardsUberStateGroup.baursReachLeaderboardPlaceState",
        UberIdentifier::new(13298, 3608),
    ),
    (
        "leaderboardsUberStateGroup.desertLeaderboardPlaceState",
        UberIdentifier::new(13298, 4929),
    ),
    (
        "leaderboardsUberStateGroup.hornbugBossLeaderboardPlaceState",
        UberIdentifier::new(13298, 6736),
    ),
    (
        "leaderboardsUberStateGroup.watermillEscapeLeaderboardPlaceState",
        UberIdentifier::new(13298, 14784),
    ),
    (
        "leaderboardsUberStateGroup.laserShooterMiniBossLeaderboardPlaceState",
        UberIdentifier::new(13298, 20341),
    ),
    (
        "leaderboardsUberStateGroup.kwolokBossLeaderboardPlaceState",
        UberIdentifier::new(13298, 37881),
    ),
    (
        "leaderboardsUberStateGroup.inkwaterLeaderboardPlaceState",
        UberIdentifier::new(13298, 40104),
    ),
    (
        "leaderboardsUberStateGroup.spiderBossLeaderboardPlaceState",
        UberIdentifier::new(13298, 41733),
    ),
    (
        "leaderboardsUberStateGroup.desertEscapeLeaderboardPlaceState",
        UberIdentifier::new(13298, 44392),
    ),
    (
        "leaderboardsUberStateGroup.kwoloksLeaderboardPlaceState",
        UberIdentifier::new(13298, 53149),
    ),
    (
        "leaderboardsUberStateGroup.avalancheEscapeLeaderboardPlaceState",
        UberIdentifier::new(13298, 53528),
    ),
    (
        "leaderboardsUberStateGroup.wellspringLeaderboardPlaceState",
        UberIdentifier::new(13298, 53967),
    ),
    (
        "leaderboardsUberStateGroup.silentWoodlandLeaderboardPlaceState",
        UberIdentifier::new(13298, 55577),
    ),
    (
        "leaderboardsUberStateGroup.lumaPoolsLeaderboardPlaceState",
        UberIdentifier::new(13298, 58679),
    ),
    (
        "leaderboardsUberStateGroup.mouldwoodLeaderboardPlaceState",
        UberIdentifier::new(13298, 59179),
    ),
    (
        "leaderboardsUberStateGroup.owlBossLeaderboardPlaceState",
        UberIdentifier::new(13298, 64962),
    ),
    (
        "bashIntroductionA__clone1Group.healthContainerA",
        UberIdentifier::new(13428, 59730),
    ),
    (
        "questUberStateGroup.gardenerHutDiscovered",
        UberIdentifier::new(14019, 353),
    ),
    (
        "questUberStateGroup.darkCaveQuestItemCollected",
        UberIdentifier::new(14019, 2782),
    ),
    (
        "questUberStateGroup.firstRaceDiscovered",
        UberIdentifier::new(14019, 5662),
    ),
    (
        "questUberStateGroup.gardenerSeedTreeCollected",
        UberIdentifier::new(14019, 7470),
    ),
    (
        "questUberStateGroup.gardenerSeedBashCollected",
        UberIdentifier::new(14019, 8192),
    ),
    (
        "questUberStateGroup.mapstoneDiscovered",
        UberIdentifier::new(14019, 9874),
    ),
    (
        "questUberStateGroup.mouldwoodDiscovered",
        UberIdentifier::new(14019, 12642),
    ),
    (
        "questUberStateGroup.lanternItemCollected",
        UberIdentifier::new(14019, 14931),
    ),
    (
        "questUberStateGroup.howlsOriginWellOpened",
        UberIdentifier::new(14019, 20290),
    ),
    (
        "questUberStateGroup.gardenerSeedFlowersCollected",
        UberIdentifier::new(14019, 20601),
    ),
    (
        "questUberStateGroup.gardenerSeedGrappleCollected",
        UberIdentifier::new(14019, 24142),
    ),
    (
        "questUberStateGroup.wellspringShrineDiscovered",
        UberIdentifier::new(14019, 27270),
    ),
    (
        "questUberStateGroup.braveMokiItemCollected",
        UberIdentifier::new(14019, 27539),
    ),
    (
        "questUberStateGroup.gardenerSeedGrassCollected",
        UberIdentifier::new(14019, 28662),
    ),
    (
        "questUberStateGroup.desertDiscovered",
        UberIdentifier::new(14019, 29163),
    ),
    (
        "questUberStateGroup.lagoonDiscovered",
        UberIdentifier::new(14019, 29202),
    ),
    (
        "questUberStateGroup.howlsOriginDiscovered",
        UberIdentifier::new(14019, 30671),
    ),
    (
        "questUberStateGroup.desertRuinsDiscovered",
        UberIdentifier::new(14019, 31413),
    ),
    (
        "questUberStateGroup.gardenerSeedSpringCollected",
        UberIdentifier::new(14019, 32376),
    ),
    (
        "questUberStateGroup.mapSecretsRevealed",
        UberIdentifier::new(14019, 35534),
    ),
    (
        "questUberStateGroup.howlsDenShrineDiscovered",
        UberIdentifier::new(14019, 36248),
    ),
    (
        "questUberStateGroup.inkwaterShrineDiscovered",
        UberIdentifier::new(14019, 40630),
    ),
    (
        "questUberStateGroup.baurDiscovered",
        UberIdentifier::new(14019, 46529),
    ),
    (
        "questUberStateGroup.discoveredWillowsEnd",
        UberIdentifier::new(14019, 50847),
    ),
    (
        "questUberStateGroup.silentWoodsShrineDiscovered",
        UberIdentifier::new(14019, 52274),
    ),
    (
        "questUberStateGroup.howlsOriginTreasureCollected",
        UberIdentifier::new(14019, 52747),
    ),
    (
        "questUberStateGroup.kwoloksWisdomItemCollected",
        UberIdentifier::new(14019, 53103),
    ),
    (
        "questUberStateGroup.mouldwoodShrineDiscovered",
        UberIdentifier::new(14019, 54970),
    ),
    (
        "questUberStateGroup.familyReunionItemCollected",
        UberIdentifier::new(14019, 57399),
    ),
    (
        "questUberStateGroup.mineGemItemCollected",
        UberIdentifier::new(14019, 58342),
    ),
    (
        "questUberStateGroup.inDangerBool",
        UberIdentifier::new(14019, 60646),
    ),
    (
        "questUberStateGroup.desertCogItemCollected",
        UberIdentifier::new(14019, 63396),
    ),
    (
        "questUberStateGroup.discoveredWeepingRidge",
        UberIdentifier::new(14019, 63965),
    ),
    (
        "questUberStateGroup.helpingHandQuestUberState",
        UberIdentifier::new(14019, 1341),
    ),
    (
        "questUberStateGroup.reachWaterMillQuestUberState",
        UberIdentifier::new(14019, 5737),
    ),
    (
        "questUberStateGroup.dialogQuest",
        UberIdentifier::new(14019, 6284),
    ),
    (
        "questUberStateGroup.winterForestWispQuestUberState",
        UberIdentifier::new(14019, 8973),
    ),
    (
        "questUberStateGroup.baursReachJTokkInteractionQuest",
        UberIdentifier::new(14019, 11308),
    ),
    (
        "questUberStateGroup.howlsDenShrineRumorMokiState",
        UberIdentifier::new(14019, 12437),
    ),
    (
        "questUberStateGroup.mouldwoodRumorMokiState",
        UberIdentifier::new(14019, 13512),
    ),
    (
        "questUberStateGroup.braveMokiQuest",
        UberIdentifier::new(14019, 15983),
    ),
    (
        "questUberStateGroup.wellspringShrineRumorMokiState",
        UberIdentifier::new(14019, 15995),
    ),
    (
        "questUberStateGroup.wellspringShrineRumorState",
        UberIdentifier::new(14019, 16509),
    ),
    (
        "questUberStateGroup.mouldwoodShrineRumorState",
        UberIdentifier::new(14019, 18061),
    ),
    (
        "questUberStateGroup.lagoonRumorState",
        UberIdentifier::new(14019, 19024),
    ),
    (
        "questUberStateGroup.desertRuinsRumorState",
        UberIdentifier::new(14019, 19060),
    ),
    (
        "questUberStateGroup.brothersQuest",
        UberIdentifier::new(14019, 19157),
    ),
    (
        "questUberStateGroup.lostCompassQuest",
        UberIdentifier::new(14019, 20667),
    ),
    (
        "questUberStateGroup.gardenerIntroQuest",
        UberIdentifier::new(14019, 23459),
    ),
    (
        "questUberStateGroup.lastTreeQuest",
        UberIdentifier::new(14019, 23787),
    ),
    (
        "questUberStateGroup.inkwaterShrineRumorState",
        UberIdentifier::new(14019, 23863),
    ),
    (
        "questUberStateGroup.optionalVSQuestAUberState",
        UberIdentifier::new(14019, 24152),
    ),
    (
        "questUberStateGroup.luposMapQuest",
        UberIdentifier::new(14019, 24683),
    ),
    (
        "questUberStateGroup.tradeSequenceQuest",
        UberIdentifier::new(14019, 26318),
    ),
    (
        "questUberStateGroup.regrowGladesQuest",
        UberIdentifier::new(14019, 26394),
    ),
    (
        "questUberStateGroup.silentWoodsShrineRumorState",
        UberIdentifier::new(14019, 27011),
    ),
    (
        "questUberStateGroup.familyReunionQuest",
        UberIdentifier::new(14019, 27804),
    ),
    (
        "questUberStateGroup.howlsDenShrineRumorState",
        UberIdentifier::new(14019, 27822),
    ),
    (
        "questUberStateGroup.gardenerHutRumorState",
        UberIdentifier::new(14019, 30596),
    ),
    (
        "questUberStateGroup.freeGromQuestUberState",
        UberIdentifier::new(14019, 33762),
    ),
    (
        "questUberStateGroup.darkCaveQuest",
        UberIdentifier::new(14019, 33776),
    ),
    (
        "questUberStateGroup.findKuQuest",
        UberIdentifier::new(14019, 34504),
    ),
    (
        "questUberStateGroup.lagoonWispQuestUberState",
        UberIdentifier::new(14019, 35087),
    ),
    (
        "questUberStateGroup.desertWispQuestUberState",
        UberIdentifier::new(14019, 35399),
    ),
    (
        "questUberStateGroup.mapstoneRumorState",
        UberIdentifier::new(14019, 39957),
    ),
    (
        "questUberStateGroup.howlsOriginRumorState",
        UberIdentifier::new(14019, 40952),
    ),
    (
        "questUberStateGroup.firstRaceRumorState",
        UberIdentifier::new(14019, 42501),
    ),
    (
        "questUberStateGroup.findHelpQuestUberState",
        UberIdentifier::new(14019, 44059),
    ),
    (
        "questUberStateGroup.baurRumorState",
        UberIdentifier::new(14019, 44184),
    ),
    (
        "questUberStateGroup.lookForKuQuestUberState",
        UberIdentifier::new(14019, 44500),
    ),
    (
        "questUberStateGroup.rebuildGladesQuest",
        UberIdentifier::new(14019, 44578),
    ),
    (
        "questUberStateGroup.mouldwoodDepthsWispQuestUberState",
        UberIdentifier::new(14019, 45931),
    ),
    (
        "questUberStateGroup.lagoonRumorMokiState",
        UberIdentifier::new(14019, 47774),
    ),
    (
        "questUberStateGroup.findToadQuestUberState",
        UberIdentifier::new(14019, 48794),
    ),
    (
        "questUberStateGroup.baurRumorMokiState",
        UberIdentifier::new(14019, 50230),
    ),
    (
        "questUberStateGroup.swampSpringIntroductionBOpherInteractionQuest",
        UberIdentifier::new(14019, 50571),
    ),
    (
        "questUberStateGroup.kwoloksWisdomQuest",
        UberIdentifier::new(14019, 50597),
    ),
    (
        "questUberStateGroup.mouldwoodRumorState",
        UberIdentifier::new(14019, 53066),
    ),
    (
        "questUberStateGroup.lastGlobalEvent",
        UberIdentifier::new(14019, 54675),
    ),
    (
        "questUberStateGroup.killTentacleQuestUberState",
        UberIdentifier::new(14019, 57066),
    ),
    (
        "questUberStateGroup.desertRumorState",
        UberIdentifier::new(14019, 57552),
    ),
    (
        "questUberStateGroup.searchForGrolQuest",
        UberIdentifier::new(14019, 59705),
    ),
    (
        "questUberStateGroup.treeKeeperQuest",
        UberIdentifier::new(14019, 59708),
    ),
    (
        "questUberStateGroup.desertCogQuest",
        UberIdentifier::new(14019, 61011),
    ),
    (
        "questUberStateGroup.getInitialWeaponQuestUberState",
        UberIdentifier::new(14019, 62230),
    ),
    (
        "questUberStateGroup.firstRaceRumorMokiState",
        UberIdentifier::new(14019, 62288),
    ),
    (
        "willowsEndGroup.expOrb",
        UberIdentifier::new(16155, 2065),
    ),
    (
        "willowsEndGroup.fallingPortalB",
        UberIdentifier::new(16155, 2235),
    ),
    (
        "willowsEndGroup.breakableWallA",
        UberIdentifier::new(16155, 3096),
    ),
    (
        "willowsEndGroup.vineEClear",
        UberIdentifier::new(16155, 3588),
    ),
    (
        "willowsEndGroup.arenaPlatform3Destroyed",
        UberIdentifier::new(16155, 3670),
    ),
    (
        "willowsEndGroup.arenaPlatform2Destroyed",
        UberIdentifier::new(16155, 5826),
    ),
    (
        "willowsEndGroup.gorlekOreA",
        UberIdentifier::new(16155, 9230),
    ),
    (
        "willowsEndGroup.fallingPortal",
        UberIdentifier::new(16155, 18906),
    ),
    (
        "willowsEndGroup.creepA",
        UberIdentifier::new(16155, 20672),
    ),
    (
        "willowsEndGroup.chaseSequenceG",
        UberIdentifier::new(16155, 21083),
    ),
    (
        "willowsEndGroup.breakableWallC",
        UberIdentifier::new(16155, 21899),
    ),
    (
        "willowsEndGroup.vineCClear",
        UberIdentifier::new(16155, 24290),
    ),
    (
        "willowsEndGroup.xpOrbA",
        UberIdentifier::new(16155, 25259),
    ),
    (
        "willowsEndGroup.chaseSequenceA",
        UberIdentifier::new(16155, 27024),
    ),
    (
        "willowsEndGroup.vineDClear",
        UberIdentifier::new(16155, 28478),
    ),
    (
        "willowsEndGroup.introPlayed",
        UberIdentifier::new(16155, 32922),
    ),
    (
        "willowsEndGroup.breakableWallC",
        UberIdentifier::new(16155, 33738),
    ),
    (
        "willowsEndGroup.secretWall",
        UberIdentifier::new(16155, 36353),
    ),
    (
        "willowsEndGroup.breakableWallA",
        UberIdentifier::new(16155, 36873),
    ),
    (
        "willowsEndGroup.breakableWallB",
        UberIdentifier::new(16155, 37558),
    ),
    (
        "willowsEndGroup.chaseSequenceC",
        UberIdentifier::new(16155, 37648),
    ),
    (
        "willowsEndGroup.chaseSequenceD",
        UberIdentifier::new(16155, 38867),
    ),
    (
        "willowsEndGroup.gorlekOreA",
        UberIdentifier::new(16155, 38979),
    ),
    (
        "willowsEndGroup.vineGClear",
        UberIdentifier::new(16155, 41488),
    ),
    (
        "willowsEndGroup.secretWallA",
        UberIdentifier::new(16155, 42106),
    ),
    (
        "willowsEndGroup.vineAClear",
        UberIdentifier::new(16155, 42976),
    ),
    (
        "willowsEndGroup.chaseSequenceB",
        UberIdentifier::new(16155, 44311),
    ),
    (
        "willowsEndGroup.arenaPlatform1Destroyed",
        UberIdentifier::new(16155, 45630),
    ),
    (
        "willowsEndGroup.healthCellA",
        UberIdentifier::new(16155, 46270),
    ),
    (
        "willowsEndGroup.expOrbB",
        UberIdentifier::new(16155, 47690),
    ),
    (
        "willowsEndGroup.expOrbA",
        UberIdentifier::new(16155, 49381),
    ),
    (
        "willowsEndGroup.chaseSequenceE",
        UberIdentifier::new(16155, 49408),
    ),
    (
        "willowsEndGroup.expOrb",
        UberIdentifier::new(16155, 49457),
    ),
    (
        "willowsEndGroup.chaseSequenceF",
        UberIdentifier::new(16155, 49744),
    ),
    (
        "willowsEndGroup.breakableWallB",
        UberIdentifier::new(16155, 52848),
    ),
    (
        "willowsEndGroup.groundDestroyed",
        UberIdentifier::new(16155, 54148),
    ),
    (
        "willowsEndGroup.vineBClear",
        UberIdentifier::new(16155, 54940),
    ),
    (
        "willowsEndGroup.xpOrbA",
        UberIdentifier::new(16155, 55446),
    ),
    (
        "willowsEndGroup.fallingPortalA",
        UberIdentifier::new(16155, 55721),
    ),
    (
        "willowsEndGroup.vineHClear",
        UberIdentifier::new(16155, 60752),
    ),
    (
        "willowsEndGroup.arenaPlatform4Destroyed",
        UberIdentifier::new(16155, 63705),
    ),
    (
        "willowsEndGroup.vineFClear",
        UberIdentifier::new(16155, 65277),
    ),
    (
        "willowsEndGroup.savePedestalUberState",
        UberIdentifier::new(16155, 41465),
    ),
    (
        "willowsEndGroup.savePedestalUberState",
        UberIdentifier::new(16155, 50867),
    ),
    (
        "willowsEndGroup.laserShooterBossState",
        UberIdentifier::new(16155, 12971),
    ),
    (
        "willowsEndGroup.petrifiedOwlBossState",
        UberIdentifier::new(16155, 47278),
    ),
    (
        "mouldwoodDepthsGroup.orePickupA",
        UberIdentifier::new(18793, 836),
    ),
    (
        "mouldwoodDepthsGroup.keystone",
        UberIdentifier::new(18793, 1914),
    ),
    (
        "mouldwoodDepthsGroup.expOrbB",
        UberIdentifier::new(18793, 2881),
    ),
    (
        "mouldwoodDepthsGroup.doorWithFourSlots",
        UberIdentifier::new(18793, 3171),
    ),
    (
        "mouldwoodDepthsGroup.blockerWallBroken",
        UberIdentifier::new(18793, 4645),
    ),
    (
        "mouldwoodDepthsGroup.creepDestroyedA",
        UberIdentifier::new(18793, 4664),
    ),
    (
        "mouldwoodDepthsGroup.secretWall",
        UberIdentifier::new(18793, 5315),
    ),
    (
        "mouldwoodDepthsGroup.expOrbB",
        UberIdentifier::new(18793, 5797),
    ),
    (
        "mouldwoodDepthsGroup.expOrbB",
        UberIdentifier::new(18793, 6573),
    ),
    (
        "mouldwoodDepthsGroup.expOrbA",
        UberIdentifier::new(18793, 9251),
    ),
    (
        "mouldwoodDepthsGroup.doorWithFourSlots",
        UberIdentifier::new(18793, 10372),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsGDoorWithTwoSlotsOpened",
        UberIdentifier::new(18793, 10758),
    ),
    (
        "mouldwoodDepthsGroup.creepB",
        UberIdentifier::new(18793, 11676),
    ),
    (
        "mouldwoodDepthsGroup.shrineEnemies",
        UberIdentifier::new(18793, 12512),
    ),
    (
        "mouldwoodDepthsGroup.kwolokCavernsBreakableFloor",
        UberIdentifier::new(18793, 13281),
    ),
    (
        "mouldwoodDepthsGroup.shortcutWall",
        UberIdentifier::new(18793, 13349),
    ),
    (
        "mouldwoodDepthsGroup.darknessLiftedUberState",
        UberIdentifier::new(18793, 13352),
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepB",
        UberIdentifier::new(18793, 13367),
    ),
    (
        "mouldwoodDepthsGroup.leverAndDoorA",
        UberIdentifier::new(18793, 14503),
    ),
    (
        "mouldwoodDepthsGroup.xpOrbC",
        UberIdentifier::new(18793, 15396),
    ),
    (
        "mouldwoodDepthsGroup.arenaBottomBrokenFloor",
        UberIdentifier::new(18793, 15422),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHBreakableWallB",
        UberIdentifier::new(18793, 15855),
    ),
    (
        "mouldwoodDepthsGroup.chamberWebFBroken",
        UberIdentifier::new(18793, 18064),
    ),
    (
        "mouldwoodDepthsGroup.XPOrbA",
        UberIdentifier::new(18793, 18395),
    ),
    (
        "mouldwoodDepthsGroup.chamberWebEBroken",
        UberIdentifier::new(18793, 18563),
    ),
    (
        "mouldwoodDepthsGroup.mediumExpA",
        UberIdentifier::new(18793, 19004),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneBCollected",
        UberIdentifier::new(18793, 20493),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneCCollected",
        UberIdentifier::new(18793, 20959),
    ),
    (
        "mouldwoodDepthsGroup.breakableWallA",
        UberIdentifier::new(18793, 21022),
    ),
    (
        "mouldwoodDepthsGroup.puzzleSolvedSequenceCompleted",
        UberIdentifier::new(18793, 21994),
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepA",
        UberIdentifier::new(18793, 22368),
    ),
    (
        "mouldwoodDepthsGroup.expOrbC",
        UberIdentifier::new(18793, 23799),
    ),
    (
        "mouldwoodDepthsGroup.areaText",
        UberIdentifier::new(18793, 23953),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHKeystoneBCollected",
        UberIdentifier::new(18793, 23986),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodGateOpen",
        UberIdentifier::new(18793, 25789),
    ),
    (
        "mouldwoodDepthsGroup.energyContainerA",
        UberIdentifier::new(18793, 26618),
    ),
    (
        "mouldwoodDepthsGroup.stompableFloor",
        UberIdentifier::new(18793, 27207),
    ),
    (
        "mouldwoodDepthsGroup.energyContainerA",
        UberIdentifier::new(18793, 28175),
    ),
    (
        "mouldwoodDepthsGroup.spiderIntereactedAfterFight",
        UberIdentifier::new(18793, 28205),
    ),
    (
        "mouldwoodDepthsGroup.breakableWebB",
        UberIdentifier::new(18793, 28677),
    ),
    (
        "mouldwoodDepthsGroup.brokenTrunkTop",
        UberIdentifier::new(18793, 28692),
    ),
    (
        "mouldwoodDepthsGroup.creepA",
        UberIdentifier::new(18793, 29066),
    ),
    (
        "mouldwoodDepthsGroup.mediumExpA",
        UberIdentifier::new(18793, 29533),
    ),
    (
        "mouldwoodDepthsGroup.expOrbA",
        UberIdentifier::new(18793, 29979),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHPushBlockPushed",
        UberIdentifier::new(18793, 30627),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneDCollected",
        UberIdentifier::new(18793, 30708),
    ),
    (
        "mouldwoodDepthsGroup.shardSlotUpgradePlaceholder",
        UberIdentifier::new(18793, 31937),
    ),
    (
        "mouldwoodDepthsGroup.chamberWebBBroken",
        UberIdentifier::new(18793, 32305),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneACollected",
        UberIdentifier::new(18793, 32441),
    ),
    (
        "mouldwoodDepthsGroup.doorWithFourSlots",
        UberIdentifier::new(18793, 33471),
    ),
    (
        "mouldwoodDepthsGroup.orePickupA",
        UberIdentifier::new(18793, 35351),
    ),
    (
        "mouldwoodDepthsGroup.expOrb",
        UberIdentifier::new(18793, 38941),
    ),
    (
        "mouldwoodDepthsGroup.creepA",
        UberIdentifier::new(18793, 39232),
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepCTest",
        UberIdentifier::new(18793, 40612),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHDoorWithFourSlotsOpened",
        UberIdentifier::new(18793, 41544),
    ),
    (
        "mouldwoodDepthsGroup.healthCellB",
        UberIdentifier::new(18793, 42235),
    ),
    (
        "mouldwoodDepthsGroup.mediumExpB",
        UberIdentifier::new(18793, 42980),
    ),
    (
        "mouldwoodDepthsGroup.webFallState",
        UberIdentifier::new(18793, 44522),
    ),
    (
        "mouldwoodDepthsGroup.leafPileA",
        UberIdentifier::new(18793, 44773),
    ),
    (
        "mouldwoodDepthsGroup.hintZoneFlash",
        UberIdentifier::new(18793, 45899),
    ),
    (
        "mouldwoodDepthsGroup.bottomRightSmallWall",
        UberIdentifier::new(18793, 45963),
    ),
    (
        "mouldwoodDepthsGroup.verticalFallingTrunk",
        UberIdentifier::new(18793, 49362),
    ),
    (
        "mouldwoodDepthsGroup.XPOrbB",
        UberIdentifier::new(18793, 49526),
    ),
    (
        "mouldwoodDepthsGroup.XPOrbA",
        UberIdentifier::new(18793, 49759),
    ),
    (
        "mouldwoodDepthsGroup.hintZoneFlashCharge",
        UberIdentifier::new(18793, 50745),
    ),
    (
        "mouldwoodDepthsGroup.arenaTrunkBroken",
        UberIdentifier::new(18793, 53347),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHKeystoneACollected",
        UberIdentifier::new(18793, 53953),
    ),
    (
        "mouldwoodDepthsGroup.breakableWebA",
        UberIdentifier::new(18793, 56320),
    ),
    (
        "mouldwoodDepthsGroup.arenaBreakableA",
        UberIdentifier::new(18793, 56666),
    ),
    (
        "mouldwoodDepthsGroup.bottomLeftSmallWall",
        UberIdentifier::new(18793, 56800),
    ),
    (
        "mouldwoodDepthsGroup.keystoneA",
        UberIdentifier::new(18793, 58148),
    ),
    (
        "mouldwoodDepthsGroup.mediumExpC",
        UberIdentifier::new(18793, 58342),
    ),
    (
        "mouldwoodDepthsGroup.healthCellA",
        UberIdentifier::new(18793, 62694),
    ),
    (
        "mouldwoodDepthsGroup.bossReward",
        UberIdentifier::new(18793, 63291),
    ),
    (
        "mouldwoodDepthsGroup.arenaWallMid",
        UberIdentifier::new(18793, 64305),
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHBreakableWallA",
        UberIdentifier::new(18793, 64772),
    ),
    (
        "mouldwoodDepthsGroup.secretWallA",
        UberIdentifier::new(18793, 65202),
    ),
    (
        "mouldwoodDepthsGroup.savePedestalUberState",
        UberIdentifier::new(18793, 38871),
    ),
    (
        "mouldwoodDepthsGroup.savePedestalUberState",
        UberIdentifier::new(18793, 39689),
    ),
    (
        "mouldwoodDepthsGroup.spiderBossState",
        UberIdentifier::new(18793, 26713),
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepBInt",
        UberIdentifier::new(18793, 39667),
    ),
    (
        "eventsUberStateGroup.gumoFreeUberState",
        UberIdentifier::new(19973, 18551),
    ),
    (
        "eventsUberStateGroup.spiritTreeReachedUberState",
        UberIdentifier::new(19973, 22047),
    ),
    (
        "eventsUberStateGroup.mistLiftedUberState",
        UberIdentifier::new(19973, 23591),
    ),
    (
        "eventsUberStateGroup.ginsoTreeKeyUberState",
        UberIdentifier::new(19973, 30524),
    ),
    (
        "eventsUberStateGroup.kwolokDeadUberState",
        UberIdentifier::new(19973, 31305),
    ),
    (
        "eventsUberStateGroup.mountHoruKeyUberState",
        UberIdentifier::new(19973, 38631),
    ),
    (
        "eventsUberStateGroup.kwolokLeftThroneUberState",
        UberIdentifier::new(19973, 45830),
    ),
    (
        "eventsUberStateGroup.gravityActivatedUberState",
        UberIdentifier::new(19973, 49418),
    ),
    (
        "eventsUberStateGroup.ginsoTreeEnteredUberState",
        UberIdentifier::new(19973, 54999),
    ),
    (
        "eventsUberStateGroup.windRestoredUberState",
        UberIdentifier::new(19973, 59537),
    ),
    (
        "eventsUberStateGroup.forlornRuinsKeyUberState",
        UberIdentifier::new(19973, 61347),
    ),
    (
        "windsweptWastesGroupDescriptor.expOrb",
        UberIdentifier::new(20120, 224),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallA",
        UberIdentifier::new(20120, 1348),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbG",
        UberIdentifier::new(20120, 2013),
    ),
    (
        "windsweptWastesGroupDescriptor.areaText",
        UberIdentifier::new(20120, 2552),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbC",
        UberIdentifier::new(20120, 3550),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier::new(20120, 8910),
    ),
    (
        "windsweptWastesGroupDescriptor.breakableWall",
        UberIdentifier::new(20120, 9095),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbB",
        UberIdentifier::new(20120, 10397),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbD",
        UberIdentifier::new(20120, 10801),
    ),
    (
        "windsweptWastesGroupDescriptor.energyHalfCell",
        UberIdentifier::new(20120, 11785),
    ),
    (
        "windsweptWastesGroupDescriptor.leverStateA",
        UberIdentifier::new(20120, 12902),
    ),
    (
        "windsweptWastesGroupDescriptor.healthContainer",
        UberIdentifier::new(20120, 12941),
    ),
    (
        "windsweptWastesGroupDescriptor.projectileBreakableWall",
        UberIdentifier::new(20120, 16172),
    ),
    (
        "windsweptWastesGroupDescriptor.digHint",
        UberIdentifier::new(20120, 16309),
    ),
    (
        "windsweptWastesGroupDescriptor.expOrbE",
        UberIdentifier::new(20120, 17798),
    ),
    (
        "windsweptWastesGroupDescriptor.lifeHalfCell",
        UberIdentifier::new(20120, 18965),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbB",
        UberIdentifier::new(20120, 19113),
    ),
    (
        "windsweptWastesGroupDescriptor.energyOrbA",
        UberIdentifier::new(20120, 22354),
    ),
    (
        "windsweptWastesGroupDescriptor.digDashHint",
        UberIdentifier::new(20120, 24078),
    ),
    (
        "windsweptWastesGroupDescriptor.drillableWallA",
        UberIdentifier::new(20120, 24774),
    ),
    (
        "windsweptWastesGroupDescriptor.e3DesertG_clone0_KeystoneDoor",
        UberIdentifier::new(20120, 28786),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbE",
        UberIdentifier::new(20120, 30358),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier::new(20120, 30740),
    ),
    (
        "windsweptWastesGroupDescriptor.drillableBlockerA",
        UberIdentifier::new(20120, 31180),
    ),
    (
        "windsweptWastesGroupDescriptor.expOrbB",
        UberIdentifier::new(20120, 33275),
    ),
    (
        "windsweptWastesGroupDescriptor.shardA",
        UberIdentifier::new(20120, 33292),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallC",
        UberIdentifier::new(20120, 33294),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallB",
        UberIdentifier::new(20120, 33775),
    ),
    (
        "windsweptWastesGroupDescriptor.drillWallA",
        UberIdentifier::new(20120, 36758),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier::new(20120, 36805),
    ),
    (
        "windsweptWastesGroupDescriptor.gorlekOreB",
        UberIdentifier::new(20120, 40245),
    ),
    (
        "windsweptWastesGroupDescriptor.hintZoneA",
        UberIdentifier::new(20120, 40816),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier::new(20120, 42393),
    ),
    (
        "windsweptWastesGroupDescriptor.shootablePod",
        UberIdentifier::new(20120, 43099),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWall",
        UberIdentifier::new(20120, 43231),
    ),
    (
        "windsweptWastesGroupDescriptor.gorlekOre",
        UberIdentifier::new(20120, 46919),
    ),
    (
        "windsweptWastesGroupDescriptor.verticalPlatformLeverA",
        UberIdentifier::new(20120, 48009),
    ),
    (
        "windsweptWastesGroupDescriptor.expOrbD",
        UberIdentifier::new(20120, 48829),
    ),
    (
        "windsweptWastesGroupDescriptor.hintZoneB",
        UberIdentifier::new(20120, 49950),
    ),
    (
        "windsweptWastesGroupDescriptor.shardA",
        UberIdentifier::new(20120, 49985),
    ),
    (
        "windsweptWastesGroupDescriptor.energyContainer",
        UberIdentifier::new(20120, 50026),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallA",
        UberIdentifier::new(20120, 51985),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrb",
        UberIdentifier::new(20120, 52812),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWall",
        UberIdentifier::new(20120, 54936),
    ),
    (
        "windsweptWastesGroupDescriptor.creepA",
        UberIdentifier::new(20120, 55057),
    ),
    (
        "windsweptWastesGroupDescriptor.wispSequencePlayedOut",
        UberIdentifier::new(20120, 55196),
    ),
    (
        "windsweptWastesGroupDescriptor.gorlekOre",
        UberIdentifier::new(20120, 55303),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallE",
        UberIdentifier::new(20120, 55388),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier::new(20120, 57133),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbB",
        UberIdentifier::new(20120, 57781),
    ),
    (
        "windsweptWastesGroupDescriptor.halfLifeCell",
        UberIdentifier::new(20120, 59046),
    ),
    (
        "windsweptWastesGroupDescriptor.breakableWallA",
        UberIdentifier::new(20120, 59275),
    ),
    (
        "windsweptWastesGroupDescriptor.doorClosingPlayed",
        UberIdentifier::new(20120, 60953),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallD",
        UberIdentifier::new(20120, 60960),
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallF",
        UberIdentifier::new(20120, 61572),
    ),
    (
        "windsweptWastesGroupDescriptor.lifeCellA",
        UberIdentifier::new(20120, 62264),
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbF",
        UberIdentifier::new(20120, 63310),
    ),
    (
        "windsweptWastesGroupDescriptor.savePedestalUberState",
        UberIdentifier::new(20120, 41398),
    ),
    (
        "windsweptWastesGroupDescriptor.savePedestalUberState",
        UberIdentifier::new(20120, 49994),
    ),
    (
        "uiGroup.displayedSpiritWellFirstUseHint",
        UberIdentifier::new(20190, 31212),
    ),
    (
        "minesUberStateGroup.stompableFloorB",
        UberIdentifier::new(21194, 6799),
    ),
    (
        "minesUberStateGroup.collectableC",
        UberIdentifier::new(21194, 7318),
    ),
    (
        "minesUberStateGroup.spiritShardA",
        UberIdentifier::new(21194, 11371),
    ),
    (
        "minesUberStateGroup.collectableA",
        UberIdentifier::new(21194, 16526),
    ),
    (
        "minesUberStateGroup.grolDefeated",
        UberIdentifier::new(21194, 18508),
    ),
    (
        "minesUberStateGroup.collectableB",
        UberIdentifier::new(21194, 26302),
    ),
    (
        "minesUberStateGroup.xpOrbA",
        UberIdentifier::new(21194, 27102),
    ),
    (
        "minesUberStateGroup.memoriesPlayed",
        UberIdentifier::new(21194, 29515),
    ),
    (
        "minesUberStateGroup.crusherActivated",
        UberIdentifier::new(21194, 29822),
    ),
    (
        "minesUberStateGroup.elevatorDoorsBottom",
        UberIdentifier::new(21194, 35345),
    ),
    (
        "minesUberStateGroup.stompableFloorA",
        UberIdentifier::new(21194, 36700),
    ),
    (
        "minesUberStateGroup.grolCuredIntroDialoguePlayed",
        UberIdentifier::new(21194, 38411),
    ),
    (
        "minesUberStateGroup.elevatorDoorsTop",
        UberIdentifier::new(21194, 43575),
    ),
    (
        "minesUberStateGroup.breakableWall",
        UberIdentifier::new(21194, 48792),
    ),
    (
        "minesUberStateGroup.enemyDoor",
        UberIdentifier::new(21194, 52416),
    ),
    (
        "minesUberStateGroup.leverA",
        UberIdentifier::new(21194, 63648),
    ),
    (
        "minesUberStateGroup.savePedestalUberState",
        UberIdentifier::new(21194, 685),
    ),
    (
        "minesUberStateGroup.savePedestalUberState",
        UberIdentifier::new(21194, 63334),
    ),
    (
        "minesUberStateGroup.gateState",
        UberIdentifier::new(21194, 17773),
    ),
    (
        "minesUberStateGroup.minesElevatorUberState",
        UberIdentifier::new(21194, 34225),
    ),
    (
        "swampStateGroup.boneBridgeBroken",
        UberIdentifier::new(21786, 808),
    ),
    (
        "swampStateGroup.creepDoorD",
        UberIdentifier::new(21786, 876),
    ),
    (
        "swampStateGroup.gorlekOreA",
        UberIdentifier::new(21786, 2046),
    ),
    (
        "swampStateGroup.laserPuzzleSolved",
        UberIdentifier::new(21786, 2852),
    ),
    (
        "swampStateGroup.enemyRoom",
        UberIdentifier::new(21786, 2869),
    ),
    (
        "swampStateGroup.xpOrbA",
        UberIdentifier::new(21786, 6987),
    ),
    (
        "swampStateGroup.stompableFloor",
        UberIdentifier::new(21786, 6994),
    ),
    (
        "swampStateGroup.halfEnergyCellA",
        UberIdentifier::new(21786, 7152),
    ),
    (
        "swampStateGroup.mediumExpA",
        UberIdentifier::new(21786, 7709),
    ),
    (
        "swampStateGroup.mediumExpC",
        UberIdentifier::new(21786, 7871),
    ),
    (
        "swampStateGroup.shardSlotA",
        UberIdentifier::new(21786, 9270),
    ),
    (
        "swampStateGroup.creepDoorB",
        UberIdentifier::new(21786, 9402),
    ),
    (
        "swampStateGroup.energyHalfCellA",
        UberIdentifier::new(21786, 10295),
    ),
    (
        "swampStateGroup.largeExpA",
        UberIdentifier::new(21786, 10413),
    ),
    (
        "swampStateGroup.attackableSwitchA",
        UberIdentifier::new(21786, 10467),
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier::new(21786, 11343),
    ),
    (
        "swampStateGroup.spiritShardPickupA",
        UberIdentifier::new(21786, 12077),
    ),
    (
        "swampStateGroup.xpOrbC",
        UberIdentifier::new(21786, 16206),
    ),
    (
        "swampStateGroup.energyHalfCell",
        UberIdentifier::new(21786, 17920),
    ),
    (
        "swampStateGroup.areaText",
        UberIdentifier::new(21786, 17957),
    ),
    (
        "swampStateGroup.shardSlotUpgradePlaceholder",
        UberIdentifier::new(21786, 18109),
    ),
    (
        "swampStateGroup.mediumExpB",
        UberIdentifier::new(21786, 19679),
    ),
    (
        "swampStateGroup.creepTreeC",
        UberIdentifier::new(21786, 20144),
    ),
    (
        "swampStateGroup.xpOrbB",
        UberIdentifier::new(21786, 20160),
    ),
    (
        "swampStateGroup.lifeCellA",
        UberIdentifier::new(21786, 20194),
    ),
    (
        "swampStateGroup.hintZone",
        UberIdentifier::new(21786, 20615),
    ),
    (
        "swampStateGroup.largeExpA",
        UberIdentifier::new(21786, 21727),
    ),
    (
        "swampStateGroup.keyStone",
        UberIdentifier::new(21786, 22068),
    ),
    (
        "swampStateGroup.playedOutKeystoneSequence",
        UberIdentifier::new(21786, 22367),
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier::new(21786, 22570),
    ),
    (
        "swampStateGroup.mediumExpA",
        UberIdentifier::new(21786, 23154),
    ),
    (
        "swampStateGroup.creepDoor",
        UberIdentifier::new(21786, 23177),
    ),
    (
        "swampStateGroup.nightCrawlerEscaped",
        UberIdentifier::new(21786, 25095),
    ),
    (
        "swampStateGroup.creepDoorA",
        UberIdentifier::new(21786, 25147),
    ),
    (
        "swampStateGroup.creepDoorE",
        UberIdentifier::new(21786, 25291),
    ),
    (
        "swampStateGroup.healthContainerA",
        UberIdentifier::new(21786, 25761),
    ),
    (
        "swampStateGroup.interactedWithOpher",
        UberIdentifier::new(21786, 26462),
    ),
    (
        "swampStateGroup.keyStoneA",
        UberIdentifier::new(21786, 27433),
    ),
    (
        "swampStateGroup.halfHealthCellA",
        UberIdentifier::new(21786, 28908),
    ),
    (
        "swampStateGroup.creepDoorB",
        UberIdentifier::new(21786, 29636),
    ),
    (
        "swampStateGroup.gorlekOreA",
        UberIdentifier::new(21786, 29892),
    ),
    (
        "swampStateGroup.spiritShardA",
        UberIdentifier::new(21786, 30305),
    ),
    (
        "swampStateGroup.nightCrawlerChaseStarted",
        UberIdentifier::new(21786, 30656),
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier::new(21786, 30928),
    ),
    (
        "swampStateGroup.hintZone",
        UberIdentifier::new(21786, 31430),
    ),
    (
        "swampStateGroup.enemyDoorA",
        UberIdentifier::new(21786, 32430),
    ),
    (
        "swampStateGroup.creepTreeD",
        UberIdentifier::new(21786, 32463),
    ),
    (
        "swampStateGroup.creepDoor",
        UberIdentifier::new(21786, 33430),
    ),
    (
        "swampStateGroup.breakableWallB",
        UberIdentifier::new(21786, 34008),
    ),
    (
        "swampStateGroup.creepDoorA",
        UberIdentifier::new(21786, 35166),
    ),
    (
        "swampStateGroup.creepDoorC",
        UberIdentifier::new(21786, 35260),
    ),
    (
        "swampStateGroup.attackableSwitchC",
        UberIdentifier::new(21786, 35350),
    ),
    (
        "swampStateGroup.enemyArenaComplete",
        UberIdentifier::new(21786, 35598),
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier::new(21786, 35925),
    ),
    (
        "swampStateGroup.keyStoneB",
        UberIdentifier::new(21786, 37225),
    ),
    (
        "swampStateGroup.creepTreeE",
        UberIdentifier::new(21786, 37833),
    ),
    (
        "swampStateGroup.hintZone",
        UberIdentifier::new(21786, 38342),
    ),
    (
        "swampStateGroup.bladeRitualFinished",
        UberIdentifier::new(21786, 38475),
    ),
    (
        "swampStateGroup.springCreep",
        UberIdentifier::new(21786, 39804),
    ),
    (
        "swampStateGroup.nightCrawlerDefeated",
        UberIdentifier::new(21786, 40322),
    ),
    (
        "swampStateGroup.breakableWall",
        UberIdentifier::new(21786, 40424),
    ),
    (
        "swampStateGroup.doorBState",
        UberIdentifier::new(21786, 41817),
    ),
    (
        "swampStateGroup.swampTorchIntroductionADoorWithTwoSlotsBooleanDescriptor",
        UberIdentifier::new(21786, 42309),
    ),
    (
        "swampStateGroup.watermillDiscovered",
        UberIdentifier::new(21786, 43216),
    ),
    (
        "swampStateGroup.xpOrbB",
        UberIdentifier::new(21786, 43668),
    ),
    (
        "swampStateGroup.energyContainerA",
        UberIdentifier::new(21786, 44157),
    ),
    (
        "swampStateGroup.secretWall",
        UberIdentifier::new(21786, 44253),
    ),
    (
        "swampStateGroup.creepTreeA",
        UberIdentifier::new(21786, 44431),
    ),
    (
        "swampStateGroup.attackableSwitchB",
        UberIdentifier::new(21786, 45648),
    ),
    (
        "swampStateGroup.nightcrawlerTeaseTimelinePlayed",
        UberIdentifier::new(21786, 46536),
    ),
    (
        "swampStateGroup.swampNightcrawlerCavernADoorWithTwoSlotsBooleanDescriptor",
        UberIdentifier::new(21786, 47445),
    ),
    (
        "swampStateGroup.torchHolded",
        UberIdentifier::new(21786, 47458),
    ),
    (
        "swampStateGroup.creepDoorB",
        UberIdentifier::new(21786, 47644),
    ),
    (
        "swampStateGroup.finishedIntroTop",
        UberIdentifier::new(21786, 48748),
    ),
    (
        "swampStateGroup.smallExpA",
        UberIdentifier::new(21786, 49485),
    ),
    (
        "swampStateGroup.mediumExpA",
        UberIdentifier::new(21786, 50255),
    ),
    (
        "swampStateGroup.swampWalljumpChallengeBKeystoneACollected",
        UberIdentifier::new(21786, 50281),
    ),
    (
        "swampStateGroup.leverA",
        UberIdentifier::new(21786, 50432),
    ),
    (
        "swampStateGroup.leverAndDoor",
        UberIdentifier::new(21786, 50453),
    ),
    (
        "swampStateGroup.doorAState",
        UberIdentifier::new(21786, 50691),
    ),
    (
        "swampStateGroup.nightcrawlerBridgeBrokenA",
        UberIdentifier::new(21786, 50994),
    ),
    (
        "swampStateGroup.powlTeaseTriggered",
        UberIdentifier::new(21786, 51018),
    ),
    (
        "swampStateGroup.smallExpA",
        UberIdentifier::new(21786, 52026),
    ),
    (
        "swampStateGroup.leverGateinkwaterMarsh",
        UberIdentifier::new(21786, 52815),
    ),
    (
        "swampStateGroup.creepDoorA",
        UberIdentifier::new(21786, 53932),
    ),
    (
        "swampStateGroup.elevatorDown",
        UberIdentifier::new(21786, 55881),
    ),
    (
        "swampStateGroup.gateUberState",
        UberIdentifier::new(21786, 58612),
    ),
    (
        "swampStateGroup.expOrb",
        UberIdentifier::new(21786, 59513),
    ),
    (
        "swampStateGroup.breakableBridgeBroken",
        UberIdentifier::new(21786, 59922),
    ),
    (
        "swampStateGroup.doorWithTwoSlots",
        UberIdentifier::new(21786, 59990),
    ),
    (
        "swampStateGroup.healthContainerA",
        UberIdentifier::new(21786, 60210),
    ),
    (
        "swampStateGroup.doorFourSlots",
        UberIdentifier::new(21786, 60616),
    ),
    (
        "swampStateGroup.ottersLeadToSpiritBlade",
        UberIdentifier::new(21786, 61644),
    ),
    (
        "swampStateGroup.halfEnergyCellA",
        UberIdentifier::new(21786, 61706),
    ),
    (
        "swampStateGroup.stompableFloor",
        UberIdentifier::new(21786, 61900),
    ),
    (
        "swampStateGroup.xpOrbA",
        UberIdentifier::new(21786, 63072),
    ),
    (
        "swampStateGroup.spiritShardA",
        UberIdentifier::new(21786, 63545),
    ),
    (
        "swampStateGroup.swampWalljumpChallengeBKeystoneBCollected",
        UberIdentifier::new(21786, 64677),
    ),
    (
        "swampStateGroup.creepTreeB",
        UberIdentifier::new(21786, 65235),
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier::new(21786, 3714),
    ),
    (
        "swampStateGroup.savePedestalSwampIntroTop",
        UberIdentifier::new(21786, 10185),
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier::new(21786, 12914),
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier::new(21786, 38720),
    ),
    (
        "swampStateGroup.savePedestalUberState",
        UberIdentifier::new(21786, 38941),
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier::new(21786, 50820),
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier::new(21786, 56901),
    ),
    (
        "swampStateGroup.pushBlockState",
        UberIdentifier::new(21786, 22091),
    ),
    (
        "pickupsGroup.hollowEnergyShardPickup",
        UberIdentifier::new(23987, 897),
    ),
    (
        "pickupsGroup.spiritPowerShardPickup",
        UberIdentifier::new(23987, 986),
    ),
    (
        "pickupsGroup.recklessShardPickup",
        UberIdentifier::new(23987, 9864),
    ),
    (
        "pickupsGroup.ultraLeashShardPickup",
        UberIdentifier::new(23987, 12104),
    ),
    (
        "pickupsGroup.energyCell",
        UberIdentifier::new(23987, 12746),
    ),
    (
        "pickupsGroup.focusShardPickup",
        UberIdentifier::new(23987, 14014),
    ),
    (
        "pickupsGroup.secretShardPickup",
        UberIdentifier::new(23987, 14832),
    ),
    (
        "pickupsGroup.untouchableShardPickup",
        UberIdentifier::new(23987, 19630),
    ),
    (
        "pickupsGroup.spiritMagnetShardPickup",
        UberIdentifier::new(23987, 20915),
    ),
    (
        "pickupsGroup.chainLightningPickup",
        UberIdentifier::new(23987, 23015),
    ),
    (
        "pickupsGroup.recycleShardPickup",
        UberIdentifier::new(23987, 25183),
    ),
    (
        "pickupsGroup.ultraBashShardPickup",
        UberIdentifier::new(23987, 25996),
    ),
    (
        "pickupsGroup.glueShardPickup",
        UberIdentifier::new(23987, 27134),
    ),
    (
        "pickupsGroup.counterstrikeShardPickup",
        UberIdentifier::new(23987, 31426),
    ),
    (
        "pickupsGroup.fractureShardPickup",
        UberIdentifier::new(23987, 36359),
    ),
    (
        "pickupsGroup.energyEfficiencyShardPickup",
        UberIdentifier::new(23987, 46461),
    ),
    (
        "pickupsGroup.aggressorShardPickup",
        UberIdentifier::new(23987, 48605),
    ),
    (
        "pickupsGroup.lastResortShardPickup",
        UberIdentifier::new(23987, 50364),
    ),
    (
        "pickupsGroup.bloodPactShardPickup",
        UberIdentifier::new(23987, 50415),
    ),
    (
        "pickupsGroup.vitalityLuckShardPickup",
        UberIdentifier::new(23987, 53934),
    ),
    (
        "pickupsGroup.barrierShardPickup",
        UberIdentifier::new(23987, 59173),
    ),
    (
        "pickupsGroup.frenzyShardPickup",
        UberIdentifier::new(23987, 61017),
    ),
    (
        "pickupsGroup.splinterShardPickup",
        UberIdentifier::new(23987, 62973),
    ),
    (
        "howlsOriginGroup.secretWallA",
        UberIdentifier::new(24922, 2524),
    ),
    (
        "howlsOriginGroup.expOrbA",
        UberIdentifier::new(24922, 8568),
    ),
    (
        "howlsOriginGroup.bellPuzzleSolved",
        UberIdentifier::new(24922, 13349),
    ),
    (
        "howlsOriginGroup.xpOrbA",
        UberIdentifier::new(24922, 13921),
    ),
    (
        "howlsOriginGroup.shardSlotUpgradePlaceholder",
        UberIdentifier::new(24922, 13993),
    ),
    (
        "howlsOriginGroup.portalsLifted",
        UberIdentifier::new(24922, 16603),
    ),
    (
        "howlsOriginGroup.smallExpA",
        UberIdentifier::new(24922, 32076),
    ),
    (
        "howlsOriginGroup.keystoneB",
        UberIdentifier::new(24922, 33535),
    ),
    (
        "howlsOriginGroup.keystoneA",
        UberIdentifier::new(24922, 34250),
    ),
    (
        "howlsOriginGroup.shrineArena",
        UberIdentifier::new(24922, 45011),
    ),
    (
        "howlsOriginGroup.interactedWithTokk",
        UberIdentifier::new(24922, 45740),
    ),
    (
        "howlsOriginGroup.spiritShard",
        UberIdentifier::new(24922, 46311),
    ),
    (
        "howlsOriginGroup.keystoneA",
        UberIdentifier::new(24922, 47244),
    ),
    (
        "howlsOriginGroup.breakableWallA",
        UberIdentifier::new(24922, 50740),
    ),
    (
        "howlsOriginGroup.bellPuzzleBSolved",
        UberIdentifier::new(24922, 59146),
    ),
    (
        "howlsOriginGroup.keystoneA",
        UberIdentifier::new(24922, 60358),
    ),
    (
        "howlsOriginGroup.largeExpA",
        UberIdentifier::new(24922, 62138),
    ),
    (
        "howlsOriginGroup.howlOriginEntranceSavePedestal",
        UberIdentifier::new(24922, 42531),
    ),
    (
        "convertedSetupsGymGroup.blowableFlameToggle",
        UberIdentifier::new(26019, 971),
    ),
    (
        "convertedSetupsGymGroup.secretWallA",
        UberIdentifier::new(26019, 1274),
    ),
    (
        "convertedSetupsGymGroup.horizontalDoorState",
        UberIdentifier::new(26019, 4052),
    ),
    (
        "convertedSetupsGymGroup.creepDoorD",
        UberIdentifier::new(26019, 4231),
    ),
    (
        "convertedSetupsGymGroup.secretWall",
        UberIdentifier::new(26019, 5259),
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsLever",
        UberIdentifier::new(26019, 6406),
    ),
    (
        "convertedSetupsGymGroup.creepD",
        UberIdentifier::new(26019, 7636),
    ),
    (
        "convertedSetupsGymGroup.mediumExpOrb",
        UberIdentifier::new(26019, 10086),
    ),
    (
        "convertedSetupsGymGroup.snowPileB",
        UberIdentifier::new(26019, 11133),
    ),
    (
        "convertedSetupsGymGroup.elevatorLever",
        UberIdentifier::new(26019, 11592),
    ),
    (
        "convertedSetupsGymGroup.stompableFloor",
        UberIdentifier::new(26019, 12371),
    ),
    (
        "convertedSetupsGymGroup.creepDoorA",
        UberIdentifier::new(26019, 13586),
    ),
    (
        "convertedSetupsGymGroup.desertBreakableWall",
        UberIdentifier::new(26019, 14277),
    ),
    (
        "convertedSetupsGymGroup.creepC",
        UberIdentifier::new(26019, 15381),
    ),
    (
        "convertedSetupsGymGroup.stompableFloorB",
        UberIdentifier::new(26019, 18425),
    ),
    (
        "convertedSetupsGymGroup.cordycepsBreakableWall",
        UberIdentifier::new(26019, 21522),
    ),
    (
        "convertedSetupsGymGroup.watermillEnemyDoor",
        UberIdentifier::new(26019, 23282),
    ),
    (
        "convertedSetupsGymGroup.leverAndDoor",
        UberIdentifier::new(26019, 23382),
    ),
    (
        "convertedSetupsGymGroup.weepingRidgeBreakableWall",
        UberIdentifier::new(26019, 25103),
    ),
    (
        "convertedSetupsGymGroup.petrifiedForestBreakableWall",
        UberIdentifier::new(26019, 26714),
    ),
    (
        "convertedSetupsGymGroup.leafPile",
        UberIdentifier::new(26019, 27176),
    ),
    (
        "convertedSetupsGymGroup.enemyDoor",
        UberIdentifier::new(26019, 28367),
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsBreakableWall",
        UberIdentifier::new(26019, 28678),
    ),
    (
        "convertedSetupsGymGroup.enemyDoorA",
        UberIdentifier::new(26019, 29970),
    ),
    (
        "convertedSetupsGymGroup.keyStoneYesCheckpoint",
        UberIdentifier::new(26019, 30549),
    ),
    (
        "convertedSetupsGymGroup.watermillBreakableWall",
        UberIdentifier::new(26019, 32221),
    ),
    (
        "convertedSetupsGymGroup.watermillBreakableWallUnderwater",
        UberIdentifier::new(26019, 33339),
    ),
    (
        "convertedSetupsGymGroup.lagoonEnemyDoor",
        UberIdentifier::new(26019, 33392),
    ),
    (
        "convertedSetupsGymGroup.desertRuinsBreakableWall",
        UberIdentifier::new(26019, 33510),
    ),
    (
        "convertedSetupsGymGroup.energyContainer",
        UberIdentifier::new(26019, 34752),
    ),
    (
        "convertedSetupsGymGroup.creepDoorC",
        UberIdentifier::new(26019, 34818),
    ),
    (
        "convertedSetupsGymGroup.snowPileA",
        UberIdentifier::new(26019, 35001),
    ),
    (
        "convertedSetupsGymGroup.classicShootableCreepDoor",
        UberIdentifier::new(26019, 37244),
    ),
    (
        "convertedSetupsGymGroup.snowPile",
        UberIdentifier::new(26019, 38710),
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsBreakableWallLock",
        UberIdentifier::new(26019, 38743),
    ),
    (
        "convertedSetupsGymGroup.keyStoneNoCheckpoint",
        UberIdentifier::new(26019, 38761),
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsBreakableWall2",
        UberIdentifier::new(26019, 40296),
    ),
    (
        "convertedSetupsGymGroup.winterForestBreakableWall",
        UberIdentifier::new(26019, 40553),
    ),
    (
        "convertedSetupsGymGroup.creepDoorB",
        UberIdentifier::new(26019, 44556),
    ),
    (
        "convertedSetupsGymGroup.creepE",
        UberIdentifier::new(26019, 47055),
    ),
    (
        "convertedSetupsGymGroup.secretWallWithLock",
        UberIdentifier::new(26019, 47874),
    ),
    (
        "convertedSetupsGymGroup.skillPointOrb",
        UberIdentifier::new(26019, 49127),
    ),
    (
        "convertedSetupsGymGroup.springCreep",
        UberIdentifier::new(26019, 51496),
    ),
    (
        "convertedSetupsGymGroup.enemyDoor",
        UberIdentifier::new(26019, 52684),
    ),
    (
        "convertedSetupsGymGroup.spiritShardPickup",
        UberIdentifier::new(26019, 53374),
    ),
    (
        "convertedSetupsGymGroup.drillZone",
        UberIdentifier::new(26019, 53543),
    ),
    (
        "convertedSetupsGymGroup.snowPileA",
        UberIdentifier::new(26019, 54405),
    ),
    (
        "convertedSetupsGymGroup.fourSlotDoor",
        UberIdentifier::new(26019, 54578),
    ),
    (
        "convertedSetupsGymGroup.minesBreakableWall",
        UberIdentifier::new(26019, 58058),
    ),
    (
        "convertedSetupsGymGroup.creepB",
        UberIdentifier::new(26019, 58116),
    ),
    (
        "convertedSetupsGymGroup.creepA",
        UberIdentifier::new(26019, 60202),
    ),
    (
        "convertedSetupsGymGroup.creepDoorE",
        UberIdentifier::new(26019, 60454),
    ),
    (
        "convertedSetupsGymGroup.lagoonBreakableWall",
        UberIdentifier::new(26019, 62800),
    ),
    (
        "convertedSetupsGymGroup.twoSlotDoor",
        UberIdentifier::new(26019, 62962),
    ),
    (
        "convertedSetupsGymGroup.swampBreakableWall",
        UberIdentifier::new(26019, 63056),
    ),
    (
        "convertedSetupsGymGroup.largeExpOrb",
        UberIdentifier::new(26019, 64001),
    ),
    (
        "convertedSetupsGymGroup.smallExpOrb",
        UberIdentifier::new(26019, 64961),
    ),
    (
        "convertedSetupsGymGroup.willowsEndSecretWall",
        UberIdentifier::new(26019, 65139),
    ),
    (
        "convertedSetupsGymGroup.xpOrbB",
        UberIdentifier::new(26019, 65172),
    ),
    (
        "convertedSetupsGymGroup.landOnAndSpawnOrbs",
        UberIdentifier::new(26019, 13498),
    ),
    (
        "winterForestGroupDescriptor.breakableFloorB",
        UberIdentifier::new(28287, 3938),
    ),
    (
        "winterForestGroupDescriptor.springBlossomTimelinePlayed",
        UberIdentifier::new(28287, 6764),
    ),
    (
        "winterForestGroupDescriptor.boxA",
        UberIdentifier::new(28287, 10460),
    ),
    (
        "winterForestGroupDescriptor.breakableFloor",
        UberIdentifier::new(28287, 11124),
    ),
    (
        "winterForestGroupDescriptor.mediumExpA",
        UberIdentifier::new(28287, 12866),
    ),
    (
        "winterForestGroupDescriptor.breakableRocksA",
        UberIdentifier::new(28287, 13168),
    ),
    (
        "winterForestGroupDescriptor.mediumExpC",
        UberIdentifier::new(28287, 15252),
    ),
    (
        "winterForestGroupDescriptor.thawStateDescriptor",
        UberIdentifier::new(28287, 16339),
    ),
    (
        "winterForestGroupDescriptor.pressurePlatePuzzleSolved",
        UberIdentifier::new(28287, 22713),
    ),
    (
        "winterForestGroupDescriptor.breakableWallA",
        UberIdentifier::new(28287, 24327),
    ),
    (
        "winterForestGroupDescriptor.breakableFloorA",
        UberIdentifier::new(28287, 26844),
    ),
    (
        "winterForestGroupDescriptor.mediumExpB",
        UberIdentifier::new(28287, 28525),
    ),
    (
        "winterForestGroupDescriptor.secretWallThaw",
        UberIdentifier::new(28287, 30157),
    ),
    (
        "winterForestGroupDescriptor.mediumExpA",
        UberIdentifier::new(28287, 32414),
    ),
    (
        "winterForestGroupDescriptor.hammerWall",
        UberIdentifier::new(28287, 40607),
    ),
    (
        "winterForestGroupDescriptor.breakableWallC",
        UberIdentifier::new(28287, 43506),
    ),
    (
        "winterForestGroupDescriptor.breakableWallB",
        UberIdentifier::new(28287, 51721),
    ),
    (
        "winterForestGroupDescriptor.boxB",
        UberIdentifier::new(28287, 52043),
    ),
    (
        "winterForestGroupDescriptor.leafPileA",
        UberIdentifier::new(28287, 55131),
    ),
    (
        "winterForestGroupDescriptor.spiritShardA",
        UberIdentifier::new(28287, 55482),
    ),
    (
        "winterForestGroupDescriptor.stompableFloor",
        UberIdentifier::new(28287, 57792),
    ),
    (
        "winterForestGroupDescriptor.leafPileB",
        UberIdentifier::new(28287, 61490),
    ),
    (
        "winterForestGroupDescriptor.breakableRocksB",
        UberIdentifier::new(28287, 62332),
    ),
    (
        "winterForestGroupDescriptor.savePedestalUberState",
        UberIdentifier::new(28287, 64528),
    ),
    (
        "baursReachGroup.keystoneB",
        UberIdentifier::new(28895, 1053),
    ),
    (
        "baursReachGroup.powlTeaseTriggered",
        UberIdentifier::new(28895, 2108),
    ),
    (
        "baursReachGroup.largeExpOrb",
        UberIdentifier::new(28895, 2129),
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier::new(28895, 2462),
    ),
    (
        "baursReachGroup.stompableFloorA",
        UberIdentifier::new(28895, 2896),
    ),
    (
        "baursReachGroup.breakableWallB",
        UberIdentifier::new(28895, 2931),
    ),
    (
        "baursReachGroup.mediumExpOrb",
        UberIdentifier::new(28895, 3777),
    ),
    (
        "baursReachGroup.doorWithFourSlots",
        UberIdentifier::new(28895, 4290),
    ),
    (
        "baursReachGroup.xpOrbF",
        UberIdentifier::new(28895, 4301),
    ),
    (
        "baursReachGroup.frozenMokiInteracted",
        UberIdentifier::new(28895, 7152),
    ),
    (
        "baursReachGroup.smallExpB",
        UberIdentifier::new(28895, 7597),
    ),
    (
        "baursReachGroup.breakableRocksA",
        UberIdentifier::new(28895, 7616),
    ),
    (
        "baursReachGroup.breakableRocksK",
        UberIdentifier::new(28895, 7703),
    ),
    (
        "baursReachGroup.stompableFloorA",
        UberIdentifier::new(28895, 8664),
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier::new(28895, 8834),
    ),
    (
        "baursReachGroup.breakableWallB",
        UberIdentifier::new(28895, 8934),
    ),
    (
        "baursReachGroup.expOrbD",
        UberIdentifier::new(28895, 9321),
    ),
    (
        "baursReachGroup.keystoneC",
        UberIdentifier::new(28895, 9949),
    ),
    (
        "baursReachGroup.keystoneC",
        UberIdentifier::new(28895, 10823),
    ),
    (
        "baursReachGroup.energyHalfCell",
        UberIdentifier::new(28895, 10840),
    ),
    (
        "baursReachGroup.breakableRocksG",
        UberIdentifier::new(28895, 11936),
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier::new(28895, 12140),
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier::new(28895, 14264),
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier::new(28895, 17510),
    ),
    (
        "baursReachGroup.afterMoraDeathRetalkPlayed",
        UberIdentifier::new(28895, 17914),
    ),
    (
        "baursReachGroup.keystoneD",
        UberIdentifier::new(28895, 18358),
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier::new(28895, 19041),
    ),
    (
        "baursReachGroup.expOrbE",
        UberIdentifier::new(28895, 19077),
    ),
    (
        "baursReachGroup.breakableRockWall",
        UberIdentifier::new(28895, 20731),
    ),
    (
        "baursReachGroup.stompableGroundA",
        UberIdentifier::new(28895, 22127),
    ),
    (
        "baursReachGroup.keystoneA",
        UberIdentifier::new(28895, 22382),
    ),
    (
        "baursReachGroup.placedCoal",
        UberIdentifier::new(28895, 22695),
    ),
    (
        "baursReachGroup.winterForestBonfire",
        UberIdentifier::new(28895, 22758),
    ),
    (
        "baursReachGroup.mediumExpOrb",
        UberIdentifier::new(28895, 22761),
    ),
    (
        "baursReachGroup.expOrbC",
        UberIdentifier::new(28895, 22959),
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier::new(28895, 23605),
    ),
    (
        "baursReachGroup.breakableRocksH",
        UberIdentifier::new(28895, 23678),
    ),
    (
        "baursReachGroup.gorlekOreA",
        UberIdentifier::new(28895, 23795),
    ),
    (
        "baursReachGroup.smallXPOrbB",
        UberIdentifier::new(28895, 24533),
    ),
    (
        "baursReachGroup.afterKwolokDeathRetalkPlayed",
        UberIdentifier::new(28895, 25315),
    ),
    (
        "baursReachGroup.wispRewardPickup",
        UberIdentifier::new(28895, 25522),
    ),
    (
        "baursReachGroup.energyUpgrade",
        UberIdentifier::new(28895, 27476),
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier::new(28895, 27787),
    ),
    (
        "baursReachGroup.seedPodBroken",
        UberIdentifier::new(28895, 28059),
    ),
    (
        "baursReachGroup.keystoneA",
        UberIdentifier::new(28895, 29898),
    ),
    (
        "baursReachGroup.grenadeLanternsHint",
        UberIdentifier::new(28895, 30189),
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier::new(28895, 30566),
    ),
    (
        "baursReachGroup.breakableWall",
        UberIdentifier::new(28895, 30794),
    ),
    (
        "baursReachGroup.fallingBranch",
        UberIdentifier::new(28895, 31575),
    ),
    (
        "baursReachGroup.areaTextZone",
        UberIdentifier::new(28895, 32092),
    ),
    (
        "baursReachGroup.creepDoor",
        UberIdentifier::new(28895, 32340),
    ),
    (
        "baursReachGroup.closingGate",
        UberIdentifier::new(28895, 32443),
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier::new(28895, 33337),
    ),
    (
        "baursReachGroup.xpOrbE",
        UberIdentifier::new(28895, 33846),
    ),
    (
        "baursReachGroup.breakableWall",
        UberIdentifier::new(28895, 34098),
    ),
    (
        "baursReachGroup.breakableRocksJ",
        UberIdentifier::new(28895, 34461),
    ),
    (
        "baursReachGroup.healthCellA",
        UberIdentifier::new(28895, 34534),
    ),
    (
        "baursReachGroup.smallXPOrbB",
        UberIdentifier::new(28895, 35045),
    ),
    (
        "baursReachGroup.memoriesPlayedOut",
        UberIdentifier::new(28895, 35436),
    ),
    (
        "baursReachGroup.hintZoneA",
        UberIdentifier::new(28895, 35874),
    ),
    (
        "baursReachGroup.xpOrbC",
        UberIdentifier::new(28895, 36231),
    ),
    (
        "baursReachGroup.smallExpA",
        UberIdentifier::new(28895, 36378),
    ),
    (
        "baursReachGroup.secretWallA",
        UberIdentifier::new(28895, 36649),
    ),
    (
        "baursReachGroup.grenadeSwitchA",
        UberIdentifier::new(28895, 37287),
    ),
    (
        "baursReachGroup.keystoneB",
        UberIdentifier::new(28895, 37444),
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier::new(28895, 38049),
    ),
    (
        "baursReachGroup.breakableRocksE",
        UberIdentifier::new(28895, 38120),
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier::new(28895, 38143),
    ),
    (
        "baursReachGroup.breakableRocksF",
        UberIdentifier::new(28895, 38525),
    ),
    (
        "baursReachGroup.gorlekOreA",
        UberIdentifier::new(28895, 39291),
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier::new(28895, 40089),
    ),
    (
        "baursReachGroup.xpOrbC",
        UberIdentifier::new(28895, 40242),
    ),
    (
        "baursReachGroup.healthCellA",
        UberIdentifier::new(28895, 40744),
    ),
    (
        "baursReachGroup.afterAvalancheRetalkPlayed",
        UberIdentifier::new(28895, 41299),
    ),
    (
        "baursReachGroup.hintZone",
        UberIdentifier::new(28895, 41777),
    ),
    (
        "baursReachGroup.enemyArenaComplete",
        UberIdentifier::new(28895, 42209),
    ),
    (
        "baursReachGroup.firePedestal",
        UberIdentifier::new(28895, 43977),
    ),
    (
        "baursReachGroup.xpOrbE",
        UberIdentifier::new(28895, 45066),
    ),
    (
        "baursReachGroup.largeXPOrbA",
        UberIdentifier::new(28895, 45337),
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier::new(28895, 46293),
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier::new(28895, 46404),
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier::new(28895, 46711),
    ),
    (
        "baursReachGroup.breakableRocksI",
        UberIdentifier::new(28895, 46875),
    ),
    (
        "baursReachGroup.orePlaceholder",
        UberIdentifier::new(28895, 47529),
    ),
    (
        "baursReachGroup.creepA",
        UberIdentifier::new(28895, 48186),
    ),
    (
        "baursReachGroup.grenadeSwitchA",
        UberIdentifier::new(28895, 48757),
    ),
    (
        "baursReachGroup.breakyBridge",
        UberIdentifier::new(28895, 49329),
    ),
    (
        "baursReachGroup.keystoneGate",
        UberIdentifier::new(28895, 49900),
    ),
    (
        "baursReachGroup.breakableRocksB",
        UberIdentifier::new(28895, 49997),
    ),
    (
        "baursReachGroup.keystoneD",
        UberIdentifier::new(28895, 50368),
    ),
    (
        "baursReachGroup.blowableFlameB",
        UberIdentifier::new(28895, 51471),
    ),
    (
        "baursReachGroup.healthHalfContainer",
        UberIdentifier::new(28895, 51853),
    ),
    (
        "baursReachGroup.kindledFire",
        UberIdentifier::new(28895, 52440),
    ),
    (
        "baursReachGroup.seenLoremasterMenu",
        UberIdentifier::new(28895, 53166),
    ),
    (
        "baursReachGroup.xpOrbC",
        UberIdentifier::new(28895, 53283),
    ),
    (
        "baursReachGroup.smallXPOrbA",
        UberIdentifier::new(28895, 54373),
    ),
    (
        "baursReachGroup.smallXPOrbA",
        UberIdentifier::new(28895, 55384),
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier::new(28895, 56062),
    ),
    (
        "baursReachGroup.secretWallBaur",
        UberIdentifier::new(28895, 57743),
    ),
    (
        "baursReachGroup.doorOpened",
        UberIdentifier::new(28895, 58337),
    ),
    (
        "baursReachGroup.gorlekOreA",
        UberIdentifier::new(28895, 58675),
    ),
    (
        "baursReachGroup.expOrbA",
        UberIdentifier::new(28895, 58848),
    ),
    (
        "baursReachGroup.talkedToSleepingBaur",
        UberIdentifier::new(28895, 59287),
    ),
    (
        "baursReachGroup.grenadeSwitchA",
        UberIdentifier::new(28895, 59394),
    ),
    (
        "baursReachGroup.interactedWithCampfire",
        UberIdentifier::new(28895, 59955),
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier::new(28895, 61536),
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier::new(28895, 61789),
    ),
    (
        "baursReachGroup.frozenMokiIceBroken",
        UberIdentifier::new(28895, 61852),
    ),
    (
        "baursReachGroup.breakableRocksC",
        UberIdentifier::new(28895, 61896),
    ),
    (
        "baursReachGroup.afterGoldenSeinRetalkPlayed",
        UberIdentifier::new(28895, 61976),
    ),
    (
        "baursReachGroup.leverSetup",
        UberIdentifier::new(28895, 62198),
    ),
    (
        "baursReachGroup.breakableRocksD",
        UberIdentifier::new(28895, 62643),
    ),
    (
        "baursReachGroup.orePickup",
        UberIdentifier::new(28895, 64226),
    ),
    (
        "baursReachGroup.blowableFlameA",
        UberIdentifier::new(28895, 64742),
    ),
    (
        "baursReachGroup.mediumExpA",
        UberIdentifier::new(28895, 65235),
    ),
    (
        "baursReachGroup.savePedestalUberState",
        UberIdentifier::new(28895, 18910),
    ),
    (
        "baursReachGroup.savePedestalUberState",
        UberIdentifier::new(28895, 54235),
    ),
    (
        "baursReachGroup.interactedWithTokk",
        UberIdentifier::new(28895, 13636),
    ),
    (
        "baursReachGroup.baurNpcState",
        UberIdentifier::new(28895, 29098),
    ),
    (
        "baursReachGroup.mokiNpcState",
        UberIdentifier::new(28895, 12170),
    ),
    (
        "weepingRidgeElevatorFightGroup.willowsEndGateOpened",
        UberIdentifier::new(31136, 3441),
    ),
    (
        "weepingRidgeElevatorFightGroup.areaText",
        UberIdentifier::new(31136, 59099),
    ),
    (
        "achievementsGroup.spiritBladeCollected",
        UberIdentifier::new(33399, 17893),
    ),
    (
        "achievementsGroup.gotHitBySpider",
        UberIdentifier::new(33399, 28382),
    ),
    (
        "achievementsGroup.shardEverEquipped",
        UberIdentifier::new(33399, 34522),
    ),
    (
        "achievementsGroup.spiritLightEverSpent",
        UberIdentifier::new(33399, 50709),
    ),
    (
        "achievementsGroup.poisonousWaterTouched",
        UberIdentifier::new(33399, 58955),
    ),
    (
        "achievementsGroup.enemiesKilledByHazards",
        UberIdentifier::new(33399, 17398),
    ),
    (
        "achievementsGroup.spiritLightGainedCounter",
        UberIdentifier::new(33399, 36285),
    ),
    (
        "achievementsGroup.energyContainersCounter",
        UberIdentifier::new(33399, 41928),
    ),
    (
        "achievementsGroup.healthContainersCounter",
        UberIdentifier::new(33399, 52378),
    ),
    (
        "achievementsGroup.collectablesCounter",
        UberIdentifier::new(33399, 61261),
    ),
    (
        "gameStateGroup.gameFinished",
        UberIdentifier::new(34543, 11226),
    ),
    (
        "gameStateGroup.gameDifficultyMode",
        UberIdentifier::new(34543, 30984),
    ),
    (
        "corruptedPeakGroup.spineStateB",
        UberIdentifier::new(36153, 2824),
    ),
    (
        "corruptedPeakGroup.gorlekOreA",
        UberIdentifier::new(36153, 3013),
    ),
    (
        "corruptedPeakGroup.expOrbB",
        UberIdentifier::new(36153, 3662),
    ),
    (
        "corruptedPeakGroup.weepingRidgeGetChargeJump",
        UberIdentifier::new(36153, 5369),
    ),
    (
        "corruptedPeakGroup.mediumExpA",
        UberIdentifier::new(36153, 5552),
    ),
    (
        "corruptedPeakGroup.xpOrbA",
        UberIdentifier::new(36153, 6682),
    ),
    (
        "corruptedPeakGroup.spineStateA",
        UberIdentifier::new(36153, 8434),
    ),
    (
        "corruptedPeakGroup.xpOrbB",
        UberIdentifier::new(36153, 12077),
    ),
    (
        "corruptedPeakGroup.healthContainerA",
        UberIdentifier::new(36153, 12457),
    ),
    (
        "corruptedPeakGroup.pressurePlatePuzzleA",
        UberIdentifier::new(36153, 14400),
    ),
    (
        "corruptedPeakGroup.pressurePlatePuzzleB",
        UberIdentifier::new(36153, 17818),
    ),
    (
        "corruptedPeakGroup.mediumExpOrbPlaceholder",
        UberIdentifier::new(36153, 18750),
    ),
    (
        "corruptedPeakGroup.breakableWallD",
        UberIdentifier::new(36153, 18883),
    ),
    (
        "corruptedPeakGroup.spineStateC",
        UberIdentifier::new(36153, 20307),
    ),
    (
        "corruptedPeakGroup.breakableRockB",
        UberIdentifier::new(36153, 22461),
    ),
    (
        "corruptedPeakGroup.elevatorCompleteState",
        UberIdentifier::new(36153, 23584),
    ),
    (
        "corruptedPeakGroup.expOrbA",
        UberIdentifier::new(36153, 23902),
    ),
    (
        "corruptedPeakGroup.corruptedPeakSecretWallB",
        UberIdentifier::new(36153, 25095),
    ),
    (
        "corruptedPeakGroup.spellPickup",
        UberIdentifier::new(36153, 30728),
    ),
    (
        "corruptedPeakGroup.expOrb",
        UberIdentifier::new(36153, 36521),
    ),
    (
        "corruptedPeakGroup.spineStateD",
        UberIdentifier::new(36153, 42305),
    ),
    (
        "corruptedPeakGroup.expOrbC",
        UberIdentifier::new(36153, 42589),
    ),
    (
        "corruptedPeakGroup.corruptedPeakSecretWallA",
        UberIdentifier::new(36153, 44835),
    ),
    (
        "corruptedPeakGroup.breakableRockA",
        UberIdentifier::new(36153, 47520),
    ),
    (
        "corruptedPeakGroup.stompableFloorA",
        UberIdentifier::new(36153, 48472),
    ),
    (
        "corruptedPeakGroup.stomperStateB",
        UberIdentifier::new(36153, 51042),
    ),
    (
        "corruptedPeakGroup.expOrb",
        UberIdentifier::new(36153, 53032),
    ),
    (
        "corruptedPeakGroup.stompableFloorC",
        UberIdentifier::new(36153, 55011),
    ),
    (
        "corruptedPeakGroup.mediumExpA",
        UberIdentifier::new(36153, 56157),
    ),
    (
        "corruptedPeakGroup.stompableFloorB",
        UberIdentifier::new(36153, 57716),
    ),
    (
        "corruptedPeakGroup.breakableWall",
        UberIdentifier::new(36153, 60795),
    ),
    (
        "corruptedPeakGroup.stomperStateA",
        UberIdentifier::new(36153, 62883),
    ),
    (
        "corruptedPeakGroup.savePedestalUberState",
        UberIdentifier::new(36153, 43597),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallADestroyedArt",
        UberIdentifier::new(37858, 2615),
    ),
    (
        "waterMillStateGroupDescriptor.smallExpOrb",
        UberIdentifier::new(37858, 2797),
    ),
    (
        "waterMillStateGroupDescriptor.hornbugBreakWall",
        UberIdentifier::new(37858, 3421),
    ),
    (
        "waterMillStateGroupDescriptor.shardSlotUpgrade",
        UberIdentifier::new(37858, 3685),
    ),
    (
        "waterMillStateGroupDescriptor.dashDoor",
        UberIdentifier::new(37858, 6338),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier::new(37858, 8344),
    ),
    (
        "waterMillStateGroupDescriptor.exitDoorOpen",
        UberIdentifier::new(37858, 9487),
    ),
    (
        "waterMillStateGroupDescriptor.orePickupA",
        UberIdentifier::new(37858, 11418),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillBossRoomBarrierOpen",
        UberIdentifier::new(37858, 11772),
    ),
    (
        "waterMillStateGroupDescriptor.finishedWatermillEscape",
        UberIdentifier::new(37858, 12379),
    ),
    (
        "waterMillStateGroupDescriptor.displayedFlingHint",
        UberIdentifier::new(37858, 13968),
    ),
    (
        "waterMillStateGroupDescriptor.spiritShardA",
        UberIdentifier::new(37858, 15961),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillEntranceFallingDiscUberStateDescriptor",
        UberIdentifier::new(37858, 16604),
    ),
    (
        "waterMillStateGroupDescriptor.expOrbA",
        UberIdentifier::new(37858, 16611),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier::new(37858, 19347),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallBDestroyed",
        UberIdentifier::new(37858, 21874),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier::new(37858, 22107),
    ),
    (
        "waterMillStateGroupDescriptor.playedNaruGumoCutaway",
        UberIdentifier::new(37858, 23225),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillEntranceDoorUberStateDescriptor",
        UberIdentifier::new(37858, 23644),
    ),
    (
        "waterMillStateGroupDescriptor.rescuedOpher",
        UberIdentifier::new(37858, 25031),
    ),
    (
        "waterMillStateGroupDescriptor.healthContainerA",
        UberIdentifier::new(37858, 25833),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillVisited",
        UberIdentifier::new(37858, 26885),
    ),
    (
        "waterMillStateGroupDescriptor.poleLowered",
        UberIdentifier::new(37858, 31104),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier::new(37858, 31136),
    ),
    (
        "waterMillStateGroupDescriptor.recedingWater",
        UberIdentifier::new(37858, 31187),
    ),
    (
        "waterMillStateGroupDescriptor.wheelsActivated",
        UberIdentifier::new(37858, 31584),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillBEntranceTriggerUberStateDescriptor",
        UberIdentifier::new(37858, 31962),
    ),
    (
        "waterMillStateGroupDescriptor.doorWithTwoSlotsBooleanDescriptor",
        UberIdentifier::new(37858, 31966),
    ),
    (
        "waterMillStateGroupDescriptor.keystoneA",
        UberIdentifier::new(37858, 32628),
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOreA",
        UberIdentifier::new(37858, 32932),
    ),
    (
        "waterMillStateGroupDescriptor.xpOrbA",
        UberIdentifier::new(37858, 33063),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpB",
        UberIdentifier::new(37858, 33642),
    ),
    (
        "waterMillStateGroupDescriptor.wheelLever",
        UberIdentifier::new(37858, 34433),
    ),
    (
        "waterMillStateGroupDescriptor.enemyDoor",
        UberIdentifier::new(37858, 34619),
    ),
    (
        "waterMillStateGroupDescriptor.spiritShardA",
        UberIdentifier::new(37858, 34646),
    ),
    (
        "waterMillStateGroupDescriptor.enemyDoorA",
        UberIdentifier::new(37858, 37323),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier::new(37858, 41380),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpB",
        UberIdentifier::new(37858, 41911),
    ),
    (
        "waterMillStateGroupDescriptor.keystoneA",
        UberIdentifier::new(37858, 43893),
    ),
    (
        "waterMillStateGroupDescriptor.xpOrbWater",
        UberIdentifier::new(37858, 45656),
    ),
    (
        "waterMillStateGroupDescriptor.smallExpA",
        UberIdentifier::new(37858, 45906),
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOraA",
        UberIdentifier::new(37858, 47533),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpC",
        UberIdentifier::new(37858, 50064),
    ),
    (
        "waterMillStateGroupDescriptor.watermillEntranceTalkedToOpher",
        UberIdentifier::new(37858, 50780),
    ),
    (
        "waterMillStateGroupDescriptor.wheelAActive",
        UberIdentifier::new(37858, 50902),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpB",
        UberIdentifier::new(37858, 52110),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillBossRoomWheelFelt",
        UberIdentifier::new(37858, 52129),
    ),
    (
        "waterMillStateGroupDescriptor.hintZone",
        UberIdentifier::new(37858, 54231),
    ),
    (
        "waterMillStateGroupDescriptor.xpOrbA",
        UberIdentifier::new(37858, 55450),
    ),
    (
        "waterMillStateGroupDescriptor.expOrbB",
        UberIdentifier::new(37858, 55499),
    ),
    (
        "waterMillStateGroupDescriptor.xpOrb",
        UberIdentifier::new(37858, 56444),
    ),
    (
        "waterMillStateGroupDescriptor.energyVessel",
        UberIdentifier::new(37858, 57552),
    ),
    (
        "waterMillStateGroupDescriptor.arenaWheelsActivated",
        UberIdentifier::new(37858, 58000),
    ),
    (
        "waterMillStateGroupDescriptor.smallExpAArt",
        UberIdentifier::new(37858, 58220),
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOreA",
        UberIdentifier::new(37858, 58286),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallADestroyedArtB",
        UberIdentifier::new(37858, 58736),
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOreB",
        UberIdentifier::new(37858, 58846),
    ),
    (
        "waterMillStateGroupDescriptor.shardSlotExpansion",
        UberIdentifier::new(37858, 58947),
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier::new(37858, 59022),
    ),
    (
        "waterMillStateGroupDescriptor.wheelBActive",
        UberIdentifier::new(37858, 60716),
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallADestroyed",
        UberIdentifier::new(37858, 61481),
    ),
    (
        "waterMillStateGroupDescriptor.wheelsActivatedEntry",
        UberIdentifier::new(37858, 64055),
    ),
    (
        "waterMillStateGroupDescriptor.expOrb",
        UberIdentifier::new(37858, 64086),
    ),
    (
        "waterMillStateGroupDescriptor.spiritShardA",
        UberIdentifier::new(37858, 64961),
    ),
    (
        "waterMillStateGroupDescriptor.lifeCellA",
        UberIdentifier::new(37858, 65187),
    ),
    (
        "waterMillStateGroupDescriptor.rotatingEnemyArenaStates",
        UberIdentifier::new(37858, 8487),
    ),
    (
        "waterMillStateGroupDescriptor.watermillEscapeState",
        UberIdentifier::new(37858, 10720),
    ),
    (
        "waterMillStateGroupDescriptor.rotatingEnemyArenaRotationStateController",
        UberIdentifier::new(37858, 34636),
    ),
    (
        "waterMillStateGroupDescriptor.rotationState",
        UberIdentifier::new(37858, 36070),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsARespawnTimer",
        UberIdentifier::new(37858, 5107),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 8675),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 17790),
    ),
    (
        "waterMillStateGroupDescriptor.healthPlantB",
        UberIdentifier::new(37858, 22902),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsA",
        UberIdentifier::new(37858, 24680),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 28311),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 38044),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 44551),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsARespawnTimerB",
        UberIdentifier::new(37858, 48554),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 54186),
    ),
    (
        "waterMillStateGroupDescriptor.healthPlant",
        UberIdentifier::new(37858, 57762),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsB",
        UberIdentifier::new(37858, 60777),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 61727),
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier::new(37858, 62404),
    ),
    (
        "spiderBatTestGroup.roundOneDefeated",
        UberIdentifier::new(42171, 14000),
    ),
    (
        "spiderBatTestGroup.arenaDoorClosed",
        UberIdentifier::new(42171, 26771),
    ),
    (
        "spiderBatTestGroup.allRoundsDefeated",
        UberIdentifier::new(42171, 32228),
    ),
    (
        "spiderBatTestGroup.roundTwoDefeated",
        UberIdentifier::new(42171, 43227),
    ),
    (
        "spiderBatTestGroup.roundThreeDefeated",
        UberIdentifier::new(42171, 56229),
    ),
    (
        "spiderBatTestGroup.enemyArenaState",
        UberIdentifier::new(42171, 53383),
    ),
    (
        "hubUberStateGroup.leafPileB",
        UberIdentifier::new(42178, 3295),
    ),
    (
        "hubUberStateGroup.mediumExpD",
        UberIdentifier::new(42178, 4125),
    ),
    (
        "hubUberStateGroup.leafPileC",
        UberIdentifier::new(42178, 5630),
    ),
    (
        "hubUberStateGroup.stompableFloorEnterHub",
        UberIdentifier::new(42178, 5815),
    ),
    (
        "hubUberStateGroup.mediumExpG",
        UberIdentifier::new(42178, 6117),
    ),
    (
        "hubUberStateGroup.woodCrateC",
        UberIdentifier::new(42178, 9319),
    ),
    (
        "hubUberStateGroup.mediumExpE",
        UberIdentifier::new(42178, 9780),
    ),
    (
        "hubUberStateGroup.leafPileA",
        UberIdentifier::new(42178, 10035),
    ),
    (
        "hubUberStateGroup.hutBExpOrb",
        UberIdentifier::new(42178, 13327),
    ),
    (
        "hubUberStateGroup.mediumExpA",
        UberIdentifier::new(42178, 14903),
    ),
    (
        "hubUberStateGroup.woodCrateA",
        UberIdentifier::new(42178, 15685),
    ),
    (
        "hubUberStateGroup.woodCrateC",
        UberIdentifier::new(42178, 17158),
    ),
    (
        "hubUberStateGroup.smallExpE",
        UberIdentifier::new(42178, 17489),
    ),
    (
        "hubUberStateGroup.drillableWallB",
        UberIdentifier::new(42178, 17732),
    ),
    (
        "hubUberStateGroup.mediumExpF",
        UberIdentifier::new(42178, 18448),
    ),
    (
        "hubUberStateGroup.areaText",
        UberIdentifier::new(42178, 19692),
    ),
    (
        "hubUberStateGroup.woodCrateE",
        UberIdentifier::new(42178, 21105),
    ),
    (
        "hubUberStateGroup.gorlekOreA",
        UberIdentifier::new(42178, 23125),
    ),
    (
        "hubUberStateGroup.hubSpritWellIconVisible",
        UberIdentifier::new(42178, 23193),
    ),
    (
        "hubUberStateGroup.woodCrateC",
        UberIdentifier::new(42178, 26189),
    ),
    (
        "hubUberStateGroup.woodCrateA",
        UberIdentifier::new(42178, 26365),
    ),
    (
        "hubUberStateGroup.gorlekOreB",
        UberIdentifier::new(42178, 27110),
    ),
    (
        "hubUberStateGroup.warpHintShowed",
        UberIdentifier::new(42178, 27777),
    ),
    (
        "hubUberStateGroup.smallExpA",
        UberIdentifier::new(42178, 30206),
    ),
    (
        "hubUberStateGroup.hutDExpOrbB",
        UberIdentifier::new(42178, 30520),
    ),
    (
        "hubUberStateGroup.drillableWallA",
        UberIdentifier::new(42178, 31795),
    ),
    (
        "hubUberStateGroup.smallExpD",
        UberIdentifier::new(42178, 35232),
    ),
    (
        "hubUberStateGroup.woodCrateE",
        UberIdentifier::new(42178, 35855),
    ),
    (
        "hubUberStateGroup.mediumExpA",
        UberIdentifier::new(42178, 36085),
    ),
    (
        "hubUberStateGroup.woodCrateD",
        UberIdentifier::new(42178, 36464),
    ),
    (
        "hubUberStateGroup.fatherMokiGone",
        UberIdentifier::new(42178, 36609),
    ),
    (
        "hubUberStateGroup.smallExpB",
        UberIdentifier::new(42178, 37028),
    ),
    (
        "hubUberStateGroup.mediumExpC",
        UberIdentifier::new(42178, 38743),
    ),
    (
        "hubUberStateGroup.pyreA",
        UberIdentifier::new(42178, 38905),
    ),
    (
        "hubUberStateGroup.mediumExpB",
        UberIdentifier::new(42178, 40609),
    ),
    (
        "hubUberStateGroup.smallExpH",
        UberIdentifier::new(42178, 42762),
    ),
    (
        "hubUberStateGroup.smallExpG",
        UberIdentifier::new(42178, 44748),
    ),
    (
        "hubUberStateGroup.woodCrateB",
        UberIdentifier::new(42178, 47152),
    ),
    (
        "hubUberStateGroup.woodCrateA",
        UberIdentifier::new(42178, 50325),
    ),
    (
        "hubUberStateGroup.gromIntroSequencePlayed",
        UberIdentifier::new(42178, 50418),
    ),
    (
        "hubUberStateGroup.woodCrateB",
        UberIdentifier::new(42178, 51080),
    ),
    (
        "hubUberStateGroup.hutAExpOrb",
        UberIdentifier::new(42178, 51468),
    ),
    (
        "hubUberStateGroup.hutEExpOrb",
        UberIdentifier::new(42178, 51934),
    ),
    (
        "hubUberStateGroup.hutDExpOrb",
        UberIdentifier::new(42178, 52497),
    ),
    (
        "hubUberStateGroup.energyCellA",
        UberIdentifier::new(42178, 52786),
    ),
    (
        "hubUberStateGroup.woodCrateD",
        UberIdentifier::new(42178, 56980),
    ),
    (
        "hubUberStateGroup.hutCExpOrb",
        UberIdentifier::new(42178, 57455),
    ),
    (
        "hubUberStateGroup.largeExpA",
        UberIdentifier::new(42178, 57675),
    ),
    (
        "hubUberStateGroup.mediumExpB",
        UberIdentifier::new(42178, 59623),
    ),
    (
        "hubUberStateGroup.woodCrateB",
        UberIdentifier::new(42178, 63260),
    ),
    (
        "hubUberStateGroup.smallExpC",
        UberIdentifier::new(42178, 63404),
    ),
    (
        "hubUberStateGroup.woodCrateF",
        UberIdentifier::new(42178, 63819),
    ),
    (
        "hubUberStateGroup.savePedestal",
        UberIdentifier::new(42178, 42096),
    ),
    (
        "hubUberStateGroup.shardPurchaseCount",
        UberIdentifier::new(42178, 38),
    ),
    (
        "hubUberStateGroup.craftCutsceneState",
        UberIdentifier::new(42178, 2654),
    ),
    (
        "hubUberStateGroup.builderProjectShardShop",
        UberIdentifier::new(42178, 7528),
    ),
    (
        "hubUberStateGroup.builderProjectBeautify",
        UberIdentifier::new(42178, 15068),
    ),
    (
        "hubUberStateGroup.gardenerProjectFlowers",
        UberIdentifier::new(42178, 16254),
    ),
    (
        "hubUberStateGroup.builderProjectOpenCave",
        UberIdentifier::new(42178, 16586),
    ),
    (
        "hubUberStateGroup.builderProjectSpiritWell",
        UberIdentifier::new(42178, 16825),
    ),
    (
        "hubUberStateGroup.builderProjectRemoveThorns",
        UberIdentifier::new(42178, 18751),
    ),
    (
        "hubUberStateGroup.builderProjectHousesB",
        UberIdentifier::new(42178, 23607),
    ),
    (
        "hubUberStateGroup.gardenerProjectGrapplePlants",
        UberIdentifier::new(42178, 33011),
    ),
    (
        "hubUberStateGroup.gardenerProjectSpringPlants",
        UberIdentifier::new(42178, 38393),
    ),
    (
        "hubUberStateGroup.gardenerProjectTree",
        UberIdentifier::new(42178, 40006),
    ),
    (
        "hubUberStateGroup.builderProjectHousesC",
        UberIdentifier::new(42178, 40448),
    ),
    (
        "hubUberStateGroup.gardenerProjectBashPlants",
        UberIdentifier::new(42178, 47651),
    ),
    (
        "hubUberStateGroup.builderProjectHouses",
        UberIdentifier::new(42178, 51230),
    ),
    (
        "hubUberStateGroup.shardShopState",
        UberIdentifier::new(42178, 61304),
    ),
    (
        "hubUberStateGroup.gardenerProjectGrass",
        UberIdentifier::new(42178, 64583),
    ),
    (
        "wellspringGladesGroup.stompableFloorC",
        UberIdentifier::new(44310, 125),
    ),
    (
        "wellspringGladesGroup.smallExpA",
        UberIdentifier::new(44310, 1647),
    ),
    (
        "wellspringGladesGroup.shardSlotUpgrade",
        UberIdentifier::new(44310, 9902),
    ),
    (
        "wellspringGladesGroup.shardTraderState",
        UberIdentifier::new(44310, 15689),
    ),
    (
        "wellspringGladesGroup.lifeVesselB",
        UberIdentifier::new(44310, 17523),
    ),
    (
        "wellspringGladesGroup.lifeVesselA",
        UberIdentifier::new(44310, 29043),
    ),
    (
        "wellspringGladesGroup.lifeVesselA",
        UberIdentifier::new(44310, 36911),
    ),
    (
        "wellspringGladesGroup.blowableFlame",
        UberIdentifier::new(44310, 47361),
    ),
    (
        "wellspringGladesGroup.largeExpA",
        UberIdentifier::new(44310, 47723),
    ),
    (
        "wellspringGladesGroup.mediumExpA",
        UberIdentifier::new(44310, 47923),
    ),
    (
        "wellspringGladesGroup.stompableFloorA",
        UberIdentifier::new(44310, 55192),
    ),
    (
        "wellspringGladesGroup.stompableFloorB",
        UberIdentifier::new(44310, 57009),
    ),
    (
        "wellspringGladesGroup.shrineEnemyRoom",
        UberIdentifier::new(44310, 58796),
    ),
    (
        "raceGroup.firstRaceUnlockedMessagePlayed",
        UberIdentifier::new(44964, 8328),
    ),
    (
        "raceGroup.wellspringRaceIcon",
        UberIdentifier::new(44964, 12682),
    ),
    (
        "raceGroup.baursReachWindTunnelRaceIcon",
        UberIdentifier::new(44964, 33045),
    ),
    (
        "raceGroup.silentWoodlandRaceIcon",
        UberIdentifier::new(44964, 34110),
    ),
    (
        "raceGroup.desertRaceIcon",
        UberIdentifier::new(44964, 38162),
    ),
    (
        "raceGroup.mouldwoodDepthsRaceIcon",
        UberIdentifier::new(44964, 40578),
    ),
    (
        "raceGroup.inkwaterMarshRaceIcon",
        UberIdentifier::new(44964, 50495),
    ),
    (
        "raceGroup.lumaPoolsRaceIcon",
        UberIdentifier::new(44964, 56533),
    ),
    (
        "raceGroup.kwolokDropRaceIcon",
        UberIdentifier::new(44964, 63031),
    ),
    (
        "raceGroup.raceLeaderboardFilterState",
        UberIdentifier::new(44964, 3798),
    ),
    (
        "raceGroup.wellspringRace",
        UberIdentifier::new(44964, 11512),
    ),
    (
        "raceGroup.silentWoodlandRace",
        UberIdentifier::new(44964, 22703),
    ),
    (
        "raceGroup.baursReachWindTunnelRace",
        UberIdentifier::new(44964, 23661),
    ),
    (
        "raceGroup.kwolokDropRace",
        UberIdentifier::new(44964, 25545),
    ),
    (
        "raceGroup.mouldwoodDepthsRace",
        UberIdentifier::new(44964, 28552),
    ),
    (
        "raceGroup.desertRace",
        UberIdentifier::new(44964, 30767),
    ),
    (
        "raceGroup.inkwaterMarshRace",
        UberIdentifier::new(44964, 45951),
    ),
    (
        "raceGroup.testRace",
        UberIdentifier::new(44964, 50634),
    ),
    (
        "raceGroup.lumaPoolsRace",
        UberIdentifier::new(44964, 54686),
    ),
    (
        "kwoloksCavernThroneRoomGroup.mediumExpA",
        UberIdentifier::new(46462, 3872),
    ),
    (
        "kwoloksCavernThroneRoomGroup.spiritShardA",
        UberIdentifier::new(46462, 9440),
    ),
    (
        "kwoloksCavernThroneRoomGroup.interactedWithMourningMoki",
        UberIdentifier::new(46462, 20733),
    ),
    (
        "kwoloksCavernThroneRoomGroup.smallExpA",
        UberIdentifier::new(46462, 20780),
    ),
    (
        "kwoloksCavernThroneRoomGroup.bombableDoor",
        UberIdentifier::new(46462, 26623),
    ),
    (
        "kwoloksCavernThroneRoomGroup.largeExpA",
        UberIdentifier::new(46462, 29054),
    ),
    (
        "kwoloksCavernThroneRoomGroup.leafPileA",
        UberIdentifier::new(46462, 31447),
    ),
    (
        "kwoloksCavernThroneRoomGroup.questRewardOrb",
        UberIdentifier::new(46462, 31575),
    ),
    (
        "kwoloksCavernThroneRoomGroup.bombableWallA",
        UberIdentifier::new(46462, 34885),
    ),
    (
        "kwoloksCavernThroneRoomGroup.gorlekOreA",
        UberIdentifier::new(46462, 37897),
    ),
    (
        "kwoloksCavernThroneRoomGroup.leafPileC",
        UberIdentifier::new(46462, 56958),
    ),
    (
        "kwoloksCavernThroneRoomGroup.wispRewardPickup",
        UberIdentifier::new(46462, 59806),
    ),
    (
        "npcsStateGroup.windtornRuinsWispTeaser",
        UberIdentifier::new(48248, 1350),
    ),
    (
        "npcsStateGroup.hasMapLumaPools",
        UberIdentifier::new(48248, 1557),
    ),
    (
        "npcsStateGroup.hasMapWellspring",
        UberIdentifier::new(48248, 1590),
    ),
    (
        "npcsStateGroup.lupoEncounteredWeepingRidge",
        UberIdentifier::new(48248, 2253),
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspringGlades",
        UberIdentifier::new(48248, 2285),
    ),
    (
        "npcsStateGroup.treekeeperBRetalk",
        UberIdentifier::new(48248, 3492),
    ),
    (
        "npcsStateGroup.hasMapKwoloksHollow",
        UberIdentifier::new(48248, 3638),
    ),
    (
        "npcsStateGroup.stenchTease",
        UberIdentifier::new(48248, 3846),
    ),
    (
        "npcsStateGroup.hasMapWillowsEnd",
        UberIdentifier::new(48248, 4045),
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspringAfterQuest",
        UberIdentifier::new(48248, 4306),
    ),
    (
        "npcsStateGroup.lupoWantsToTalkToYou",
        UberIdentifier::new(48248, 4510),
    ),
    (
        "npcsStateGroup.tuleyMentionedSeed",
        UberIdentifier::new(48248, 5060),
    ),
    (
        "npcsStateGroup.gromMentionedOre",
        UberIdentifier::new(48248, 5186),
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispE",
        UberIdentifier::new(48248, 5269),
    ),
    (
        "npcsStateGroup.metOpherHubAfterWatermill",
        UberIdentifier::new(48248, 5982),
    ),
    (
        "npcsStateGroup.twillenGaveRumor",
        UberIdentifier::new(48248, 6194),
    ),
    (
        "npcsStateGroup.interactedWindsweptWastesCondition",
        UberIdentifier::new(48248, 6730),
    ),
    (
        "npcsStateGroup.lupoEncounteredSilentWoodlands",
        UberIdentifier::new(48248, 6992),
    ),
    (
        "npcsStateGroup.lupoEncounteredMouldwoodDepths",
        UberIdentifier::new(48248, 7056),
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispA",
        UberIdentifier::new(48248, 7160),
    ),
    (
        "npcsStateGroup.gromTalkedAboutBaur",
        UberIdentifier::new(48248, 7321),
    ),
    (
        "npcsStateGroup.gromGaveWarning",
        UberIdentifier::new(48248, 7646),
    ),
    (
        "npcsStateGroup.willowsEndSeirExitCutscene",
        UberIdentifier::new(48248, 8985),
    ),
    (
        "npcsStateGroup.metGrom",
        UberIdentifier::new(48248, 9394),
    ),
    (
        "npcsStateGroup.hasMapGorlekMines",
        UberIdentifier::new(48248, 9750),
    ),
    (
        "npcsStateGroup.mouldwoodDepthWispTeaser",
        UberIdentifier::new(48248, 11223),
    ),
    (
        "npcsStateGroup.lupoEncounteredBaursReach",
        UberIdentifier::new(48248, 12352),
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispD",
        UberIdentifier::new(48248, 13320),
    ),
    (
        "npcsStateGroup.interactedAfterWellOpened",
        UberIdentifier::new(48248, 14878),
    ),
    (
        "npcsStateGroup.hasMapWindtornRuins",
        UberIdentifier::new(48248, 14995),
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispB",
        UberIdentifier::new(48248, 15833),
    ),
    (
        "npcsStateGroup.lupoEncounteredWillowsEnd",
        UberIdentifier::new(48248, 16157),
    ),
    (
        "npcsStateGroup.lumaPoolsWispSpotted",
        UberIdentifier::new(48248, 18425),
    ),
    (
        "npcsStateGroup.hasMapInkwaterMarsh",
        UberIdentifier::new(48248, 18767),
    ),
    (
        "npcsStateGroup.kiiWantsToTalkToYou",
        UberIdentifier::new(48248, 19551),
    ),
    (
        "npcsStateGroup.Has bought everything",
        UberIdentifier::new(48248, 20000),
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspringValley",
        UberIdentifier::new(48248, 21009),
    ),
    (
        "npcsStateGroup.metOpherLibrary",
        UberIdentifier::new(48248, 22890),
    ),
    (
        "npcsStateGroup.motayWantsToTalkToYou",
        UberIdentifier::new(48248, 24328),
    ),
    (
        "npcsStateGroup.tokkWantsToTalkToYou",
        UberIdentifier::new(48248, 25629),
    ),
    (
        "npcsStateGroup.lupoEncounteredKwoloksHollow",
        UberIdentifier::new(48248, 26627),
    ),
    (
        "npcsStateGroup.lupoEncounteredGorlekMines",
        UberIdentifier::new(48248, 27701),
    ),
    (
        "npcsStateGroup.tuleyWantsToTalkToYou",
        UberIdentifier::new(48248, 28327),
    ),
    (
        "npcsStateGroup.windsweptWastesRuinsDoor",
        UberIdentifier::new(48248, 28782),
    ),
    (
        "npcsStateGroup.hasMapBaursReach",
        UberIdentifier::new(48248, 29604),
    ),
    (
        "npcsStateGroup.gromTalkedAboutLagoon",
        UberIdentifier::new(48248, 30073),
    ),
    (
        "npcsStateGroup.interactedKwoloksCavern",
        UberIdentifier::new(48248, 32549),
    ),
    (
        "npcsStateGroup.baurReachWispTease",
        UberIdentifier::new(48248, 32918),
    ),
    (
        "npcsStateGroup.lupoEncounteredWindsweptWastes",
        UberIdentifier::new(48248, 34318),
    ),
    (
        "npcsStateGroup.twillenMournedKu",
        UberIdentifier::new(48248, 34756),
    ),
    (
        "npcsStateGroup.lupoEncounteredWindtornRuins",
        UberIdentifier::new(48248, 35651),
    ),
    (
        "npcsStateGroup.mouldwoodDepthsWisptIntro",
        UberIdentifier::new(48248, 37364),
    ),
    (
        "npcsStateGroup.hasMapWeepingRidge",
        UberIdentifier::new(48248, 37481),
    ),
    (
        "npcsStateGroup.treekeeperARetalk",
        UberIdentifier::new(48248, 37606),
    ),
    (
        "npcsStateGroup.lupoEncounteredUberState",
        UberIdentifier::new(48248, 40170),
    ),
    (
        "npcsStateGroup.baurReachWispIntro",
        UberIdentifier::new(48248, 40451),
    ),
    (
        "npcsStateGroup.lupoEncounteredBaursReachAfterThaw",
        UberIdentifier::new(48248, 41206),
    ),
    (
        "npcsStateGroup.tokkIntroduced",
        UberIdentifier::new(48248, 42584),
    ),
    (
        "npcsStateGroup.gromWantsToTalkToYou",
        UberIdentifier::new(48248, 43860),
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispF",
        UberIdentifier::new(48248, 44446),
    ),
    (
        "npcsStateGroup.hasMapHowlsOrigin",
        UberIdentifier::new(48248, 45538),
    ),
    (
        "npcsStateGroup.willowsEndSeirIntro",
        UberIdentifier::new(48248, 45600),
    ),
    (
        "npcsStateGroup.interactedBeforeMill",
        UberIdentifier::new(48248, 45664),
    ),
    (
        "npcsStateGroup.gromTalkedAboutDesert",
        UberIdentifier::new(48248, 45751),
    ),
    (
        "npcsStateGroup.gromTalkedAboutMouldwoodGate",
        UberIdentifier::new(48248, 46471),
    ),
    (
        "npcsStateGroup.opherMentiodedWatermill",
        UberIdentifier::new(48248, 46745),
    ),
    (
        "npcsStateGroup.gromInteractedOnce",
        UberIdentifier::new(48248, 46863),
    ),
    (
        "npcsStateGroup.hasMapWellspringValley",
        UberIdentifier::new(48248, 47517),
    ),
    (
        "npcsStateGroup.lupoEncounteredHowlsOrigin",
        UberIdentifier::new(48248, 47546),
    ),
    (
        "npcsStateGroup.feedingGroundsWispIntro",
        UberIdentifier::new(48248, 47785),
    ),
    (
        "npcsStateGroup.hasMapMouldwoodDepths",
        UberIdentifier::new(48248, 48423),
    ),
    (
        "npcsStateGroup.lupoEncounteredInkwaterMarsh",
        UberIdentifier::new(48248, 48619),
    ),
    (
        "npcsStateGroup.interactedBeforeKwolok",
        UberIdentifier::new(48248, 50408),
    ),
    (
        "npcsStateGroup.opherWantsToTalkToYou",
        UberIdentifier::new(48248, 51005),
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispC",
        UberIdentifier::new(48248, 52065),
    ),
    (
        "npcsStateGroup.metMotay",
        UberIdentifier::new(48248, 53028),
    ),
    (
        "npcsStateGroup.hasMapWellspringGlades",
        UberIdentifier::new(48248, 54647),
    ),
    (
        "npcsStateGroup.gromTalkedAboutWatermill",
        UberIdentifier::new(48248, 54806),
    ),
    (
        "npcsStateGroup.metOpherHubBeforeWatermill",
        UberIdentifier::new(48248, 55122),
    ),
    (
        "npcsStateGroup.lupoEncounteredLumaPools",
        UberIdentifier::new(48248, 55617),
    ),
    (
        "npcsStateGroup.metOpherHub",
        UberIdentifier::new(48248, 56448),
    ),
    (
        "npcsStateGroup.twilenWantsToTalkToYou",
        UberIdentifier::new(48248, 60805),
    ),
    (
        "npcsStateGroup.hasMapWindsweptWastes",
        UberIdentifier::new(48248, 61146),
    ),
    (
        "npcsStateGroup.hasMapSilentWoodlands",
        UberIdentifier::new(48248, 61819),
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspring",
        UberIdentifier::new(48248, 61868),
    ),
    (
        "npcsStateGroup.tokkLagoonDialogState",
        UberIdentifier::new(48248, 2131),
    ),
    (
        "npcsStateGroup.talkedInHub",
        UberIdentifier::new(48248, 10337),
    ),
    (
        "npcsStateGroup.twillenHubDialogState",
        UberIdentifier::new(48248, 12799),
    ),
    (
        "npcsStateGroup.inkwaterWellQuest",
        UberIdentifier::new(48248, 18458),
    ),
    (
        "npcsStateGroup.HCMapIconCost",
        UberIdentifier::new(48248, 19397),
    ),
    (
        "npcsStateGroup.twillenKwolokDialogState",
        UberIdentifier::new(48248, 25267),
    ),
    (
        "npcsStateGroup.watermillCEntranceInteraction",
        UberIdentifier::new(48248, 26696),
    ),
    (
        "npcsStateGroup.childMokiDialogState",
        UberIdentifier::new(48248, 28897),
    ),
    (
        "npcsStateGroup.wandererNeedleQuest",
        UberIdentifier::new(48248, 32160),
    ),
    (
        "npcsStateGroup.tokkKwolokDialogState",
        UberIdentifier::new(48248, 33981),
    ),
    (
        "npcsStateGroup.ShardMapIconCost",
        UberIdentifier::new(48248, 41667),
    ),
    (
        "npcsStateGroup.frozenMokiDialogState",
        UberIdentifier::new(48248, 42865),
    ),
    (
        "npcsStateGroup.marshKeystoneQuest",
        UberIdentifier::new(48248, 51645),
    ),
    (
        "npcsStateGroup.iceFisherDialogState",
        UberIdentifier::new(48248, 54962),
    ),
    (
        "npcsStateGroup.mouldwoodMokiDialogState",
        UberIdentifier::new(48248, 57674),
    ),
    (
        "npcsStateGroup.ECMapIconCost",
        UberIdentifier::new(48248, 57988),
    ),
    (
        "npcsStateGroup.lupoIntroState",
        UberIdentifier::new(48248, 62835),
    ),
    (
        "npcsStateGroup.tokkState",
        UberIdentifier::new(48248, 15642),
    ),
    (
        "npcsStateGroup.fastTravelEnabledUberState",
        UberIdentifier::new(48248, 16489),
    ),
    (
        "npcsStateGroup.mapmakerShowMapIconEnergyUberState",
        UberIdentifier::new(48248, 19396),
    ),
    (
        "npcsStateGroup.ShowMapIconCreepheartUberState",
        UberIdentifier::new(48248, 38077),
    ),
    (
        "npcsStateGroup.mapmakerShowMapIconShardUberState",
        UberIdentifier::new(48248, 41666),
    ),
    (
        "npcsStateGroup.mapmakerShowMapIconHealthUberState",
        UberIdentifier::new(48248, 57987),
    ),
    (
        "wellspringGroupDescriptor.energyVesselA",
        UberIdentifier::new(53632, 1911),
    ),
    (
        "wellspringGroupDescriptor.lanternAndCreepA",
        UberIdentifier::new(53632, 2522),
    ),
    (
        "wellspringGroupDescriptor.pushBlockPuzzleA",
        UberIdentifier::new(53632, 3195),
    ),
    (
        "wellspringGroupDescriptor.secretWallB",
        UberIdentifier::new(53632, 3382),
    ),
    (
        "wellspringGroupDescriptor.leafPileA",
        UberIdentifier::new(53632, 3622),
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderG",
        UberIdentifier::new(53632, 6500),
    ),
    (
        "wellspringGroupDescriptor.energyVesselB",
        UberIdentifier::new(53632, 6869),
    ),
    (
        "wellspringGroupDescriptor.secretWallC",
        UberIdentifier::new(53632, 9366),
    ),
    (
        "wellspringGroupDescriptor.mediumExpA",
        UberIdentifier::new(53632, 12019),
    ),
    (
        "wellspringGroupDescriptor.lifeVesselA",
        UberIdentifier::new(53632, 17403),
    ),
    (
        "wellspringGroupDescriptor.orePickupA",
        UberIdentifier::new(53632, 21124),
    ),
    (
        "wellspringGroupDescriptor.smallExpA",
        UberIdentifier::new(53632, 21790),
    ),
    (
        "wellspringGroupDescriptor.wispSequencePlayed",
        UberIdentifier::new(53632, 22486),
    ),
    (
        "wellspringGroupDescriptor.orePickupB",
        UberIdentifier::new(53632, 25556),
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier::new(53632, 25817),
    ),
    (
        "wellspringGroupDescriptor.leafPileA",
        UberIdentifier::new(53632, 32197),
    ),
    (
        "wellspringGroupDescriptor.expOrbG",
        UberIdentifier::new(53632, 32785),
    ),
    (
        "wellspringGroupDescriptor.spiritShard",
        UberIdentifier::new(53632, 33168),
    ),
    (
        "wellspringGroupDescriptor.secretWallB",
        UberIdentifier::new(53632, 40587),
    ),
    (
        "wellspringGroupDescriptor.questItemCompass",
        UberIdentifier::new(53632, 41227),
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderF",
        UberIdentifier::new(53632, 42264),
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderE",
        UberIdentifier::new(53632, 51706),
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier::new(53632, 51735),
    ),
    (
        "wellspringGroupDescriptor.xpOrbUberState",
        UberIdentifier::new(53632, 54915),
    ),
    (
        "wellspringGroupDescriptor.mediumExpA",
        UberIdentifier::new(53632, 56829),
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier::new(53632, 58126),
    ),
    (
        "wellspringGroupDescriptor.rotatingWheel",
        UberIdentifier::new(53632, 61074),
    ),
    (
        "wellspringGroupDescriptor.spiritShardA",
        UberIdentifier::new(53632, 61128),
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderC",
        UberIdentifier::new(53632, 62356),
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier::new(53632, 62781),
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier::new(53632, 64763),
    ),
    (
        "wellspringGroupDescriptor.savePedestal",
        UberIdentifier::new(53632, 14947),
    ),
    (
        "wellspringGroupDescriptor.savePedestalUberState",
        UberIdentifier::new(53632, 18181),
    ),
    (
        "wellspringGroupDescriptor.savePedestal",
        UberIdentifier::new(53632, 53974),
    ),
    (
        "wellspringGroupDescriptor.savePedestalUberState",
        UberIdentifier::new(53632, 63074),
    ),
    (
        "wellspringGroupDescriptor.showDoorCutsceneState",
        UberIdentifier::new(53632, 26178),
    ),
    (
        "prologueGroup.areaText",
        UberIdentifier::new(54846, 27125),
    ),
    (
        "_petrifiedForestGroup.xpOrbA",
        UberIdentifier::new(58674, 193),
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier::new(58674, 595),
    ),
    (
        "_petrifiedForestGroup.keyStoneD",
        UberIdentifier::new(58674, 780),
    ),
    (
        "_petrifiedForestGroup.creepBlocker",
        UberIdentifier::new(58674, 902),
    ),
    (
        "_petrifiedForestGroup.keystoneDUberState",
        UberIdentifier::new(58674, 1816),
    ),
    (
        "_petrifiedForestGroup.keystoneBUberState",
        UberIdentifier::new(58674, 2169),
    ),
    (
        "_petrifiedForestGroup.keyStoneA",
        UberIdentifier::new(58674, 2227),
    ),
    (
        "_petrifiedForestGroup.areaText",
        UberIdentifier::new(58674, 2317),
    ),
    (
        "_petrifiedForestGroup.stompableFloorA",
        UberIdentifier::new(58674, 2797),
    ),
    (
        "_petrifiedForestGroup.stompableFloorB",
        UberIdentifier::new(58674, 3577),
    ),
    (
        "_petrifiedForestGroup.blowableFlameA",
        UberIdentifier::new(58674, 5285),
    ),
    (
        "_petrifiedForestGroup.xpOrbUberState",
        UberIdentifier::new(58674, 6936),
    ),
    (
        "_petrifiedForestGroup.petrifiedOwlStalkSequenceCompleted",
        UberIdentifier::new(58674, 7636),
    ),
    (
        "_petrifiedForestGroup.CollectibleXpA",
        UberIdentifier::new(58674, 8487),
    ),
    (
        "_petrifiedForestGroup.drillableWallA",
        UberIdentifier::new(58674, 8810),
    ),
    (
        "_petrifiedForestGroup.leafPileB",
        UberIdentifier::new(58674, 9239),
    ),
    (
        "_petrifiedForestGroup.energyContainerA",
        UberIdentifier::new(58674, 9583),
    ),
    (
        "_petrifiedForestGroup.expOrbC",
        UberIdentifier::new(58674, 9881),
    ),
    (
        "_petrifiedForestGroup.narratorLineShownHowl",
        UberIdentifier::new(58674, 10677),
    ),
    (
        "_petrifiedForestGroup.blowableFlameA",
        UberIdentifier::new(58674, 10685),
    ),
    (
        "_petrifiedForestGroup.leafPile",
        UberIdentifier::new(58674, 10877),
    ),
    (
        "_petrifiedForestGroup.stompableFloor",
        UberIdentifier::new(58674, 11400),
    ),
    (
        "_petrifiedForestGroup.keyStoneC",
        UberIdentifier::new(58674, 11736),
    ),
    (
        "_petrifiedForestGroup.hutDoorUnlocked",
        UberIdentifier::new(58674, 14313),
    ),
    (
        "_petrifiedForestGroup.powlVignettePlayed",
        UberIdentifier::new(58674, 14539),
    ),
    (
        "_petrifiedForestGroup.mediumPickupB",
        UberIdentifier::new(58674, 14590),
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier::new(58674, 14593),
    ),
    (
        "_petrifiedForestGroup.displayedGlideHint",
        UberIdentifier::new(58674, 14912),
    ),
    (
        "_petrifiedForestGroup.playedEpilogue",
        UberIdentifier::new(58674, 15269),
    ),
    (
        "_petrifiedForestGroup.keyStoneB",
        UberIdentifier::new(58674, 17420),
    ),
    (
        "_petrifiedForestGroup.blowableFlame",
        UberIdentifier::new(58674, 17742),
    ),
    (
        "_petrifiedForestGroup.smallPickupA",
        UberIdentifier::new(58674, 17974),
    ),
    (
        "_petrifiedForestGroup.lifeCellUberState",
        UberIdentifier::new(58674, 18735),
    ),
    (
        "_petrifiedForestGroup.mediumPickupA",
        UberIdentifier::new(58674, 18924),
    ),
    (
        "_petrifiedForestGroup.keyStoneB",
        UberIdentifier::new(58674, 19769),
    ),
    (
        "_petrifiedForestGroup.leafPile",
        UberIdentifier::new(58674, 20143),
    ),
    (
        "_petrifiedForestGroup.gorlekOreA",
        UberIdentifier::new(58674, 20713),
    ),
    (
        "_petrifiedForestGroup.boolean_gasBallBridge",
        UberIdentifier::new(58674, 20724),
    ),
    (
        "_petrifiedForestGroup.keystoneCUberState",
        UberIdentifier::new(58674, 20944),
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier::new(58674, 20983),
    ),
    (
        "_petrifiedForestGroup.leverGateA",
        UberIdentifier::new(58674, 21139),
    ),
    (
        "_petrifiedForestGroup.narratorLineShriekAttackShown",
        UberIdentifier::new(58674, 21385),
    ),
    (
        "_petrifiedForestGroup.doorState",
        UberIdentifier::new(58674, 21500),
    ),
    (
        "_petrifiedForestGroup.narratorLineShown",
        UberIdentifier::new(58674, 22056),
    ),
    (
        "_petrifiedForestGroup.xpOrbA",
        UberIdentifier::new(58674, 22472),
    ),
    (
        "_petrifiedForestGroup.xpOrbUberState",
        UberIdentifier::new(58674, 22503),
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier::new(58674, 23186),
    ),
    (
        "_petrifiedForestGroup.lagoonBreakableFloor",
        UberIdentifier::new(58674, 24457),
    ),
    (
        "_petrifiedForestGroup.xpOrbA",
        UberIdentifier::new(58674, 24911),
    ),
    (
        "_petrifiedForestGroup.gorlekOreA",
        UberIdentifier::new(58674, 26274),
    ),
    (
        "_petrifiedForestGroup.shardSlotA",
        UberIdentifier::new(58674, 26282),
    ),
    (
        "_petrifiedForestGroup.smallExpOrbA",
        UberIdentifier::new(58674, 26639),
    ),
    (
        "_petrifiedForestGroup.gorlekOreA",
        UberIdentifier::new(58674, 28710),
    ),
    (
        "_petrifiedForestGroup.narrationPetrifiedOwlStalk",
        UberIdentifier::new(58674, 29035),
    ),
    (
        "_petrifiedForestGroup.shardSlotUpgradePlaceholder",
        UberIdentifier::new(58674, 29265),
    ),
    (
        "_petrifiedForestGroup.stompableFloor",
        UberIdentifier::new(58674, 29622),
    ),
    (
        "_petrifiedForestGroup.shardPickupA",
        UberIdentifier::new(58674, 30377),
    ),
    (
        "_petrifiedForestGroup.areaText",
        UberIdentifier::new(58674, 30897),
    ),
    (
        "_petrifiedForestGroup.expOrbC",
        UberIdentifier::new(58674, 30908),
    ),
    (
        "_petrifiedForestGroup.powlVignettePlayed",
        UberIdentifier::new(58674, 32369),
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier::new(58674, 32647),
    ),
    (
        "_petrifiedForestGroup.expOrbD",
        UberIdentifier::new(58674, 33893),
    ),
    (
        "_petrifiedForestGroup.breakableWallA",
        UberIdentifier::new(58674, 33965),
    ),
    (
        "_petrifiedForestGroup.diggableWallA",
        UberIdentifier::new(58674, 34799),
    ),
    (
        "_petrifiedForestGroup.expOrbB",
        UberIdentifier::new(58674, 36199),
    ),
    (
        "_petrifiedForestGroup.floatZoneState",
        UberIdentifier::new(58674, 36832),
    ),
    (
        "_petrifiedForestGroup.featherVignettePlayed",
        UberIdentifier::new(58674, 36965),
    ),
    (
        "_petrifiedForestGroup.expOrbB",
        UberIdentifier::new(58674, 37006),
    ),
    (
        "_petrifiedForestGroup.leafPile",
        UberIdentifier::new(58674, 37037),
    ),
    (
        "_petrifiedForestGroup.lifeCellUberState",
        UberIdentifier::new(58674, 37128),
    ),
    (
        "_petrifiedForestGroup.stompableFloorC",
        UberIdentifier::new(58674, 37636),
    ),
    (
        "_petrifiedForestGroup.mokiCleanWaterVignetteTriggered",
        UberIdentifier::new(58674, 37811),
    ),
    (
        "_petrifiedForestGroup.expOrb",
        UberIdentifier::new(58674, 37885),
    ),
    (
        "_petrifiedForestGroup.shardA",
        UberIdentifier::new(58674, 38285),
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier::new(58674, 39950),
    ),
    (
        "_petrifiedForestGroup.boolean_skeletonBoneA",
        UberIdentifier::new(58674, 40066),
    ),
    (
        "_petrifiedForestGroup.keyStoneD",
        UberIdentifier::new(58674, 40073),
    ),
    (
        "_petrifiedForestGroup.mokiFoulWaterVignetteTriggered",
        UberIdentifier::new(58674, 41644),
    ),
    (
        "_petrifiedForestGroup.expOrbC",
        UberIdentifier::new(58674, 42158),
    ),
    (
        "_petrifiedForestGroup.keyStoneA",
        UberIdentifier::new(58674, 42531),
    ),
    (
        "_petrifiedForestGroup.keyStoneC",
        UberIdentifier::new(58674, 43033),
    ),
    (
        "_petrifiedForestGroup.keystoneAUberState",
        UberIdentifier::new(58674, 44215),
    ),
    (
        "_petrifiedForestGroup.creepA",
        UberIdentifier::new(58674, 44324),
    ),
    (
        "_petrifiedForestGroup.escapeRocks",
        UberIdentifier::new(58674, 44864),
    ),
    (
        "_petrifiedForestGroup.collapsingSkeletonA",
        UberIdentifier::new(58674, 46547),
    ),
    (
        "_petrifiedForestGroup.petrifiedForestNewTransitionOriVignettePlayed",
        UberIdentifier::new(58674, 46980),
    ),
    (
        "_petrifiedForestGroup.setupDownB",
        UberIdentifier::new(58674, 47179),
    ),
    (
        "_petrifiedForestGroup.breakableGroundA",
        UberIdentifier::new(58674, 47751),
    ),
    (
        "_petrifiedForestGroup.creebBulb",
        UberIdentifier::new(58674, 48394),
    ),
    (
        "_petrifiedForestGroup.creepBall",
        UberIdentifier::new(58674, 49272),
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier::new(58674, 49535),
    ),
    (
        "_petrifiedForestGroup.boolean_skeletonBoneC",
        UberIdentifier::new(58674, 50410),
    ),
    (
        "_petrifiedForestGroup.boolean_skeletonBoneB",
        UberIdentifier::new(58674, 51501),
    ),
    (
        "_petrifiedForestGroup.shownHint",
        UberIdentifier::new(58674, 51890),
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier::new(58674, 52280),
    ),
    (
        "_petrifiedForestGroup.patrifiedForestBreakableFloor",
        UberIdentifier::new(58674, 52345),
    ),
    (
        "_petrifiedForestGroup.mediumPickupC",
        UberIdentifier::new(58674, 54516),
    ),
    (
        "_petrifiedForestGroup.stompableFloor",
        UberIdentifier::new(58674, 54560),
    ),
    (
        "_petrifiedForestGroup.breakableWall",
        UberIdentifier::new(58674, 54686),
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier::new(58674, 55650),
    ),
    (
        "_petrifiedForestGroup.enemyRoom",
        UberIdentifier::new(58674, 56043),
    ),
    (
        "_petrifiedForestGroup.wispCutscenePlayed",
        UberIdentifier::new(58674, 58268),
    ),
    (
        "_petrifiedForestGroup.shownFlapEnemyHint",
        UberIdentifier::new(58674, 58684),
    ),
    (
        "_petrifiedForestGroup.xpOrbB",
        UberIdentifier::new(58674, 59372),
    ),
    (
        "_petrifiedForestGroup.CollectibleXPB",
        UberIdentifier::new(58674, 59691),
    ),
    (
        "_petrifiedForestGroup.expOrbD",
        UberIdentifier::new(58674, 59714),
    ),
    (
        "_petrifiedForestGroup.skeletonState",
        UberIdentifier::new(58674, 61252),
    ),
    (
        "_petrifiedForestGroup.setupDownA",
        UberIdentifier::new(58674, 61327),
    ),
    (
        "_petrifiedForestGroup.stomableFloorB",
        UberIdentifier::new(58674, 61391),
    ),
    (
        "_petrifiedForestGroup.breakableHiddenWall",
        UberIdentifier::new(58674, 61577),
    ),
    (
        "_petrifiedForestGroup.clothBroken",
        UberIdentifier::new(58674, 63837),
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier::new(58674, 64057),
    ),
    (
        "_petrifiedForestGroup.expOrb",
        UberIdentifier::new(58674, 64484),
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier::new(58674, 64690),
    ),
    (
        "_petrifiedForestGroup.stomableFloorA",
        UberIdentifier::new(58674, 65519),
    ),
    (
        "_petrifiedForestGroup.savePedestalA",
        UberIdentifier::new(58674, 1965),
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier::new(58674, 7071),
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier::new(58674, 10029),
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier::new(58674, 10997),
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier::new(58674, 11221),
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier::new(58674, 36061),
    ),
    (
        "_petrifiedForestGroup.chaseState",
        UberIdentifier::new(58674, 32810),
    ),
    (
        "_petrifiedForestGroup.petrifiedForestNewTransitionKuVignettePlayed",
        UberIdentifier::new(58674, 44798),
    ),
    (
        "_petrifiedForestGroup.petrifiedOwlClothState",
        UberIdentifier::new(58674, 45819),
    ),
    (
        "_petrifiedForestGroup.petrifiedOwlState",
        UberIdentifier::new(58674, 61616),
    ),
    (
        "shrineGroup.shrineLaser",
        UberIdentifier::new(61306, 2129),
    ),
    (
        "shrineGroup.shrineProjectile",
        UberIdentifier::new(61306, 2239),
    ),
    (
        "shrineGroup.shrineMouldwoodDepths",
        UberIdentifier::new(61306, 18888),
    ),
    (
        "shrineGroup.shrineHammer",
        UberIdentifier::new(61306, 26590),
    ),
    (
        "shrineGroup.shrinePortal",
        UberIdentifier::new(61306, 40441),
    ),
    (
        "shrineGroup.shrineTeleport",
        UberIdentifier::new(61306, 52344),
    ),
    (
        "shrineGroup.shrineOfFall",
        UberIdentifier::new(61306, 56122),
    ),
    (
        "spiderGroupDescriptor.spiderlingsQuestUberState",
        UberIdentifier::new(61314, 55764),
    ),
    (
        "spiderGroupDescriptor.spiderNpcState",
        UberIdentifier::new(61314, 61458),
    ),
    (
        "testUberStateGroupDescriptor.floatUberStateDescriptor",
        UberIdentifier::new(63018, 22925),
    ),
];
