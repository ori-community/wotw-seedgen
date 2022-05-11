use super::UberIdentifier;

pub(super) const UBER_STATES: &[(&str, UberIdentifier)] = &[
    (
        "trees.bash",
        UberIdentifier { uber_group: 0, uber_id: 0 },
    ),
    (
        "trees.double_jump",
        UberIdentifier { uber_group: 0, uber_id: 5 },
    ),
    (
        "trees.launch",
        UberIdentifier { uber_group: 0, uber_id: 8 },
    ),
    (
        "trees.grenade",
        UberIdentifier { uber_group: 0, uber_id: 51 },
    ),
    (
        "trees.grapple",
        UberIdentifier { uber_group: 0, uber_id: 57 },
    ),
    (
        "trees.flash",
        UberIdentifier { uber_group: 0, uber_id: 62 },
    ),
    (
        "trees.regenerate",
        UberIdentifier { uber_group: 0, uber_id: 77 },
    ),
    (
        "trees.bow",
        UberIdentifier { uber_group: 0, uber_id: 97 },
    ),
    (
        "trees.sword",
        UberIdentifier { uber_group: 0, uber_id: 100 },
    ),
    (
        "trees.burrow",
        UberIdentifier { uber_group: 0, uber_id: 101 },
    ),
    (
        "trees.dash",
        UberIdentifier { uber_group: 0, uber_id: 102 },
    ),
    (
        "trees.water_dash",
        UberIdentifier { uber_group: 0, uber_id: 104 },
    ),
    (
        "trees.ancestral_light",
        UberIdentifier { uber_group: 0, uber_id: 120 },
    ),
    (
        "trees.ancestral_light_2",
        UberIdentifier { uber_group: 0, uber_id: 121 },
    ),
    (
        "opher_weapons.Water Breath",
        UberIdentifier { uber_group: 1, uber_id: 23 },
    ),
    (
        "opher_weapons.Spike",
        UberIdentifier { uber_group: 1, uber_id: 74 },
    ),
    (
        "opher_weapons.Spirit Smash",
        UberIdentifier { uber_group: 1, uber_id: 98 },
    ),
    (
        "opher_weapons.Fast Travel",
        UberIdentifier { uber_group: 1, uber_id: 105 },
    ),
    (
        "opher_weapons.Spirit Star",
        UberIdentifier { uber_group: 1, uber_id: 106 },
    ),
    (
        "opher_weapons.Blaze",
        UberIdentifier { uber_group: 1, uber_id: 115 },
    ),
    (
        "opher_weapons.Sentry",
        UberIdentifier { uber_group: 1, uber_id: 116 },
    ),
    (
        "opher_weapons.Exploding Spike",
        UberIdentifier { uber_group: 1, uber_id: 1074 },
    ),
    (
        "opher_weapons.Shock Smash",
        UberIdentifier { uber_group: 1, uber_id: 1098 },
    ),
    (
        "opher_weapons.Static Star",
        UberIdentifier { uber_group: 1, uber_id: 1106 },
    ),
    (
        "opher_weapons.Charge Blaze",
        UberIdentifier { uber_group: 1, uber_id: 1115 },
    ),
    (
        "opher_weapons.Rapid Sentry",
        UberIdentifier { uber_group: 1, uber_id: 1116 },
    ),
    (
        "opher_weapons.Has bought everything",
        UberIdentifier { uber_group: 1, uber_id: 20000 },
    ),
    (
        "opher_weapons.Water Breath cost",
        UberIdentifier { uber_group: 1, uber_id: 10023 },
    ),
    (
        "opher_weapons.Spike cost",
        UberIdentifier { uber_group: 1, uber_id: 10074 },
    ),
    (
        "opher_weapons.Spirit Smash cost",
        UberIdentifier { uber_group: 1, uber_id: 10098 },
    ),
    (
        "opher_weapons.Fast Travel cost",
        UberIdentifier { uber_group: 1, uber_id: 10105 },
    ),
    (
        "opher_weapons.Spirit Star cost",
        UberIdentifier { uber_group: 1, uber_id: 10106 },
    ),
    (
        "opher_weapons.Blaze cost",
        UberIdentifier { uber_group: 1, uber_id: 10115 },
    ),
    (
        "opher_weapons.Sentry cost",
        UberIdentifier { uber_group: 1, uber_id: 10116 },
    ),
    (
        "opher_weapons.Exploding Spike cost",
        UberIdentifier { uber_group: 1, uber_id: 11074 },
    ),
    (
        "opher_weapons.Shock Smash cost",
        UberIdentifier { uber_group: 1, uber_id: 11098 },
    ),
    (
        "opher_weapons.Static Star cost",
        UberIdentifier { uber_group: 1, uber_id: 11106 },
    ),
    (
        "opher_weapons.Charge Blaze cost",
        UberIdentifier { uber_group: 1, uber_id: 11115 },
    ),
    (
        "opher_weapons.Rapid Sentry cost",
        UberIdentifier { uber_group: 1, uber_id: 11116 },
    ),
    (
        "twillen_shards.Overcharge",
        UberIdentifier { uber_group: 2, uber_id: 1 },
    ),
    (
        "twillen_shards.TripleJump",
        UberIdentifier { uber_group: 2, uber_id: 2 },
    ),
    (
        "twillen_shards.Wingclip",
        UberIdentifier { uber_group: 2, uber_id: 3 },
    ),
    (
        "twillen_shards.Swap",
        UberIdentifier { uber_group: 2, uber_id: 5 },
    ),
    (
        "twillen_shards.LightHarvest",
        UberIdentifier { uber_group: 2, uber_id: 19 },
    ),
    (
        "twillen_shards.Vitality",
        UberIdentifier { uber_group: 2, uber_id: 22 },
    ),
    (
        "twillen_shards.Energy",
        UberIdentifier { uber_group: 2, uber_id: 26 },
    ),
    (
        "twillen_shards.Finesse",
        UberIdentifier { uber_group: 2, uber_id: 40 },
    ),
    (
        "twillen_shards.Has bought everything",
        UberIdentifier { uber_group: 2, uber_id: 20000 },
    ),
    (
        "twillen_shards.Overcharge cost",
        UberIdentifier { uber_group: 2, uber_id: 101 },
    ),
    (
        "twillen_shards.TripleJump cost",
        UberIdentifier { uber_group: 2, uber_id: 102 },
    ),
    (
        "twillen_shards.Wingclip cost",
        UberIdentifier { uber_group: 2, uber_id: 103 },
    ),
    (
        "twillen_shards.Swap cost",
        UberIdentifier { uber_group: 2, uber_id: 105 },
    ),
    (
        "twillen_shards.LightHarvest cost",
        UberIdentifier { uber_group: 2, uber_id: 119 },
    ),
    (
        "twillen_shards.Vitality cost",
        UberIdentifier { uber_group: 2, uber_id: 122 },
    ),
    (
        "twillen_shards.Energy cost",
        UberIdentifier { uber_group: 2, uber_id: 126 },
    ),
    (
        "twillen_shards.Finesse cost",
        UberIdentifier { uber_group: 2, uber_id: 140 },
    ),
    (
        "game_state.Spawn",
        UberIdentifier { uber_group: 3, uber_id: 0 },
    ),
    (
        "game_state.Goal Modes Complete",
        UberIdentifier { uber_group: 3, uber_id: 11 },
    ),
    (
        "game_state.On Teleport",
        UberIdentifier { uber_group: 3, uber_id: 20 },
    ),
    (
        "game_state.Reload",
        UberIdentifier { uber_group: 3, uber_id: 1 },
    ),
    (
        "game_state.Binding One",
        UberIdentifier { uber_group: 3, uber_id: 2 },
    ),
    (
        "game_state.Binding Two",
        UberIdentifier { uber_group: 3, uber_id: 3 },
    ),
    (
        "game_state.Binding Three",
        UberIdentifier { uber_group: 3, uber_id: 4 },
    ),
    (
        "game_state.Binding Four",
        UberIdentifier { uber_group: 3, uber_id: 5 },
    ),
    (
        "game_state.Binding Five",
        UberIdentifier { uber_group: 3, uber_id: 6 },
    ),
    (
        "game_state.Load",
        UberIdentifier { uber_group: 3, uber_id: 7 },
    ),
    (
        "rando_upgrades.Autoaim",
        UberIdentifier { uber_group: 4, uber_id: 37 },
    ),
    (
        "rando_upgrades.Grenades explode on collision",
        UberIdentifier { uber_group: 4, uber_id: 41 },
    ),
    (
        "rando_upgrades.Bashable uncharged Grenades",
        UberIdentifier { uber_group: 4, uber_id: 42 },
    ),
    (
        "rando_upgrades.Charged Air Grenades",
        UberIdentifier { uber_group: 4, uber_id: 43 },
    ),
    (
        "rando_upgrades.Bow as fire source",
        UberIdentifier { uber_group: 4, uber_id: 70 },
    ),
    (
        "rando_upgrades.Blaze as fire source",
        UberIdentifier { uber_group: 4, uber_id: 71 },
    ),
    (
        "rando_upgrades.Sword as fire source",
        UberIdentifier { uber_group: 4, uber_id: 72 },
    ),
    (
        "rando_upgrades.Hammer as fire source",
        UberIdentifier { uber_group: 4, uber_id: 73 },
    ),
    (
        "rando_upgrades.Spear as fire source",
        UberIdentifier { uber_group: 4, uber_id: 74 },
    ),
    (
        "rando_upgrades.Shuriken as fire source",
        UberIdentifier { uber_group: 4, uber_id: 75 },
    ),
    (
        "rando_upgrades.Hammer speed multiplier",
        UberIdentifier { uber_group: 4, uber_id: 0 },
    ),
    (
        "rando_upgrades.Sword speed multiplier",
        UberIdentifier { uber_group: 4, uber_id: 1 },
    ),
    (
        "rando_upgrades.Blaze cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 2 },
    ),
    (
        "rando_upgrades.Spike cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 3 },
    ),
    (
        "rando_upgrades.Shuriken cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 4 },
    ),
    (
        "rando_upgrades.Sentry cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 5 },
    ),
    (
        "rando_upgrades.Bow cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 6 },
    ),
    (
        "rando_upgrades.Regeneration cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 7 },
    ),
    (
        "rando_upgrades.Flash cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 8 },
    ),
    (
        "rando_upgrades.Light Burst cost multiplier",
        UberIdentifier { uber_group: 4, uber_id: 9 },
    ),
    (
        "rando_upgrades.Bow rapid fire multiplier",
        UberIdentifier { uber_group: 4, uber_id: 10 },
    ),
    (
        "rando_upgrades.Spear speed multiplier",
        UberIdentifier { uber_group: 4, uber_id: 11 },
    ),
    (
        "rando_upgrades.Grenade charge time modifier",
        UberIdentifier { uber_group: 4, uber_id: 44 },
    ),
    (
        "rando_upgrades.Launch Speed",
        UberIdentifier { uber_group: 4, uber_id: 80 },
    ),
    (
        "rando_upgrades.Dash Distance",
        UberIdentifier { uber_group: 4, uber_id: 81 },
    ),
    (
        "rando_upgrades.Bash Speed",
        UberIdentifier { uber_group: 4, uber_id: 82 },
    ),
    (
        "rando_upgrades.Burrow Speed",
        UberIdentifier { uber_group: 4, uber_id: 83 },
    ),
    (
        "rando_upgrades.Burrow Dash Speed",
        UberIdentifier { uber_group: 4, uber_id: 84 },
    ),
    (
        "rando_upgrades.Swim Speed",
        UberIdentifier { uber_group: 4, uber_id: 85 },
    ),
    (
        "rando_upgrades.Swim Dash Speed",
        UberIdentifier { uber_group: 4, uber_id: 86 },
    ),
    (
        "rando_upgrades.Jump Height",
        UberIdentifier { uber_group: 4, uber_id: 87 },
    ),
    (
        "rando_upgrades.Relic",
        UberIdentifier { uber_group: 4, uber_id: 20 },
    ),
    (
        "rando_upgrades.Health Regeneration",
        UberIdentifier { uber_group: 4, uber_id: 30 },
    ),
    (
        "rando_upgrades.Energy Regeneration",
        UberIdentifier { uber_group: 4, uber_id: 31 },
    ),
    (
        "rando_upgrades.Extra Double Jumps",
        UberIdentifier { uber_group: 4, uber_id: 35 },
    ),
    (
        "rando_upgrades.Extra Dashes",
        UberIdentifier { uber_group: 4, uber_id: 36 },
    ),
    (
        "rando_upgrades.Extra Grenades",
        UberIdentifier { uber_group: 4, uber_id: 40 },
    ),
    (
        "rando_upgrades.Grenade multishot",
        UberIdentifier { uber_group: 4, uber_id: 45 },
    ),
    (
        "rando_upgrades.Hammer Speed",
        UberIdentifier { uber_group: 4, uber_id: 50 },
    ),
    (
        "rando_upgrades.Sword Speed",
        UberIdentifier { uber_group: 4, uber_id: 51 },
    ),
    (
        "rando_upgrades.Blaze Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 52 },
    ),
    (
        "rando_upgrades.Spike Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 53 },
    ),
    (
        "rando_upgrades.Shuriken Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 54 },
    ),
    (
        "rando_upgrades.Sentry Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 55 },
    ),
    (
        "rando_upgrades.Bow Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 56 },
    ),
    (
        "rando_upgrades.Regenerate Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 57 },
    ),
    (
        "rando_upgrades.Flash Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 58 },
    ),
    (
        "rando_upgrades.Light Burst Efficiency",
        UberIdentifier { uber_group: 4, uber_id: 59 },
    ),
    (
        "rando_upgrades.Exploding Spike",
        UberIdentifier { uber_group: 4, uber_id: 95 },
    ),
    (
        "rando_upgrades.Shock Smash",
        UberIdentifier { uber_group: 4, uber_id: 96 },
    ),
    (
        "rando_upgrades.Static Star",
        UberIdentifier { uber_group: 4, uber_id: 97 },
    ),
    (
        "rando_upgrades.Charge Blaze",
        UberIdentifier { uber_group: 4, uber_id: 98 },
    ),
    (
        "rando_upgrades.Rapid Sentry",
        UberIdentifier { uber_group: 4, uber_id: 99 },
    ),
    (
        "rando_upgrades.Marsh Relic",
        UberIdentifier { uber_group: 4, uber_id: 100 },
    ),
    (
        "rando_upgrades.Hollow Relic",
        UberIdentifier { uber_group: 4, uber_id: 101 },
    ),
    (
        "rando_upgrades.Glades Relic",
        UberIdentifier { uber_group: 4, uber_id: 102 },
    ),
    (
        "rando_upgrades.Wellspring Relic",
        UberIdentifier { uber_group: 4, uber_id: 103 },
    ),
    (
        "rando_upgrades.Burrows Relic",
        UberIdentifier { uber_group: 4, uber_id: 104 },
    ),
    (
        "rando_upgrades.Woods Relic",
        UberIdentifier { uber_group: 4, uber_id: 105 },
    ),
    (
        "rando_upgrades.Reach Relic",
        UberIdentifier { uber_group: 4, uber_id: 106 },
    ),
    (
        "rando_upgrades.Pools Relic",
        UberIdentifier { uber_group: 4, uber_id: 107 },
    ),
    (
        "rando_upgrades.Depths Relic",
        UberIdentifier { uber_group: 4, uber_id: 108 },
    ),
    (
        "rando_upgrades.Wastes Relic",
        UberIdentifier { uber_group: 4, uber_id: 109 },
    ),
    (
        "rando_upgrades.Willow Relic",
        UberIdentifier { uber_group: 4, uber_id: 111 },
    ),
    (
        "rando_state.Checkable Item Hint 1",
        UberIdentifier { uber_group: 6, uber_id: 10 },
    ),
    (
        "rando_state.Checkable Item Hint 2",
        UberIdentifier { uber_group: 6, uber_id: 11 },
    ),
    (
        "rando_state.Checkable Item Hint 3",
        UberIdentifier { uber_group: 6, uber_id: 12 },
    ),
    (
        "rando_state.Checkable Item Hint 4",
        UberIdentifier { uber_group: 6, uber_id: 13 },
    ),
    (
        "rando_state.Checkable Item Hint 5",
        UberIdentifier { uber_group: 6, uber_id: 14 },
    ),
    (
        "rando_state.Checkable Item Hint 6",
        UberIdentifier { uber_group: 6, uber_id: 15 },
    ),
    (
        "rando_state.Checkable Item Hint 7",
        UberIdentifier { uber_group: 6, uber_id: 16 },
    ),
    (
        "rando_state.Checkable Item Hint 8",
        UberIdentifier { uber_group: 6, uber_id: 17 },
    ),
    (
        "rando_state.Checkable Item Hint 9",
        UberIdentifier { uber_group: 6, uber_id: 18 },
    ),
    (
        "rando_state.Checkable Item Hint 10",
        UberIdentifier { uber_group: 6, uber_id: 19 },
    ),
    (
        "rando_state.HollowTP",
        UberIdentifier { uber_group: 6, uber_id: 106 },
    ),
    (
        "rando_state.Bash",
        UberIdentifier { uber_group: 6, uber_id: 1000 },
    ),
    (
        "rando_state.WallJump",
        UberIdentifier { uber_group: 6, uber_id: 1003 },
    ),
    (
        "rando_state.DoubleJump",
        UberIdentifier { uber_group: 6, uber_id: 1005 },
    ),
    (
        "rando_state.Launch",
        UberIdentifier { uber_group: 6, uber_id: 1008 },
    ),
    (
        "rando_state.Feather",
        UberIdentifier { uber_group: 6, uber_id: 1014 },
    ),
    (
        "rando_state.Spirit Flame",
        UberIdentifier { uber_group: 6, uber_id: 1015 },
    ),
    (
        "rando_state.WaterBreath",
        UberIdentifier { uber_group: 6, uber_id: 1023 },
    ),
    (
        "rando_state.Resilience",
        UberIdentifier { uber_group: 6, uber_id: 1031 },
    ),
    (
        "rando_state.Health Efficiency",
        UberIdentifier { uber_group: 6, uber_id: 1032 },
    ),
    (
        "rando_state.Energy Efficiency",
        UberIdentifier { uber_group: 6, uber_id: 1039 },
    ),
    (
        "rando_state.LightBurst",
        UberIdentifier { uber_group: 6, uber_id: 1051 },
    ),
    (
        "rando_state.Grapple",
        UberIdentifier { uber_group: 6, uber_id: 1057 },
    ),
    (
        "rando_state.Flash",
        UberIdentifier { uber_group: 6, uber_id: 1062 },
    ),
    (
        "rando_state.Spike",
        UberIdentifier { uber_group: 6, uber_id: 1074 },
    ),
    (
        "rando_state.Regenerate",
        UberIdentifier { uber_group: 6, uber_id: 1077 },
    ),
    (
        "rando_state.SpiritArc",
        UberIdentifier { uber_group: 6, uber_id: 1097 },
    ),
    (
        "rando_state.SpiritSmash",
        UberIdentifier { uber_group: 6, uber_id: 1098 },
    ),
    (
        "rando_state.Torch",
        UberIdentifier { uber_group: 6, uber_id: 1099 },
    ),
    (
        "rando_state.SpiritEdge",
        UberIdentifier { uber_group: 6, uber_id: 1100 },
    ),
    (
        "rando_state.Burrow",
        UberIdentifier { uber_group: 6, uber_id: 1101 },
    ),
    (
        "rando_state.Dash",
        UberIdentifier { uber_group: 6, uber_id: 1102 },
    ),
    (
        "rando_state.WaterDash",
        UberIdentifier { uber_group: 6, uber_id: 1104 },
    ),
    (
        "rando_state.SpiritStar",
        UberIdentifier { uber_group: 6, uber_id: 1106 },
    ),
    (
        "rando_state.Seir",
        UberIdentifier { uber_group: 6, uber_id: 1108 },
    ),
    (
        "rando_state.Bow Charge",
        UberIdentifier { uber_group: 6, uber_id: 1109 },
    ),
    (
        "rando_state.Spirit Magnet",
        UberIdentifier { uber_group: 6, uber_id: 1112 },
    ),
    (
        "rando_state.Blaze",
        UberIdentifier { uber_group: 6, uber_id: 1115 },
    ),
    (
        "rando_state.Sentry",
        UberIdentifier { uber_group: 6, uber_id: 1116 },
    ),
    (
        "rando_state.Flap",
        UberIdentifier { uber_group: 6, uber_id: 1118 },
    ),
    (
        "rando_state.Weapon Charge",
        UberIdentifier { uber_group: 6, uber_id: 1119 },
    ),
    (
        "rando_state.DamageUpgrade1",
        UberIdentifier { uber_group: 6, uber_id: 1120 },
    ),
    (
        "rando_state.DamageUpgrade2",
        UberIdentifier { uber_group: 6, uber_id: 1121 },
    ),
    (
        "rando_state.Clean Water",
        UberIdentifier { uber_group: 6, uber_id: 2000 },
    ),
    (
        "rando_state.Collected Keystones",
        UberIdentifier { uber_group: 6, uber_id: 0 },
    ),
    (
        "rando_state.Purchased Keystones",
        UberIdentifier { uber_group: 6, uber_id: 1 },
    ),
    (
        "rando_state.Pickups Collected",
        UberIdentifier { uber_group: 6, uber_id: 2 },
    ),
    (
        "rando_state.Spirit Light Collected",
        UberIdentifier { uber_group: 6, uber_id: 3 },
    ),
    (
        "rando_state.Spirit Light Spent",
        UberIdentifier { uber_group: 6, uber_id: 4 },
    ),
    (
        "rando_state.Ore Collected",
        UberIdentifier { uber_group: 6, uber_id: 5 },
    ),
    (
        "rando_state.Ore Spent",
        UberIdentifier { uber_group: 6, uber_id: 6 },
    ),
    (
        "rando_state.Marsh Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10000 },
    ),
    (
        "rando_state.Hollow Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10001 },
    ),
    (
        "rando_state.Glades Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10002 },
    ),
    (
        "rando_state.Wellspring Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10003 },
    ),
    (
        "rando_state.Burrows Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10004 },
    ),
    (
        "rando_state.Woods Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10005 },
    ),
    (
        "rando_state.Reach Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10006 },
    ),
    (
        "rando_state.Pools Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10007 },
    ),
    (
        "rando_state.Depths Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10008 },
    ),
    (
        "rando_state.Wastes Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10009 },
    ),
    (
        "rando_state.Willow Key Item Hint",
        UberIdentifier { uber_group: 6, uber_id: 10011 },
    ),
    (
        "rando_config.glades_tp_fix",
        UberIdentifier { uber_group: 7, uber_id: 0 },
    ),
    (
        "rando_config.prevent_map_reactivate_tps",
        UberIdentifier { uber_group: 7, uber_id: 1 },
    ),
    (
        "rando_config.marsh_starts_sunny",
        UberIdentifier { uber_group: 7, uber_id: 2 },
    ),
    (
        "rando_config.howl_starts_dead",
        UberIdentifier { uber_group: 7, uber_id: 3 },
    ),
    (
        "rando_config.enable_vanilla_regen_tree",
        UberIdentifier { uber_group: 7, uber_id: 4 },
    ),
    (
        "rando_config.disable_tree_check_for_rain",
        UberIdentifier { uber_group: 7, uber_id: 5 },
    ),
    (
        "map_filter.show_spoiler",
        UberIdentifier { uber_group: 8, uber_id: 70 },
    ),
    (
        "plando_vars.100_bool",
        UberIdentifier { uber_group: 9, uber_id: 100 },
    ),
    (
        "plando_vars.101_bool",
        UberIdentifier { uber_group: 9, uber_id: 101 },
    ),
    (
        "plando_vars.102_bool",
        UberIdentifier { uber_group: 9, uber_id: 102 },
    ),
    (
        "plando_vars.103_bool",
        UberIdentifier { uber_group: 9, uber_id: 103 },
    ),
    (
        "plando_vars.104_bool",
        UberIdentifier { uber_group: 9, uber_id: 104 },
    ),
    (
        "plando_vars.105_bool",
        UberIdentifier { uber_group: 9, uber_id: 105 },
    ),
    (
        "plando_vars.106_bool",
        UberIdentifier { uber_group: 9, uber_id: 106 },
    ),
    (
        "plando_vars.107_bool",
        UberIdentifier { uber_group: 9, uber_id: 107 },
    ),
    (
        "plando_vars.108_bool",
        UberIdentifier { uber_group: 9, uber_id: 108 },
    ),
    (
        "plando_vars.109_bool",
        UberIdentifier { uber_group: 9, uber_id: 109 },
    ),
    (
        "plando_vars.110_bool",
        UberIdentifier { uber_group: 9, uber_id: 110 },
    ),
    (
        "plando_vars.111_bool",
        UberIdentifier { uber_group: 9, uber_id: 111 },
    ),
    (
        "plando_vars.112_bool",
        UberIdentifier { uber_group: 9, uber_id: 112 },
    ),
    (
        "plando_vars.113_bool",
        UberIdentifier { uber_group: 9, uber_id: 113 },
    ),
    (
        "plando_vars.114_bool",
        UberIdentifier { uber_group: 9, uber_id: 114 },
    ),
    (
        "plando_vars.115_bool",
        UberIdentifier { uber_group: 9, uber_id: 115 },
    ),
    (
        "plando_vars.116_bool",
        UberIdentifier { uber_group: 9, uber_id: 116 },
    ),
    (
        "plando_vars.117_bool",
        UberIdentifier { uber_group: 9, uber_id: 117 },
    ),
    (
        "plando_vars.118_bool",
        UberIdentifier { uber_group: 9, uber_id: 118 },
    ),
    (
        "plando_vars.119_bool",
        UberIdentifier { uber_group: 9, uber_id: 119 },
    ),
    (
        "plando_vars.120_bool",
        UberIdentifier { uber_group: 9, uber_id: 120 },
    ),
    (
        "plando_vars.121_bool",
        UberIdentifier { uber_group: 9, uber_id: 121 },
    ),
    (
        "plando_vars.122_bool",
        UberIdentifier { uber_group: 9, uber_id: 122 },
    ),
    (
        "plando_vars.123_bool",
        UberIdentifier { uber_group: 9, uber_id: 123 },
    ),
    (
        "plando_vars.124_bool",
        UberIdentifier { uber_group: 9, uber_id: 124 },
    ),
    (
        "plando_vars.125_bool",
        UberIdentifier { uber_group: 9, uber_id: 125 },
    ),
    (
        "plando_vars.126_bool",
        UberIdentifier { uber_group: 9, uber_id: 126 },
    ),
    (
        "plando_vars.127_bool",
        UberIdentifier { uber_group: 9, uber_id: 127 },
    ),
    (
        "plando_vars.128_bool",
        UberIdentifier { uber_group: 9, uber_id: 128 },
    ),
    (
        "plando_vars.129_bool",
        UberIdentifier { uber_group: 9, uber_id: 129 },
    ),
    (
        "plando_vars.130_bool",
        UberIdentifier { uber_group: 9, uber_id: 130 },
    ),
    (
        "plando_vars.131_bool",
        UberIdentifier { uber_group: 9, uber_id: 131 },
    ),
    (
        "plando_vars.132_bool",
        UberIdentifier { uber_group: 9, uber_id: 132 },
    ),
    (
        "plando_vars.133_bool",
        UberIdentifier { uber_group: 9, uber_id: 133 },
    ),
    (
        "plando_vars.134_bool",
        UberIdentifier { uber_group: 9, uber_id: 134 },
    ),
    (
        "plando_vars.135_bool",
        UberIdentifier { uber_group: 9, uber_id: 135 },
    ),
    (
        "plando_vars.136_bool",
        UberIdentifier { uber_group: 9, uber_id: 136 },
    ),
    (
        "plando_vars.137_bool",
        UberIdentifier { uber_group: 9, uber_id: 137 },
    ),
    (
        "plando_vars.138_bool",
        UberIdentifier { uber_group: 9, uber_id: 138 },
    ),
    (
        "plando_vars.139_bool",
        UberIdentifier { uber_group: 9, uber_id: 139 },
    ),
    (
        "plando_vars.140_bool",
        UberIdentifier { uber_group: 9, uber_id: 140 },
    ),
    (
        "plando_vars.141_bool",
        UberIdentifier { uber_group: 9, uber_id: 141 },
    ),
    (
        "plando_vars.142_bool",
        UberIdentifier { uber_group: 9, uber_id: 142 },
    ),
    (
        "plando_vars.143_bool",
        UberIdentifier { uber_group: 9, uber_id: 143 },
    ),
    (
        "plando_vars.144_bool",
        UberIdentifier { uber_group: 9, uber_id: 144 },
    ),
    (
        "plando_vars.145_bool",
        UberIdentifier { uber_group: 9, uber_id: 145 },
    ),
    (
        "plando_vars.146_bool",
        UberIdentifier { uber_group: 9, uber_id: 146 },
    ),
    (
        "plando_vars.147_bool",
        UberIdentifier { uber_group: 9, uber_id: 147 },
    ),
    (
        "plando_vars.148_bool",
        UberIdentifier { uber_group: 9, uber_id: 148 },
    ),
    (
        "plando_vars.149_bool",
        UberIdentifier { uber_group: 9, uber_id: 149 },
    ),
    (
        "plando_vars.0_int",
        UberIdentifier { uber_group: 9, uber_id: 0 },
    ),
    (
        "plando_vars.1_int",
        UberIdentifier { uber_group: 9, uber_id: 1 },
    ),
    (
        "plando_vars.2_int",
        UberIdentifier { uber_group: 9, uber_id: 2 },
    ),
    (
        "plando_vars.3_int",
        UberIdentifier { uber_group: 9, uber_id: 3 },
    ),
    (
        "plando_vars.4_int",
        UberIdentifier { uber_group: 9, uber_id: 4 },
    ),
    (
        "plando_vars.5_int",
        UberIdentifier { uber_group: 9, uber_id: 5 },
    ),
    (
        "plando_vars.6_int",
        UberIdentifier { uber_group: 9, uber_id: 6 },
    ),
    (
        "plando_vars.7_int",
        UberIdentifier { uber_group: 9, uber_id: 7 },
    ),
    (
        "plando_vars.8_int",
        UberIdentifier { uber_group: 9, uber_id: 8 },
    ),
    (
        "plando_vars.9_int",
        UberIdentifier { uber_group: 9, uber_id: 9 },
    ),
    (
        "plando_vars.10_int",
        UberIdentifier { uber_group: 9, uber_id: 10 },
    ),
    (
        "plando_vars.11_int",
        UberIdentifier { uber_group: 9, uber_id: 11 },
    ),
    (
        "plando_vars.12_int",
        UberIdentifier { uber_group: 9, uber_id: 12 },
    ),
    (
        "plando_vars.13_int",
        UberIdentifier { uber_group: 9, uber_id: 13 },
    ),
    (
        "plando_vars.14_int",
        UberIdentifier { uber_group: 9, uber_id: 14 },
    ),
    (
        "plando_vars.15_int",
        UberIdentifier { uber_group: 9, uber_id: 15 },
    ),
    (
        "plando_vars.16_int",
        UberIdentifier { uber_group: 9, uber_id: 16 },
    ),
    (
        "plando_vars.17_int",
        UberIdentifier { uber_group: 9, uber_id: 17 },
    ),
    (
        "plando_vars.18_int",
        UberIdentifier { uber_group: 9, uber_id: 18 },
    ),
    (
        "plando_vars.19_int",
        UberIdentifier { uber_group: 9, uber_id: 19 },
    ),
    (
        "plando_vars.20_int",
        UberIdentifier { uber_group: 9, uber_id: 20 },
    ),
    (
        "plando_vars.21_int",
        UberIdentifier { uber_group: 9, uber_id: 21 },
    ),
    (
        "plando_vars.22_int",
        UberIdentifier { uber_group: 9, uber_id: 22 },
    ),
    (
        "plando_vars.23_int",
        UberIdentifier { uber_group: 9, uber_id: 23 },
    ),
    (
        "plando_vars.24_int",
        UberIdentifier { uber_group: 9, uber_id: 24 },
    ),
    (
        "plando_vars.25_int",
        UberIdentifier { uber_group: 9, uber_id: 25 },
    ),
    (
        "plando_vars.26_int",
        UberIdentifier { uber_group: 9, uber_id: 26 },
    ),
    (
        "plando_vars.27_int",
        UberIdentifier { uber_group: 9, uber_id: 27 },
    ),
    (
        "plando_vars.28_int",
        UberIdentifier { uber_group: 9, uber_id: 28 },
    ),
    (
        "plando_vars.29_int",
        UberIdentifier { uber_group: 9, uber_id: 29 },
    ),
    (
        "plando_vars.30_int",
        UberIdentifier { uber_group: 9, uber_id: 30 },
    ),
    (
        "plando_vars.31_int",
        UberIdentifier { uber_group: 9, uber_id: 31 },
    ),
    (
        "plando_vars.32_int",
        UberIdentifier { uber_group: 9, uber_id: 32 },
    ),
    (
        "plando_vars.33_int",
        UberIdentifier { uber_group: 9, uber_id: 33 },
    ),
    (
        "plando_vars.34_int",
        UberIdentifier { uber_group: 9, uber_id: 34 },
    ),
    (
        "plando_vars.35_int",
        UberIdentifier { uber_group: 9, uber_id: 35 },
    ),
    (
        "plando_vars.36_int",
        UberIdentifier { uber_group: 9, uber_id: 36 },
    ),
    (
        "plando_vars.37_int",
        UberIdentifier { uber_group: 9, uber_id: 37 },
    ),
    (
        "plando_vars.38_int",
        UberIdentifier { uber_group: 9, uber_id: 38 },
    ),
    (
        "plando_vars.39_int",
        UberIdentifier { uber_group: 9, uber_id: 39 },
    ),
    (
        "plando_vars.40_int",
        UberIdentifier { uber_group: 9, uber_id: 40 },
    ),
    (
        "plando_vars.41_int",
        UberIdentifier { uber_group: 9, uber_id: 41 },
    ),
    (
        "plando_vars.42_int",
        UberIdentifier { uber_group: 9, uber_id: 42 },
    ),
    (
        "plando_vars.43_int",
        UberIdentifier { uber_group: 9, uber_id: 43 },
    ),
    (
        "plando_vars.44_int",
        UberIdentifier { uber_group: 9, uber_id: 44 },
    ),
    (
        "plando_vars.45_int",
        UberIdentifier { uber_group: 9, uber_id: 45 },
    ),
    (
        "plando_vars.46_int",
        UberIdentifier { uber_group: 9, uber_id: 46 },
    ),
    (
        "plando_vars.47_int",
        UberIdentifier { uber_group: 9, uber_id: 47 },
    ),
    (
        "plando_vars.48_int",
        UberIdentifier { uber_group: 9, uber_id: 48 },
    ),
    (
        "plando_vars.49_int",
        UberIdentifier { uber_group: 9, uber_id: 49 },
    ),
    (
        "plando_vars.50_int",
        UberIdentifier { uber_group: 9, uber_id: 50 },
    ),
    (
        "plando_vars.51_int",
        UberIdentifier { uber_group: 9, uber_id: 51 },
    ),
    (
        "plando_vars.52_int",
        UberIdentifier { uber_group: 9, uber_id: 52 },
    ),
    (
        "plando_vars.53_int",
        UberIdentifier { uber_group: 9, uber_id: 53 },
    ),
    (
        "plando_vars.54_int",
        UberIdentifier { uber_group: 9, uber_id: 54 },
    ),
    (
        "plando_vars.55_int",
        UberIdentifier { uber_group: 9, uber_id: 55 },
    ),
    (
        "plando_vars.56_int",
        UberIdentifier { uber_group: 9, uber_id: 56 },
    ),
    (
        "plando_vars.57_int",
        UberIdentifier { uber_group: 9, uber_id: 57 },
    ),
    (
        "plando_vars.58_int",
        UberIdentifier { uber_group: 9, uber_id: 58 },
    ),
    (
        "plando_vars.59_int",
        UberIdentifier { uber_group: 9, uber_id: 59 },
    ),
    (
        "plando_vars.60_int",
        UberIdentifier { uber_group: 9, uber_id: 60 },
    ),
    (
        "plando_vars.61_int",
        UberIdentifier { uber_group: 9, uber_id: 61 },
    ),
    (
        "plando_vars.62_int",
        UberIdentifier { uber_group: 9, uber_id: 62 },
    ),
    (
        "plando_vars.63_int",
        UberIdentifier { uber_group: 9, uber_id: 63 },
    ),
    (
        "plando_vars.64_int",
        UberIdentifier { uber_group: 9, uber_id: 64 },
    ),
    (
        "plando_vars.65_int",
        UberIdentifier { uber_group: 9, uber_id: 65 },
    ),
    (
        "plando_vars.66_int",
        UberIdentifier { uber_group: 9, uber_id: 66 },
    ),
    (
        "plando_vars.67_int",
        UberIdentifier { uber_group: 9, uber_id: 67 },
    ),
    (
        "plando_vars.68_int",
        UberIdentifier { uber_group: 9, uber_id: 68 },
    ),
    (
        "plando_vars.69_int",
        UberIdentifier { uber_group: 9, uber_id: 69 },
    ),
    (
        "plando_vars.70_int",
        UberIdentifier { uber_group: 9, uber_id: 70 },
    ),
    (
        "plando_vars.71_int",
        UberIdentifier { uber_group: 9, uber_id: 71 },
    ),
    (
        "plando_vars.72_int",
        UberIdentifier { uber_group: 9, uber_id: 72 },
    ),
    (
        "plando_vars.73_int",
        UberIdentifier { uber_group: 9, uber_id: 73 },
    ),
    (
        "plando_vars.74_int",
        UberIdentifier { uber_group: 9, uber_id: 74 },
    ),
    (
        "plando_vars.75_int",
        UberIdentifier { uber_group: 9, uber_id: 75 },
    ),
    (
        "plando_vars.76_int",
        UberIdentifier { uber_group: 9, uber_id: 76 },
    ),
    (
        "plando_vars.77_int",
        UberIdentifier { uber_group: 9, uber_id: 77 },
    ),
    (
        "plando_vars.78_int",
        UberIdentifier { uber_group: 9, uber_id: 78 },
    ),
    (
        "plando_vars.79_int",
        UberIdentifier { uber_group: 9, uber_id: 79 },
    ),
    (
        "plando_vars.80_int",
        UberIdentifier { uber_group: 9, uber_id: 80 },
    ),
    (
        "plando_vars.81_int",
        UberIdentifier { uber_group: 9, uber_id: 81 },
    ),
    (
        "plando_vars.82_int",
        UberIdentifier { uber_group: 9, uber_id: 82 },
    ),
    (
        "plando_vars.83_int",
        UberIdentifier { uber_group: 9, uber_id: 83 },
    ),
    (
        "plando_vars.84_int",
        UberIdentifier { uber_group: 9, uber_id: 84 },
    ),
    (
        "plando_vars.85_int",
        UberIdentifier { uber_group: 9, uber_id: 85 },
    ),
    (
        "plando_vars.86_int",
        UberIdentifier { uber_group: 9, uber_id: 86 },
    ),
    (
        "plando_vars.87_int",
        UberIdentifier { uber_group: 9, uber_id: 87 },
    ),
    (
        "plando_vars.88_int",
        UberIdentifier { uber_group: 9, uber_id: 88 },
    ),
    (
        "plando_vars.89_int",
        UberIdentifier { uber_group: 9, uber_id: 89 },
    ),
    (
        "plando_vars.90_int",
        UberIdentifier { uber_group: 9, uber_id: 90 },
    ),
    (
        "plando_vars.91_int",
        UberIdentifier { uber_group: 9, uber_id: 91 },
    ),
    (
        "plando_vars.92_int",
        UberIdentifier { uber_group: 9, uber_id: 92 },
    ),
    (
        "plando_vars.93_int",
        UberIdentifier { uber_group: 9, uber_id: 93 },
    ),
    (
        "plando_vars.94_int",
        UberIdentifier { uber_group: 9, uber_id: 94 },
    ),
    (
        "plando_vars.95_int",
        UberIdentifier { uber_group: 9, uber_id: 95 },
    ),
    (
        "plando_vars.96_int",
        UberIdentifier { uber_group: 9, uber_id: 96 },
    ),
    (
        "plando_vars.97_int",
        UberIdentifier { uber_group: 9, uber_id: 97 },
    ),
    (
        "plando_vars.98_int",
        UberIdentifier { uber_group: 9, uber_id: 98 },
    ),
    (
        "plando_vars.99_int",
        UberIdentifier { uber_group: 9, uber_id: 99 },
    ),
    (
        "plando_vars.150_float",
        UberIdentifier { uber_group: 9, uber_id: 150 },
    ),
    (
        "plando_vars.151_float",
        UberIdentifier { uber_group: 9, uber_id: 151 },
    ),
    (
        "plando_vars.152_float",
        UberIdentifier { uber_group: 9, uber_id: 152 },
    ),
    (
        "plando_vars.153_float",
        UberIdentifier { uber_group: 9, uber_id: 153 },
    ),
    (
        "plando_vars.154_float",
        UberIdentifier { uber_group: 9, uber_id: 154 },
    ),
    (
        "plando_vars.155_float",
        UberIdentifier { uber_group: 9, uber_id: 155 },
    ),
    (
        "plando_vars.156_float",
        UberIdentifier { uber_group: 9, uber_id: 156 },
    ),
    (
        "plando_vars.157_float",
        UberIdentifier { uber_group: 9, uber_id: 157 },
    ),
    (
        "plando_vars.158_float",
        UberIdentifier { uber_group: 9, uber_id: 158 },
    ),
    (
        "plando_vars.159_float",
        UberIdentifier { uber_group: 9, uber_id: 159 },
    ),
    (
        "plando_vars.160_float",
        UberIdentifier { uber_group: 9, uber_id: 160 },
    ),
    (
        "plando_vars.161_float",
        UberIdentifier { uber_group: 9, uber_id: 161 },
    ),
    (
        "plando_vars.162_float",
        UberIdentifier { uber_group: 9, uber_id: 162 },
    ),
    (
        "plando_vars.163_float",
        UberIdentifier { uber_group: 9, uber_id: 163 },
    ),
    (
        "plando_vars.164_float",
        UberIdentifier { uber_group: 9, uber_id: 164 },
    ),
    (
        "plando_vars.165_float",
        UberIdentifier { uber_group: 9, uber_id: 165 },
    ),
    (
        "plando_vars.166_float",
        UberIdentifier { uber_group: 9, uber_id: 166 },
    ),
    (
        "plando_vars.167_float",
        UberIdentifier { uber_group: 9, uber_id: 167 },
    ),
    (
        "plando_vars.168_float",
        UberIdentifier { uber_group: 9, uber_id: 168 },
    ),
    (
        "plando_vars.169_float",
        UberIdentifier { uber_group: 9, uber_id: 169 },
    ),
    (
        "plando_vars.170_float",
        UberIdentifier { uber_group: 9, uber_id: 170 },
    ),
    (
        "plando_vars.171_float",
        UberIdentifier { uber_group: 9, uber_id: 171 },
    ),
    (
        "plando_vars.172_float",
        UberIdentifier { uber_group: 9, uber_id: 172 },
    ),
    (
        "plando_vars.173_float",
        UberIdentifier { uber_group: 9, uber_id: 173 },
    ),
    (
        "plando_vars.174_float",
        UberIdentifier { uber_group: 9, uber_id: 174 },
    ),
    (
        "bingo_state.Squares",
        UberIdentifier { uber_group: 10, uber_id: 0 },
    ),
    (
        "bingo_state.Lines",
        UberIdentifier { uber_group: 10, uber_id: 1 },
    ),
    (
        "bingo_state.Rank",
        UberIdentifier { uber_group: 10, uber_id: 2 },
    ),
    (
        "bingo_state.Kills",
        UberIdentifier { uber_group: 10, uber_id: 10 },
    ),
    (
        "bingo_state.SwordKills",
        UberIdentifier { uber_group: 10, uber_id: 11 },
    ),
    (
        "bingo_state.HammerKills",
        UberIdentifier { uber_group: 10, uber_id: 12 },
    ),
    (
        "bingo_state.BowKills",
        UberIdentifier { uber_group: 10, uber_id: 13 },
    ),
    (
        "bingo_state.SpearKills",
        UberIdentifier { uber_group: 10, uber_id: 14 },
    ),
    (
        "bingo_state.SentryKills",
        UberIdentifier { uber_group: 10, uber_id: 15 },
    ),
    (
        "bingo_state.BlazeKills",
        UberIdentifier { uber_group: 10, uber_id: 16 },
    ),
    (
        "bingo_state.GrenadeKills",
        UberIdentifier { uber_group: 10, uber_id: 17 },
    ),
    (
        "bingo_state.BurnDoTKills",
        UberIdentifier { uber_group: 10, uber_id: 18 },
    ),
    (
        "bingo_state.ShurikenKills",
        UberIdentifier { uber_group: 10, uber_id: 19 },
    ),
    (
        "bingo_state.LaunchKills",
        UberIdentifier { uber_group: 10, uber_id: 20 },
    ),
    (
        "bingo_state.FlashKills",
        UberIdentifier { uber_group: 10, uber_id: 21 },
    ),
    (
        "bingo_state.BashKills",
        UberIdentifier { uber_group: 10, uber_id: 22 },
    ),
    (
        "bingo_state.DrownedEnemies",
        UberIdentifier { uber_group: 10, uber_id: 23 },
    ),
    (
        "bingo_state.MinerKills",
        UberIdentifier { uber_group: 10, uber_id: 40 },
    ),
    (
        "bingo_state.FlierKills",
        UberIdentifier { uber_group: 10, uber_id: 41 },
    ),
    (
        "bingo_state.TentaKills",
        UberIdentifier { uber_group: 10, uber_id: 42 },
    ),
    (
        "bingo_state.SlimeKills",
        UberIdentifier { uber_group: 10, uber_id: 43 },
    ),
    (
        "bingo_state.FishKills",
        UberIdentifier { uber_group: 10, uber_id: 44 },
    ),
    (
        "bingo_state.ExploderKills",
        UberIdentifier { uber_group: 10, uber_id: 45 },
    ),
    (
        "appliers_serialization.0_id",
        UberIdentifier { uber_group: 11, uber_id: 0 },
    ),
    (
        "appliers_serialization.1_value",
        UberIdentifier { uber_group: 11, uber_id: 1 },
    ),
    (
        "appliers_serialization.2_id",
        UberIdentifier { uber_group: 11, uber_id: 2 },
    ),
    (
        "appliers_serialization.3_value",
        UberIdentifier { uber_group: 11, uber_id: 3 },
    ),
    (
        "appliers_serialization.4_id",
        UberIdentifier { uber_group: 11, uber_id: 4 },
    ),
    (
        "appliers_serialization.5_value",
        UberIdentifier { uber_group: 11, uber_id: 5 },
    ),
    (
        "appliers_serialization.6_id",
        UberIdentifier { uber_group: 11, uber_id: 6 },
    ),
    (
        "appliers_serialization.7_value",
        UberIdentifier { uber_group: 11, uber_id: 7 },
    ),
    (
        "appliers_serialization.8_id",
        UberIdentifier { uber_group: 11, uber_id: 8 },
    ),
    (
        "appliers_serialization.9_value",
        UberIdentifier { uber_group: 11, uber_id: 9 },
    ),
    (
        "appliers_serialization.10_id",
        UberIdentifier { uber_group: 11, uber_id: 10 },
    ),
    (
        "appliers_serialization.11_value",
        UberIdentifier { uber_group: 11, uber_id: 11 },
    ),
    (
        "appliers_serialization.12_id",
        UberIdentifier { uber_group: 11, uber_id: 12 },
    ),
    (
        "appliers_serialization.13_value",
        UberIdentifier { uber_group: 11, uber_id: 13 },
    ),
    (
        "appliers_serialization.14_id",
        UberIdentifier { uber_group: 11, uber_id: 14 },
    ),
    (
        "appliers_serialization.15_value",
        UberIdentifier { uber_group: 11, uber_id: 15 },
    ),
    (
        "appliers_serialization.16_id",
        UberIdentifier { uber_group: 11, uber_id: 16 },
    ),
    (
        "appliers_serialization.17_value",
        UberIdentifier { uber_group: 11, uber_id: 17 },
    ),
    (
        "appliers_serialization.18_id",
        UberIdentifier { uber_group: 11, uber_id: 18 },
    ),
    (
        "appliers_serialization.19_value",
        UberIdentifier { uber_group: 11, uber_id: 19 },
    ),
    (
        "appliers_serialization.20_id",
        UberIdentifier { uber_group: 11, uber_id: 20 },
    ),
    (
        "appliers_serialization.21_value",
        UberIdentifier { uber_group: 11, uber_id: 21 },
    ),
    (
        "appliers_serialization.22_id",
        UberIdentifier { uber_group: 11, uber_id: 22 },
    ),
    (
        "appliers_serialization.23_value",
        UberIdentifier { uber_group: 11, uber_id: 23 },
    ),
    (
        "appliers_serialization.24_id",
        UberIdentifier { uber_group: 11, uber_id: 24 },
    ),
    (
        "appliers_serialization.25_value",
        UberIdentifier { uber_group: 11, uber_id: 25 },
    ),
    (
        "appliers_serialization.26_id",
        UberIdentifier { uber_group: 11, uber_id: 26 },
    ),
    (
        "appliers_serialization.27_value",
        UberIdentifier { uber_group: 11, uber_id: 27 },
    ),
    (
        "appliers_serialization.28_id",
        UberIdentifier { uber_group: 11, uber_id: 28 },
    ),
    (
        "appliers_serialization.29_value",
        UberIdentifier { uber_group: 11, uber_id: 29 },
    ),
    (
        "appliers_serialization.30_id",
        UberIdentifier { uber_group: 11, uber_id: 30 },
    ),
    (
        "appliers_serialization.31_value",
        UberIdentifier { uber_group: 11, uber_id: 31 },
    ),
    (
        "appliers_serialization.32_id",
        UberIdentifier { uber_group: 11, uber_id: 32 },
    ),
    (
        "appliers_serialization.33_value",
        UberIdentifier { uber_group: 11, uber_id: 33 },
    ),
    (
        "appliers_serialization.34_id",
        UberIdentifier { uber_group: 11, uber_id: 34 },
    ),
    (
        "appliers_serialization.35_value",
        UberIdentifier { uber_group: 11, uber_id: 35 },
    ),
    (
        "appliers_serialization.36_id",
        UberIdentifier { uber_group: 11, uber_id: 36 },
    ),
    (
        "appliers_serialization.37_value",
        UberIdentifier { uber_group: 11, uber_id: 37 },
    ),
    (
        "appliers_serialization.38_id",
        UberIdentifier { uber_group: 11, uber_id: 38 },
    ),
    (
        "appliers_serialization.39_value",
        UberIdentifier { uber_group: 11, uber_id: 39 },
    ),
    (
        "appliers_serialization.40_id",
        UberIdentifier { uber_group: 11, uber_id: 40 },
    ),
    (
        "appliers_serialization.41_value",
        UberIdentifier { uber_group: 11, uber_id: 41 },
    ),
    (
        "appliers_serialization.42_id",
        UberIdentifier { uber_group: 11, uber_id: 42 },
    ),
    (
        "appliers_serialization.43_value",
        UberIdentifier { uber_group: 11, uber_id: 43 },
    ),
    (
        "appliers_serialization.44_id",
        UberIdentifier { uber_group: 11, uber_id: 44 },
    ),
    (
        "appliers_serialization.45_value",
        UberIdentifier { uber_group: 11, uber_id: 45 },
    ),
    (
        "appliers_serialization.46_id",
        UberIdentifier { uber_group: 11, uber_id: 46 },
    ),
    (
        "appliers_serialization.47_value",
        UberIdentifier { uber_group: 11, uber_id: 47 },
    ),
    (
        "appliers_serialization.48_id",
        UberIdentifier { uber_group: 11, uber_id: 48 },
    ),
    (
        "appliers_serialization.49_value",
        UberIdentifier { uber_group: 11, uber_id: 49 },
    ),
    (
        "appliers_serialization.50_id",
        UberIdentifier { uber_group: 11, uber_id: 50 },
    ),
    (
        "appliers_serialization.51_value",
        UberIdentifier { uber_group: 11, uber_id: 51 },
    ),
    (
        "appliers_serialization.52_id",
        UberIdentifier { uber_group: 11, uber_id: 52 },
    ),
    (
        "appliers_serialization.53_value",
        UberIdentifier { uber_group: 11, uber_id: 53 },
    ),
    (
        "appliers_serialization.54_id",
        UberIdentifier { uber_group: 11, uber_id: 54 },
    ),
    (
        "appliers_serialization.55_value",
        UberIdentifier { uber_group: 11, uber_id: 55 },
    ),
    (
        "appliers_serialization.56_id",
        UberIdentifier { uber_group: 11, uber_id: 56 },
    ),
    (
        "appliers_serialization.57_value",
        UberIdentifier { uber_group: 11, uber_id: 57 },
    ),
    (
        "appliers_serialization.58_id",
        UberIdentifier { uber_group: 11, uber_id: 58 },
    ),
    (
        "appliers_serialization.59_value",
        UberIdentifier { uber_group: 11, uber_id: 59 },
    ),
    (
        "appliers_serialization.60_id",
        UberIdentifier { uber_group: 11, uber_id: 60 },
    ),
    (
        "appliers_serialization.61_value",
        UberIdentifier { uber_group: 11, uber_id: 61 },
    ),
    (
        "appliers_serialization.62_id",
        UberIdentifier { uber_group: 11, uber_id: 62 },
    ),
    (
        "appliers_serialization.63_value",
        UberIdentifier { uber_group: 11, uber_id: 63 },
    ),
    (
        "appliers_serialization.64_id",
        UberIdentifier { uber_group: 11, uber_id: 64 },
    ),
    (
        "appliers_serialization.65_value",
        UberIdentifier { uber_group: 11, uber_id: 65 },
    ),
    (
        "appliers_serialization.66_id",
        UberIdentifier { uber_group: 11, uber_id: 66 },
    ),
    (
        "appliers_serialization.67_value",
        UberIdentifier { uber_group: 11, uber_id: 67 },
    ),
    (
        "appliers_serialization.68_id",
        UberIdentifier { uber_group: 11, uber_id: 68 },
    ),
    (
        "appliers_serialization.69_value",
        UberIdentifier { uber_group: 11, uber_id: 69 },
    ),
    (
        "appliers_serialization.70_id",
        UberIdentifier { uber_group: 11, uber_id: 70 },
    ),
    (
        "appliers_serialization.71_value",
        UberIdentifier { uber_group: 11, uber_id: 71 },
    ),
    (
        "appliers_serialization.72_id",
        UberIdentifier { uber_group: 11, uber_id: 72 },
    ),
    (
        "appliers_serialization.73_value",
        UberIdentifier { uber_group: 11, uber_id: 73 },
    ),
    (
        "appliers_serialization.74_id",
        UberIdentifier { uber_group: 11, uber_id: 74 },
    ),
    (
        "appliers_serialization.75_value",
        UberIdentifier { uber_group: 11, uber_id: 75 },
    ),
    (
        "appliers_serialization.76_id",
        UberIdentifier { uber_group: 11, uber_id: 76 },
    ),
    (
        "appliers_serialization.77_value",
        UberIdentifier { uber_group: 11, uber_id: 77 },
    ),
    (
        "appliers_serialization.78_id",
        UberIdentifier { uber_group: 11, uber_id: 78 },
    ),
    (
        "appliers_serialization.79_value",
        UberIdentifier { uber_group: 11, uber_id: 79 },
    ),
    (
        "appliers_serialization.80_id",
        UberIdentifier { uber_group: 11, uber_id: 80 },
    ),
    (
        "appliers_serialization.81_value",
        UberIdentifier { uber_group: 11, uber_id: 81 },
    ),
    (
        "appliers_serialization.82_id",
        UberIdentifier { uber_group: 11, uber_id: 82 },
    ),
    (
        "appliers_serialization.83_value",
        UberIdentifier { uber_group: 11, uber_id: 83 },
    ),
    (
        "appliers_serialization.84_id",
        UberIdentifier { uber_group: 11, uber_id: 84 },
    ),
    (
        "appliers_serialization.85_value",
        UberIdentifier { uber_group: 11, uber_id: 85 },
    ),
    (
        "appliers_serialization.86_id",
        UberIdentifier { uber_group: 11, uber_id: 86 },
    ),
    (
        "appliers_serialization.87_value",
        UberIdentifier { uber_group: 11, uber_id: 87 },
    ),
    (
        "appliers_serialization.88_id",
        UberIdentifier { uber_group: 11, uber_id: 88 },
    ),
    (
        "appliers_serialization.89_value",
        UberIdentifier { uber_group: 11, uber_id: 89 },
    ),
    (
        "appliers_serialization.90_id",
        UberIdentifier { uber_group: 11, uber_id: 90 },
    ),
    (
        "appliers_serialization.91_value",
        UberIdentifier { uber_group: 11, uber_id: 91 },
    ),
    (
        "appliers_serialization.92_id",
        UberIdentifier { uber_group: 11, uber_id: 92 },
    ),
    (
        "appliers_serialization.93_value",
        UberIdentifier { uber_group: 11, uber_id: 93 },
    ),
    (
        "appliers_serialization.94_id",
        UberIdentifier { uber_group: 11, uber_id: 94 },
    ),
    (
        "appliers_serialization.95_value",
        UberIdentifier { uber_group: 11, uber_id: 95 },
    ),
    (
        "appliers_serialization.96_id",
        UberIdentifier { uber_group: 11, uber_id: 96 },
    ),
    (
        "appliers_serialization.97_value",
        UberIdentifier { uber_group: 11, uber_id: 97 },
    ),
    (
        "appliers_serialization.98_id",
        UberIdentifier { uber_group: 11, uber_id: 98 },
    ),
    (
        "appliers_serialization.99_value",
        UberIdentifier { uber_group: 11, uber_id: 99 },
    ),
    (
        "multi_vars.0_multi",
        UberIdentifier { uber_group: 12, uber_id: 0 },
    ),
    (
        "multi_vars.1_multi",
        UberIdentifier { uber_group: 12, uber_id: 1 },
    ),
    (
        "multi_vars.2_multi",
        UberIdentifier { uber_group: 12, uber_id: 2 },
    ),
    (
        "multi_vars.3_multi",
        UberIdentifier { uber_group: 12, uber_id: 3 },
    ),
    (
        "multi_vars.4_multi",
        UberIdentifier { uber_group: 12, uber_id: 4 },
    ),
    (
        "multi_vars.5_multi",
        UberIdentifier { uber_group: 12, uber_id: 5 },
    ),
    (
        "multi_vars.6_multi",
        UberIdentifier { uber_group: 12, uber_id: 6 },
    ),
    (
        "multi_vars.7_multi",
        UberIdentifier { uber_group: 12, uber_id: 7 },
    ),
    (
        "multi_vars.8_multi",
        UberIdentifier { uber_group: 12, uber_id: 8 },
    ),
    (
        "multi_vars.9_multi",
        UberIdentifier { uber_group: 12, uber_id: 9 },
    ),
    (
        "multi_vars.10_multi",
        UberIdentifier { uber_group: 12, uber_id: 10 },
    ),
    (
        "multi_vars.11_multi",
        UberIdentifier { uber_group: 12, uber_id: 11 },
    ),
    (
        "multi_vars.12_multi",
        UberIdentifier { uber_group: 12, uber_id: 12 },
    ),
    (
        "multi_vars.13_multi",
        UberIdentifier { uber_group: 12, uber_id: 13 },
    ),
    (
        "multi_vars.14_multi",
        UberIdentifier { uber_group: 12, uber_id: 14 },
    ),
    (
        "multi_vars.15_multi",
        UberIdentifier { uber_group: 12, uber_id: 15 },
    ),
    (
        "multi_vars.16_multi",
        UberIdentifier { uber_group: 12, uber_id: 16 },
    ),
    (
        "multi_vars.17_multi",
        UberIdentifier { uber_group: 12, uber_id: 17 },
    ),
    (
        "multi_vars.18_multi",
        UberIdentifier { uber_group: 12, uber_id: 18 },
    ),
    (
        "multi_vars.19_multi",
        UberIdentifier { uber_group: 12, uber_id: 19 },
    ),
    (
        "multi_vars.20_multi",
        UberIdentifier { uber_group: 12, uber_id: 20 },
    ),
    (
        "multi_vars.21_multi",
        UberIdentifier { uber_group: 12, uber_id: 21 },
    ),
    (
        "multi_vars.22_multi",
        UberIdentifier { uber_group: 12, uber_id: 22 },
    ),
    (
        "multi_vars.23_multi",
        UberIdentifier { uber_group: 12, uber_id: 23 },
    ),
    (
        "multi_vars.24_multi",
        UberIdentifier { uber_group: 12, uber_id: 24 },
    ),
    (
        "multi_vars.25_multi",
        UberIdentifier { uber_group: 12, uber_id: 25 },
    ),
    (
        "multi_vars.26_multi",
        UberIdentifier { uber_group: 12, uber_id: 26 },
    ),
    (
        "multi_vars.27_multi",
        UberIdentifier { uber_group: 12, uber_id: 27 },
    ),
    (
        "multi_vars.28_multi",
        UberIdentifier { uber_group: 12, uber_id: 28 },
    ),
    (
        "multi_vars.29_multi",
        UberIdentifier { uber_group: 12, uber_id: 29 },
    ),
    (
        "multi_vars.30_multi",
        UberIdentifier { uber_group: 12, uber_id: 30 },
    ),
    (
        "multi_vars.31_multi",
        UberIdentifier { uber_group: 12, uber_id: 31 },
    ),
    (
        "multi_vars.32_multi",
        UberIdentifier { uber_group: 12, uber_id: 32 },
    ),
    (
        "multi_vars.33_multi",
        UberIdentifier { uber_group: 12, uber_id: 33 },
    ),
    (
        "multi_vars.34_multi",
        UberIdentifier { uber_group: 12, uber_id: 34 },
    ),
    (
        "multi_vars.35_multi",
        UberIdentifier { uber_group: 12, uber_id: 35 },
    ),
    (
        "multi_vars.36_multi",
        UberIdentifier { uber_group: 12, uber_id: 36 },
    ),
    (
        "multi_vars.37_multi",
        UberIdentifier { uber_group: 12, uber_id: 37 },
    ),
    (
        "multi_vars.38_multi",
        UberIdentifier { uber_group: 12, uber_id: 38 },
    ),
    (
        "multi_vars.39_multi",
        UberIdentifier { uber_group: 12, uber_id: 39 },
    ),
    (
        "multi_vars.40_multi",
        UberIdentifier { uber_group: 12, uber_id: 40 },
    ),
    (
        "multi_vars.41_multi",
        UberIdentifier { uber_group: 12, uber_id: 41 },
    ),
    (
        "multi_vars.42_multi",
        UberIdentifier { uber_group: 12, uber_id: 42 },
    ),
    (
        "multi_vars.43_multi",
        UberIdentifier { uber_group: 12, uber_id: 43 },
    ),
    (
        "multi_vars.44_multi",
        UberIdentifier { uber_group: 12, uber_id: 44 },
    ),
    (
        "multi_vars.45_multi",
        UberIdentifier { uber_group: 12, uber_id: 45 },
    ),
    (
        "multi_vars.46_multi",
        UberIdentifier { uber_group: 12, uber_id: 46 },
    ),
    (
        "multi_vars.47_multi",
        UberIdentifier { uber_group: 12, uber_id: 47 },
    ),
    (
        "multi_vars.48_multi",
        UberIdentifier { uber_group: 12, uber_id: 48 },
    ),
    (
        "multi_vars.49_multi",
        UberIdentifier { uber_group: 12, uber_id: 49 },
    ),
    (
        "multi_vars.50_multi",
        UberIdentifier { uber_group: 12, uber_id: 50 },
    ),
    (
        "multi_vars.51_multi",
        UberIdentifier { uber_group: 12, uber_id: 51 },
    ),
    (
        "multi_vars.52_multi",
        UberIdentifier { uber_group: 12, uber_id: 52 },
    ),
    (
        "multi_vars.53_multi",
        UberIdentifier { uber_group: 12, uber_id: 53 },
    ),
    (
        "multi_vars.54_multi",
        UberIdentifier { uber_group: 12, uber_id: 54 },
    ),
    (
        "multi_vars.55_multi",
        UberIdentifier { uber_group: 12, uber_id: 55 },
    ),
    (
        "multi_vars.56_multi",
        UberIdentifier { uber_group: 12, uber_id: 56 },
    ),
    (
        "multi_vars.57_multi",
        UberIdentifier { uber_group: 12, uber_id: 57 },
    ),
    (
        "multi_vars.58_multi",
        UberIdentifier { uber_group: 12, uber_id: 58 },
    ),
    (
        "multi_vars.59_multi",
        UberIdentifier { uber_group: 12, uber_id: 59 },
    ),
    (
        "multi_vars.60_multi",
        UberIdentifier { uber_group: 12, uber_id: 60 },
    ),
    (
        "multi_vars.61_multi",
        UberIdentifier { uber_group: 12, uber_id: 61 },
    ),
    (
        "multi_vars.62_multi",
        UberIdentifier { uber_group: 12, uber_id: 62 },
    ),
    (
        "multi_vars.63_multi",
        UberIdentifier { uber_group: 12, uber_id: 63 },
    ),
    (
        "multi_vars.64_multi",
        UberIdentifier { uber_group: 12, uber_id: 64 },
    ),
    (
        "multi_vars.65_multi",
        UberIdentifier { uber_group: 12, uber_id: 65 },
    ),
    (
        "multi_vars.66_multi",
        UberIdentifier { uber_group: 12, uber_id: 66 },
    ),
    (
        "multi_vars.67_multi",
        UberIdentifier { uber_group: 12, uber_id: 67 },
    ),
    (
        "multi_vars.68_multi",
        UberIdentifier { uber_group: 12, uber_id: 68 },
    ),
    (
        "multi_vars.69_multi",
        UberIdentifier { uber_group: 12, uber_id: 69 },
    ),
    (
        "multi_vars.70_multi",
        UberIdentifier { uber_group: 12, uber_id: 70 },
    ),
    (
        "multi_vars.71_multi",
        UberIdentifier { uber_group: 12, uber_id: 71 },
    ),
    (
        "multi_vars.72_multi",
        UberIdentifier { uber_group: 12, uber_id: 72 },
    ),
    (
        "multi_vars.73_multi",
        UberIdentifier { uber_group: 12, uber_id: 73 },
    ),
    (
        "multi_vars.74_multi",
        UberIdentifier { uber_group: 12, uber_id: 74 },
    ),
    (
        "multi_vars.75_multi",
        UberIdentifier { uber_group: 12, uber_id: 75 },
    ),
    (
        "multi_vars.76_multi",
        UberIdentifier { uber_group: 12, uber_id: 76 },
    ),
    (
        "multi_vars.77_multi",
        UberIdentifier { uber_group: 12, uber_id: 77 },
    ),
    (
        "multi_vars.78_multi",
        UberIdentifier { uber_group: 12, uber_id: 78 },
    ),
    (
        "multi_vars.79_multi",
        UberIdentifier { uber_group: 12, uber_id: 79 },
    ),
    (
        "multi_vars.80_multi",
        UberIdentifier { uber_group: 12, uber_id: 80 },
    ),
    (
        "multi_vars.81_multi",
        UberIdentifier { uber_group: 12, uber_id: 81 },
    ),
    (
        "multi_vars.82_multi",
        UberIdentifier { uber_group: 12, uber_id: 82 },
    ),
    (
        "multi_vars.83_multi",
        UberIdentifier { uber_group: 12, uber_id: 83 },
    ),
    (
        "multi_vars.84_multi",
        UberIdentifier { uber_group: 12, uber_id: 84 },
    ),
    (
        "multi_vars.85_multi",
        UberIdentifier { uber_group: 12, uber_id: 85 },
    ),
    (
        "multi_vars.86_multi",
        UberIdentifier { uber_group: 12, uber_id: 86 },
    ),
    (
        "multi_vars.87_multi",
        UberIdentifier { uber_group: 12, uber_id: 87 },
    ),
    (
        "multi_vars.88_multi",
        UberIdentifier { uber_group: 12, uber_id: 88 },
    ),
    (
        "multi_vars.89_multi",
        UberIdentifier { uber_group: 12, uber_id: 89 },
    ),
    (
        "multi_vars.90_multi",
        UberIdentifier { uber_group: 12, uber_id: 90 },
    ),
    (
        "multi_vars.91_multi",
        UberIdentifier { uber_group: 12, uber_id: 91 },
    ),
    (
        "multi_vars.92_multi",
        UberIdentifier { uber_group: 12, uber_id: 92 },
    ),
    (
        "multi_vars.93_multi",
        UberIdentifier { uber_group: 12, uber_id: 93 },
    ),
    (
        "multi_vars.94_multi",
        UberIdentifier { uber_group: 12, uber_id: 94 },
    ),
    (
        "multi_vars.95_multi",
        UberIdentifier { uber_group: 12, uber_id: 95 },
    ),
    (
        "multi_vars.96_multi",
        UberIdentifier { uber_group: 12, uber_id: 96 },
    ),
    (
        "multi_vars.97_multi",
        UberIdentifier { uber_group: 12, uber_id: 97 },
    ),
    (
        "multi_vars.98_multi",
        UberIdentifier { uber_group: 12, uber_id: 98 },
    ),
    (
        "multi_vars.99_multi",
        UberIdentifier { uber_group: 12, uber_id: 99 },
    ),
    (
        "rando_stats.Deaths",
        UberIdentifier { uber_group: 14, uber_id: 101 },
    ),
    (
        "rando_stats.warps used",
        UberIdentifier { uber_group: 14, uber_id: 106 },
    ),
    (
        "rando_stats.Peak PPM count",
        UberIdentifier { uber_group: 14, uber_id: 108 },
    ),
    (
        "rando_stats.Marsh Time",
        UberIdentifier { uber_group: 14, uber_id: 0 },
    ),
    (
        "rando_stats.Hollow Time",
        UberIdentifier { uber_group: 14, uber_id: 1 },
    ),
    (
        "rando_stats.Glades Time",
        UberIdentifier { uber_group: 14, uber_id: 2 },
    ),
    (
        "rando_stats.Wellspring Time",
        UberIdentifier { uber_group: 14, uber_id: 3 },
    ),
    (
        "rando_stats.Burrows Time",
        UberIdentifier { uber_group: 14, uber_id: 4 },
    ),
    (
        "rando_stats.Woods Time",
        UberIdentifier { uber_group: 14, uber_id: 5 },
    ),
    (
        "rando_stats.Reach Time",
        UberIdentifier { uber_group: 14, uber_id: 6 },
    ),
    (
        "rando_stats.Pools Time",
        UberIdentifier { uber_group: 14, uber_id: 7 },
    ),
    (
        "rando_stats.Depths Time",
        UberIdentifier { uber_group: 14, uber_id: 8 },
    ),
    (
        "rando_stats.Wastes Time",
        UberIdentifier { uber_group: 14, uber_id: 9 },
    ),
    (
        "rando_stats.Ruins Time",
        UberIdentifier { uber_group: 14, uber_id: 10 },
    ),
    (
        "rando_stats.Willow Time",
        UberIdentifier { uber_group: 14, uber_id: 11 },
    ),
    (
        "rando_stats.Void Time",
        UberIdentifier { uber_group: 14, uber_id: 12 },
    ),
    (
        "rando_stats.Time",
        UberIdentifier { uber_group: 14, uber_id: 100 },
    ),
    (
        "rando_stats.Current Drought",
        UberIdentifier { uber_group: 14, uber_id: 102 },
    ),
    (
        "rando_stats.Longest Drought",
        UberIdentifier { uber_group: 14, uber_id: 103 },
    ),
    (
        "rando_stats.Time since last checkpoint",
        UberIdentifier { uber_group: 14, uber_id: 104 },
    ),
    (
        "rando_stats.Time lost to deaths",
        UberIdentifier { uber_group: 14, uber_id: 105 },
    ),
    (
        "rando_stats.Peak PPM time",
        UberIdentifier { uber_group: 14, uber_id: 107 },
    ),
    (
        "rando_stats.Marsh Deaths",
        UberIdentifier { uber_group: 14, uber_id: 20 },
    ),
    (
        "rando_stats.Hollow Deaths",
        UberIdentifier { uber_group: 14, uber_id: 21 },
    ),
    (
        "rando_stats.Glades Deaths",
        UberIdentifier { uber_group: 14, uber_id: 22 },
    ),
    (
        "rando_stats.Wellspring Deaths",
        UberIdentifier { uber_group: 14, uber_id: 23 },
    ),
    (
        "rando_stats.Burrows Deaths",
        UberIdentifier { uber_group: 14, uber_id: 24 },
    ),
    (
        "rando_stats.Woods Deaths",
        UberIdentifier { uber_group: 14, uber_id: 25 },
    ),
    (
        "rando_stats.Reach Deaths",
        UberIdentifier { uber_group: 14, uber_id: 26 },
    ),
    (
        "rando_stats.Pools Deaths",
        UberIdentifier { uber_group: 14, uber_id: 27 },
    ),
    (
        "rando_stats.Depths Deaths",
        UberIdentifier { uber_group: 14, uber_id: 28 },
    ),
    (
        "rando_stats.Wastes Deaths",
        UberIdentifier { uber_group: 14, uber_id: 29 },
    ),
    (
        "rando_stats.Ruins Deaths",
        UberIdentifier { uber_group: 14, uber_id: 30 },
    ),
    (
        "rando_stats.Willow Deaths",
        UberIdentifier { uber_group: 14, uber_id: 31 },
    ),
    (
        "rando_stats.Void Deaths",
        UberIdentifier { uber_group: 14, uber_id: 32 },
    ),
    (
        "rando_stats.Marsh Pickups",
        UberIdentifier { uber_group: 14, uber_id: 40 },
    ),
    (
        "rando_stats.Hollow Pickups",
        UberIdentifier { uber_group: 14, uber_id: 41 },
    ),
    (
        "rando_stats.Glades Pickups",
        UberIdentifier { uber_group: 14, uber_id: 42 },
    ),
    (
        "rando_stats.Wellspring Pickups",
        UberIdentifier { uber_group: 14, uber_id: 43 },
    ),
    (
        "rando_stats.Burrows Pickups",
        UberIdentifier { uber_group: 14, uber_id: 44 },
    ),
    (
        "rando_stats.Woods Pickups",
        UberIdentifier { uber_group: 14, uber_id: 45 },
    ),
    (
        "rando_stats.Reach Pickups",
        UberIdentifier { uber_group: 14, uber_id: 46 },
    ),
    (
        "rando_stats.Pools Pickups",
        UberIdentifier { uber_group: 14, uber_id: 47 },
    ),
    (
        "rando_stats.Depths Pickups",
        UberIdentifier { uber_group: 14, uber_id: 48 },
    ),
    (
        "rando_stats.Wastes Pickups",
        UberIdentifier { uber_group: 14, uber_id: 49 },
    ),
    (
        "rando_stats.Ruins Pickups",
        UberIdentifier { uber_group: 14, uber_id: 50 },
    ),
    (
        "rando_stats.Willow Pickups",
        UberIdentifier { uber_group: 14, uber_id: 51 },
    ),
    (
        "rando_stats.Void Pickups",
        UberIdentifier { uber_group: 14, uber_id: 52 },
    ),
    (
        "animalCutsceneGroupDescriptor.animalCutsceneDoneUberState",
        UberIdentifier { uber_group: 192, uber_id: 80 },
    ),
    (
        "kwoloksGroupDescriptor.leafPileB",
        UberIdentifier { uber_group: 195, uber_id: 56127 },
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorBOpen",
        UberIdentifier { uber_group: 229, uber_id: 2 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalOBurning",
        UberIdentifier { uber_group: 229, uber_id: 27 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalFBurning",
        UberIdentifier { uber_group: 229, uber_id: 30 },
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorAOpen",
        UberIdentifier { uber_group: 229, uber_id: 35 },
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorFOpen",
        UberIdentifier { uber_group: 229, uber_id: 49 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalEBurning",
        UberIdentifier { uber_group: 229, uber_id: 52 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalABurning",
        UberIdentifier { uber_group: 229, uber_id: 62 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalIBurning",
        UberIdentifier { uber_group: 229, uber_id: 66 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalNBurning",
        UberIdentifier { uber_group: 229, uber_id: 71 },
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorEOpen",
        UberIdentifier { uber_group: 229, uber_id: 76 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalGBurning",
        UberIdentifier { uber_group: 229, uber_id: 108 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalMBurning",
        UberIdentifier { uber_group: 229, uber_id: 119 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalCBurning",
        UberIdentifier { uber_group: 229, uber_id: 129 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalKBurning",
        UberIdentifier { uber_group: 229, uber_id: 174 },
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorCOpen",
        UberIdentifier { uber_group: 229, uber_id: 185 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalHBurning",
        UberIdentifier { uber_group: 229, uber_id: 210 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalJBurning",
        UberIdentifier { uber_group: 229, uber_id: 222 },
    ),
    (
        "_riverlandsGroup.riverlands_blueFlameDoorDOpen",
        UberIdentifier { uber_group: 229, uber_id: 226 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalDBurning",
        UberIdentifier { uber_group: 229, uber_id: 231 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalBBurning",
        UberIdentifier { uber_group: 229, uber_id: 233 },
    ),
    (
        "_riverlandsGroup.riverlands_pedestalLBurning",
        UberIdentifier { uber_group: 229, uber_id: 237 },
    ),
    (
        "_riverlandsGroup.savePedestalUberState",
        UberIdentifier { uber_group: 229, uber_id: 41675 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 937, uber_id: 109 },
    ),
    (
        "kwolokGroupDescriptor.watermillDoor",
        UberIdentifier { uber_group: 937, uber_id: 749 },
    ),
    (
        "kwolokGroupDescriptor.cavernGLeverAndDoor",
        UberIdentifier { uber_group: 937, uber_id: 1174 },
    ),
    (
        "kwolokGroupDescriptor.halfHealthCell",
        UberIdentifier { uber_group: 937, uber_id: 2463 },
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier { uber_group: 937, uber_id: 2538 },
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop04",
        UberIdentifier { uber_group: 937, uber_id: 4057 },
    ),
    (
        "kwolokGroupDescriptor.xpOrbB",
        UberIdentifier { uber_group: 937, uber_id: 5568 },
    ),
    (
        "kwolokGroupDescriptor.energyContainerA",
        UberIdentifier { uber_group: 937, uber_id: 5668 },
    ),
    (
        "kwolokGroupDescriptor.orePickup",
        UberIdentifier { uber_group: 937, uber_id: 6703 },
    ),
    (
        "kwolokGroupDescriptor.mokiGateOpened",
        UberIdentifier { uber_group: 937, uber_id: 6778 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokF",
        UberIdentifier { uber_group: 937, uber_id: 7119 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 937, uber_id: 7153 },
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop01",
        UberIdentifier { uber_group: 937, uber_id: 7941 },
    ),
    (
        "kwolokGroupDescriptor.energyHalfCell",
        UberIdentifier { uber_group: 937, uber_id: 8518 },
    ),
    (
        "kwolokGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 937, uber_id: 10140 },
    ),
    (
        "kwolokGroupDescriptor.gromOreA",
        UberIdentifier { uber_group: 937, uber_id: 10729 },
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier { uber_group: 937, uber_id: 10877 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 937, uber_id: 11430 },
    ),
    (
        "kwolokGroupDescriptor.bombableWallAkwoloksCavernEa",
        UberIdentifier { uber_group: 937, uber_id: 11610 },
    ),
    (
        "kwolokGroupDescriptor.orePickupB",
        UberIdentifier { uber_group: 937, uber_id: 11846 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokH",
        UberIdentifier { uber_group: 937, uber_id: 12458 },
    ),
    (
        "kwolokGroupDescriptor.stepsRisen",
        UberIdentifier { uber_group: 937, uber_id: 13273 },
    ),
    (
        "kwolokGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 937, uber_id: 13413 },
    ),
    (
        "kwolokGroupDescriptor.smallExpA",
        UberIdentifier { uber_group: 937, uber_id: 15993 },
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier { uber_group: 937, uber_id: 16163 },
    ),
    (
        "kwolokGroupDescriptor.energyContainerA",
        UberIdentifier { uber_group: 937, uber_id: 17761 },
    ),
    (
        "kwolokGroupDescriptor.temp_WispQuestStandIn",
        UberIdentifier { uber_group: 937, uber_id: 18035 },
    ),
    (
        "kwolokGroupDescriptor.smallExpA",
        UberIdentifier { uber_group: 937, uber_id: 18103 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpC",
        UberIdentifier { uber_group: 937, uber_id: 19529 },
    ),
    (
        "kwolokGroupDescriptor.smallExpB",
        UberIdentifier { uber_group: 937, uber_id: 20219 },
    ),
    (
        "kwolokGroupDescriptor.bombableWallAkwoloksCavernE",
        UberIdentifier { uber_group: 937, uber_id: 20294 },
    ),
    (
        "kwolokGroupDescriptor.kwolokCavernsGate",
        UberIdentifier { uber_group: 937, uber_id: 21165 },
    ),
    (
        "kwolokGroupDescriptor.kwolokShrineBreakableWall",
        UberIdentifier { uber_group: 937, uber_id: 22302 },
    ),
    (
        "kwolokGroupDescriptor.leverDoor",
        UberIdentifier { uber_group: 937, uber_id: 22419 },
    ),
    (
        "kwolokGroupDescriptor.pressurePlatePuzzle",
        UberIdentifier { uber_group: 937, uber_id: 22716 },
    ),
    (
        "kwolokGroupDescriptor.lifeCellA",
        UberIdentifier { uber_group: 937, uber_id: 23486 },
    ),
    (
        "kwolokGroupDescriptor.energyHalfCell",
        UberIdentifier { uber_group: 937, uber_id: 23772 },
    ),
    (
        "kwolokGroupDescriptor.spiritShardA",
        UberIdentifier { uber_group: 937, uber_id: 24039 },
    ),
    (
        "kwolokGroupDescriptor.energyHalfContainer",
        UberIdentifier { uber_group: 937, uber_id: 24175 },
    ),
    (
        "kwolokGroupDescriptor.spiritShardMagnet",
        UberIdentifier { uber_group: 937, uber_id: 25413 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokC",
        UberIdentifier { uber_group: 937, uber_id: 27481 },
    ),
    (
        "kwolokGroupDescriptor.hornbugWallBroken",
        UberIdentifier { uber_group: 937, uber_id: 27671 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpB",
        UberIdentifier { uber_group: 937, uber_id: 30182 },
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop01",
        UberIdentifier { uber_group: 937, uber_id: 30594 },
    ),
    (
        "kwolokGroupDescriptor.breakableWallA",
        UberIdentifier { uber_group: 937, uber_id: 31026 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpC",
        UberIdentifier { uber_group: 937, uber_id: 31036 },
    ),
    (
        "kwolokGroupDescriptor.switchTop",
        UberIdentifier { uber_group: 937, uber_id: 31222 },
    ),
    (
        "kwolokGroupDescriptor.door",
        UberIdentifier { uber_group: 937, uber_id: 32165 },
    ),
    (
        "kwolokGroupDescriptor.areaText",
        UberIdentifier { uber_group: 937, uber_id: 32175 },
    ),
    (
        "kwolokGroupDescriptor.drillZoneA",
        UberIdentifier { uber_group: 937, uber_id: 32452 },
    ),
    (
        "kwolokGroupDescriptor.keyStoneD",
        UberIdentifier { uber_group: 937, uber_id: 33763 },
    ),
    (
        "kwolokGroupDescriptor.winterForestDoor",
        UberIdentifier { uber_group: 937, uber_id: 33773 },
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop03",
        UberIdentifier { uber_group: 937, uber_id: 34340 },
    ),
    (
        "kwolokGroupDescriptor.airDashHint",
        UberIdentifier { uber_group: 937, uber_id: 34343 },
    ),
    (
        "kwolokGroupDescriptor.leverDoorKwoloksHollowEntrance",
        UberIdentifier { uber_group: 937, uber_id: 34396 },
    ),
    (
        "kwolokGroupDescriptor.haveSpokenToOtters",
        UberIdentifier { uber_group: 937, uber_id: 34516 },
    ),
    (
        "kwolokGroupDescriptor.frogTongueA",
        UberIdentifier { uber_group: 937, uber_id: 34849 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokI",
        UberIdentifier { uber_group: 937, uber_id: 37823 },
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholder",
        UberIdentifier { uber_group: 937, uber_id: 37926 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokD",
        UberIdentifier { uber_group: 937, uber_id: 39338 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokE",
        UberIdentifier { uber_group: 937, uber_id: 39661 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokG",
        UberIdentifier { uber_group: 937, uber_id: 39715 },
    ),
    (
        "kwolokGroupDescriptor.secretWallB",
        UberIdentifier { uber_group: 937, uber_id: 40042 },
    ),
    (
        "kwolokGroupDescriptor.stompableFloor",
        UberIdentifier { uber_group: 937, uber_id: 40225 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpD",
        UberIdentifier { uber_group: 937, uber_id: 40298 },
    ),
    (
        "kwolokGroupDescriptor.secretWallKwolok",
        UberIdentifier { uber_group: 937, uber_id: 40466 },
    ),
    (
        "kwolokGroupDescriptor.smallExpA",
        UberIdentifier { uber_group: 937, uber_id: 40657 },
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholderB",
        UberIdentifier { uber_group: 937, uber_id: 42333 },
    ),
    (
        "kwolokGroupDescriptor.interactedWithTokk",
        UberIdentifier { uber_group: 937, uber_id: 42585 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolok",
        UberIdentifier { uber_group: 937, uber_id: 44594 },
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop03",
        UberIdentifier { uber_group: 937, uber_id: 44861 },
    ),
    (
        "kwolokGroupDescriptor.stompableFloorB",
        UberIdentifier { uber_group: 937, uber_id: 45349 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 937, uber_id: 45625 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpB",
        UberIdentifier { uber_group: 937, uber_id: 45744 },
    ),
    (
        "kwolokGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 937, uber_id: 45811 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 937, uber_id: 45987 },
    ),
    (
        "kwolokGroupDescriptor.ravineToadTop02",
        UberIdentifier { uber_group: 937, uber_id: 47364 },
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholderA",
        UberIdentifier { uber_group: 937, uber_id: 48192 },
    ),
    (
        "kwolokGroupDescriptor.desertBombableWall",
        UberIdentifier { uber_group: 937, uber_id: 49545 },
    ),
    (
        "kwolokGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 937, uber_id: 50176 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokB",
        UberIdentifier { uber_group: 937, uber_id: 50357 },
    ),
    (
        "kwolokGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 937, uber_id: 50474 },
    ),
    (
        "kwolokGroupDescriptor.energyContainerPlaceholder",
        UberIdentifier { uber_group: 937, uber_id: 50615 },
    ),
    (
        "kwolokGroupDescriptor.breakableWallA",
        UberIdentifier { uber_group: 937, uber_id: 51878 },
    ),
    (
        "kwolokGroupDescriptor.kwolokBossBridgeBroken",
        UberIdentifier { uber_group: 937, uber_id: 51919 },
    ),
    (
        "kwolokGroupDescriptor.spiritShardPickupPlaceholder",
        UberIdentifier { uber_group: 937, uber_id: 52258 },
    ),
    (
        "kwolokGroupDescriptor.frogDoor",
        UberIdentifier { uber_group: 937, uber_id: 52652 },
    ),
    (
        "kwolokGroupDescriptor.hornBugBossDefeatedState",
        UberIdentifier { uber_group: 937, uber_id: 53122 },
    ),
    (
        "kwolokGroupDescriptor.bombableWallAkwoloksCavernEb",
        UberIdentifier { uber_group: 937, uber_id: 53969 },
    ),
    (
        "kwolokGroupDescriptor.keyStoneC",
        UberIdentifier { uber_group: 937, uber_id: 54102 },
    ),
    (
        "kwolokGroupDescriptor.brokenWallA",
        UberIdentifier { uber_group: 937, uber_id: 54236 },
    ),
    (
        "kwolokGroupDescriptor.risingPedestals",
        UberIdentifier { uber_group: 937, uber_id: 54318 },
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop02",
        UberIdentifier { uber_group: 937, uber_id: 55341 },
    ),
    (
        "kwolokGroupDescriptor.dashHint",
        UberIdentifier { uber_group: 937, uber_id: 55538 },
    ),
    (
        "kwolokGroupDescriptor.drillableWallKwolokJ",
        UberIdentifier { uber_group: 937, uber_id: 56352 },
    ),
    (
        "kwolokGroupDescriptor.frogTongueC",
        UberIdentifier { uber_group: 937, uber_id: 56795 },
    ),
    (
        "kwolokGroupDescriptor.leverDoorA",
        UberIdentifier { uber_group: 937, uber_id: 57028 },
    ),
    (
        "kwolokGroupDescriptor.healthHalfCell",
        UberIdentifier { uber_group: 937, uber_id: 58598 },
    ),
    (
        "kwolokGroupDescriptor.shootableTargetDoor",
        UberIdentifier { uber_group: 937, uber_id: 58747 },
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop05",
        UberIdentifier { uber_group: 937, uber_id: 59404 },
    ),
    (
        "kwolokGroupDescriptor.serializedBooleanUberState",
        UberIdentifier { uber_group: 937, uber_id: 59515 },
    ),
    (
        "kwolokGroupDescriptor.doorA",
        UberIdentifier { uber_group: 937, uber_id: 59850 },
    ),
    (
        "kwolokGroupDescriptor.switchDoorUberState",
        UberIdentifier { uber_group: 937, uber_id: 59920 },
    ),
    (
        "kwolokGroupDescriptor.ravineBottomTop04",
        UberIdentifier { uber_group: 937, uber_id: 61099 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 937, uber_id: 61460 },
    ),
    (
        "kwolokGroupDescriptor.hornbugIntroArenaUberState",
        UberIdentifier { uber_group: 937, uber_id: 61633 },
    ),
    (
        "kwolokGroupDescriptor.smallExpOrbPlaceholderC",
        UberIdentifier { uber_group: 937, uber_id: 61744 },
    ),
    (
        "kwolokGroupDescriptor.xpOrbC",
        UberIdentifier { uber_group: 937, uber_id: 61783 },
    ),
    (
        "kwolokGroupDescriptor.healthHalfCell",
        UberIdentifier { uber_group: 937, uber_id: 61897 },
    ),
    (
        "kwolokGroupDescriptor.entranceStatueOpened",
        UberIdentifier { uber_group: 937, uber_id: 64003 },
    ),
    (
        "kwolokGroupDescriptor.spiritShardPickupPlaceholderB",
        UberIdentifier { uber_group: 937, uber_id: 64146 },
    ),
    (
        "kwolokGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 937, uber_id: 65195 },
    ),
    (
        "kwolokGroupDescriptor.savePedestal",
        UberIdentifier { uber_group: 937, uber_id: 5281 },
    ),
    (
        "kwolokGroupDescriptor.savePedestal",
        UberIdentifier { uber_group: 937, uber_id: 26601 },
    ),
    (
        "kwolokGroupDescriptor.kwolokNpcState",
        UberIdentifier { uber_group: 937, uber_id: 10071 },
    ),
    (
        "kwolokGroupDescriptor.cleanseWellspringQuestUberState",
        UberIdentifier { uber_group: 937, uber_id: 34641 },
    ),
    (
        "kwolokGroupDescriptor.recedingWaterSetup",
        UberIdentifier { uber_group: 937, uber_id: 42245 },
    ),
    (
        "kwolokGroupDescriptor.shardTraderState",
        UberIdentifier { uber_group: 937, uber_id: 47836 },
    ),
    (
        "kwolokGroupDescriptor.hornBugBossState",
        UberIdentifier { uber_group: 937, uber_id: 48534 },
    ),
    (
        "kwolokGroupDescriptor.recedingWaterSetupJordi",
        UberIdentifier { uber_group: 937, uber_id: 52814 },
    ),
    (
        "kwolokGroupDescriptor.healthPlantTimer",
        UberIdentifier { uber_group: 937, uber_id: 14501 },
    ),
    (
        "kwolokGroupDescriptor.healthPlant",
        UberIdentifier { uber_group: 937, uber_id: 15130 },
    ),
    (
        "kwolokGroupDescriptor.eyesPlacedIntoStatue",
        UberIdentifier { uber_group: 937, uber_id: 1038 },
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomBottom",
        UberIdentifier { uber_group: 937, uber_id: 6040 },
    ),
    (
        "kwolokGroupDescriptor.frogTongueB",
        UberIdentifier { uber_group: 937, uber_id: 12557 },
    ),
    (
        "kwolokGroupDescriptor.retractTongue",
        UberIdentifier { uber_group: 937, uber_id: 13557 },
    ),
    (
        "kwolokGroupDescriptor.frogTongueE",
        UberIdentifier { uber_group: 937, uber_id: 14026 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernGPuzzleLeftUp",
        UberIdentifier { uber_group: 937, uber_id: 19495 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernBRight",
        UberIdentifier { uber_group: 937, uber_id: 24510 },
    ),
    (
        "kwolokGroupDescriptor.frogTop01",
        UberIdentifier { uber_group: 937, uber_id: 28504 },
    ),
    (
        "kwolokGroupDescriptor.attackableFrogByteUberState",
        UberIdentifier { uber_group: 937, uber_id: 30661 },
    ),
    (
        "kwolokGroupDescriptor.kwolokCavernsAttackableToad",
        UberIdentifier { uber_group: 937, uber_id: 32948 },
    ),
    (
        "kwolokGroupDescriptor.frogBottom03",
        UberIdentifier { uber_group: 937, uber_id: 37928 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernFBottom",
        UberIdentifier { uber_group: 937, uber_id: 38183 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernGPuzzleRight",
        UberIdentifier { uber_group: 937, uber_id: 40810 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernBLeft",
        UberIdentifier { uber_group: 937, uber_id: 41587 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernELeft",
        UberIdentifier { uber_group: 937, uber_id: 44452 },
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomTopC",
        UberIdentifier { uber_group: 937, uber_id: 45423 },
    ),
    (
        "kwolokGroupDescriptor.frogBottom02",
        UberIdentifier { uber_group: 937, uber_id: 49392 },
    ),
    (
        "kwolokGroupDescriptor.kwolokCavernsAttackableToadB",
        UberIdentifier { uber_group: 937, uber_id: 49874 },
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomTopA",
        UberIdentifier { uber_group: 937, uber_id: 50411 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernGPuzzleLeft",
        UberIdentifier { uber_group: 937, uber_id: 50803 },
    ),
    (
        "kwolokGroupDescriptor.frogTongueD",
        UberIdentifier { uber_group: 937, uber_id: 51234 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernERight",
        UberIdentifier { uber_group: 937, uber_id: 53749 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernGBottom",
        UberIdentifier { uber_group: 937, uber_id: 56395 },
    ),
    (
        "kwolokGroupDescriptor.frogTop02",
        UberIdentifier { uber_group: 937, uber_id: 57711 },
    ),
    (
        "kwolokGroupDescriptor.frogTop03",
        UberIdentifier { uber_group: 937, uber_id: 59288 },
    ),
    (
        "kwolokGroupDescriptor.frogBottom04",
        UberIdentifier { uber_group: 937, uber_id: 62300 },
    ),
    (
        "kwolokGroupDescriptor.frogUpperMainRoomTopB",
        UberIdentifier { uber_group: 937, uber_id: 63347 },
    ),
    (
        "kwolokGroupDescriptor.frogCavernBTopRight",
        UberIdentifier { uber_group: 937, uber_id: 63834 },
    ),
    (
        "kwolokGroupDescriptor.frogBottom01",
        UberIdentifier { uber_group: 937, uber_id: 64257 },
    ),
    (
        "lagoonStateGroup.secretWallA",
        UberIdentifier { uber_group: 945, uber_id: 3487 },
    ),
    (
        "lagoonStateGroup.canShowGlideHint",
        UberIdentifier { uber_group: 945, uber_id: 3659 },
    ),
    (
        "lagoonStateGroup.mediumExpA",
        UberIdentifier { uber_group: 945, uber_id: 7031 },
    ),
    (
        "lagoonStateGroup.breakableWallB",
        UberIdentifier { uber_group: 945, uber_id: 7465 },
    ),
    (
        "lagoonStateGroup.kwolokBossBridgeBreak",
        UberIdentifier { uber_group: 945, uber_id: 9034 },
    ),
    (
        "lagoonStateGroup.wispSequencePlayedOut",
        UberIdentifier { uber_group: 945, uber_id: 9367 },
    ),
    (
        "lagoonStateGroup.mediumExpB",
        UberIdentifier { uber_group: 945, uber_id: 10682 },
    ),
    (
        "lagoonStateGroup.largeExpC",
        UberIdentifier { uber_group: 945, uber_id: 10833 },
    ),
    (
        "lagoonStateGroup.tentacleKilled",
        UberIdentifier { uber_group: 945, uber_id: 12852 },
    ),
    (
        "lagoonStateGroup.mediumExpA",
        UberIdentifier { uber_group: 945, uber_id: 14530 },
    ),
    (
        "lagoonStateGroup.energyCellA",
        UberIdentifier { uber_group: 945, uber_id: 21334 },
    ),
    (
        "lagoonStateGroup.memoriesPlayed",
        UberIdentifier { uber_group: 945, uber_id: 25182 },
    ),
    (
        "lagoonStateGroup.energyContainerA",
        UberIdentifier { uber_group: 945, uber_id: 25520 },
    ),
    (
        "lagoonStateGroup.breakableWallA",
        UberIdentifier { uber_group: 945, uber_id: 28631 },
    ),
    (
        "lagoonStateGroup.mediumExpA",
        UberIdentifier { uber_group: 945, uber_id: 32890 },
    ),
    (
        "lagoonStateGroup.displayedGlideHint",
        UberIdentifier { uber_group: 945, uber_id: 33930 },
    ),
    (
        "lagoonStateGroup.lagoonMillTransitionHealthCell",
        UberIdentifier { uber_group: 945, uber_id: 37243 },
    ),
    (
        "lagoonStateGroup.mediumExpB",
        UberIdentifier { uber_group: 945, uber_id: 38319 },
    ),
    (
        "lagoonStateGroup.breakableWallB",
        UberIdentifier { uber_group: 945, uber_id: 39004 },
    ),
    (
        "lagoonStateGroup.secretWallB",
        UberIdentifier { uber_group: 945, uber_id: 43451 },
    ),
    (
        "lagoonStateGroup.bossReward",
        UberIdentifier { uber_group: 945, uber_id: 49747 },
    ),
    (
        "lagoonStateGroup.breakableWallA",
        UberIdentifier { uber_group: 945, uber_id: 55795 },
    ),
    (
        "lagoonStateGroup.medExpA",
        UberIdentifier { uber_group: 945, uber_id: 58723 },
    ),
    (
        "lagoonStateGroup.savePedestalUberState",
        UberIdentifier { uber_group: 945, uber_id: 1370 },
    ),
    (
        "lagoonStateGroup.savePedestalUberState",
        UberIdentifier { uber_group: 945, uber_id: 58183 },
    ),
    (
        "lagoonStateGroup.healthPlantA",
        UberIdentifier { uber_group: 945, uber_id: 23296 },
    ),
    (
        "lagoonStateGroup.kwolokBossState",
        UberIdentifier { uber_group: 945, uber_id: 58403 },
    ),
    (
        "playerUberStateGroupDescriptor.playerPurchasedWeaponMasterUpgrade",
        UberIdentifier { uber_group: 3440, uber_id: 20131 },
    ),
    (
        "playerUberStateGroupDescriptor.playerOnTandemUberState",
        UberIdentifier { uber_group: 3440, uber_id: 54402 },
    ),
    (
        "playerUberStateGroupDescriptor.playerWeaponDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 34448 },
    ),
    (
        "playerUberStateGroupDescriptor.hammerSpeedUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 1157 },
    ),
    (
        "playerUberStateGroupDescriptor.chargeWeaponsUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 2234 },
    ),
    (
        "playerUberStateGroupDescriptor.spikeExplosiveUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 5687 },
    ),
    (
        "playerUberStateGroupDescriptor.spellMeditateUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 9670 },
    ),
    (
        "playerUberStateGroupDescriptor.waterBreathUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 10233 },
    ),
    (
        "playerUberStateGroupDescriptor.chakramSpinUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 10776 },
    ),
    (
        "playerUberStateGroupDescriptor.bashSplitUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 10928 },
    ),
    (
        "playerUberStateGroupDescriptor.grenadeDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 16155 },
    ),
    (
        "playerUberStateGroupDescriptor.spellChakramUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 17265 },
    ),
    (
        "playerUberStateGroupDescriptor.missilesDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 18770 },
    ),
    (
        "playerUberStateGroupDescriptor.spellSpikeUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 24142 },
    ),
    (
        "playerUberStateGroupDescriptor.missilesAmountUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 26998 },
    ),
    (
        "playerUberStateGroupDescriptor.bowDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 29503 },
    ),
    (
        "playerUberStateGroupDescriptor.swordComboUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 30415 },
    ),
    (
        "playerUberStateGroupDescriptor.healEfficiencyUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 31259 },
    ),
    (
        "playerUberStateGroupDescriptor.spikeDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 33963 },
    ),
    (
        "playerUberStateGroupDescriptor.spellSentryUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 38929 },
    ),
    (
        "playerUberStateGroupDescriptor.swordDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 39658 },
    ),
    (
        "playerUberStateGroupDescriptor.chakramMagnetUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 40954 },
    ),
    (
        "playerUberStateGroupDescriptor.chakramDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 42913 },
    ),
    (
        "playerUberStateGroupDescriptor.invisibilityDurationUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 45208 },
    ),
    (
        "playerUberStateGroupDescriptor.hammerStompUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 46488 },
    ),
    (
        "playerUberStateGroupDescriptor.sentryAmountUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 48877 },
    ),
    (
        "playerUberStateGroupDescriptor.hammerDamageUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 53415 },
    ),
    (
        "playerUberStateGroupDescriptor.sentrySpeedUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 57376 },
    ),
    (
        "playerUberStateGroupDescriptor.spellBlazeUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 58703 },
    ),
    (
        "playerUberStateGroupDescriptor.blazeChargeUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 61898 },
    ),
    (
        "playerUberStateGroupDescriptor.chakramAmountUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 62563 },
    ),
    (
        "playerUberStateGroupDescriptor.spellHammerUpgradeLevel",
        UberIdentifier { uber_group: 3440, uber_id: 64152 },
    ),
    (
        "lumaPoolsStateGroup.largeExpOrbPlaceholderA",
        UberIdentifier { uber_group: 5377, uber_id: 628 },
    ),
    (
        "lumaPoolsStateGroup.energyCellFragmentA",
        UberIdentifier { uber_group: 5377, uber_id: 1600 },
    ),
    (
        "lumaPoolsStateGroup.waterRaised",
        UberIdentifier { uber_group: 5377, uber_id: 2286 },
    ),
    (
        "lumaPoolsStateGroup.pullWallLeft",
        UberIdentifier { uber_group: 5377, uber_id: 2518 },
    ),
    (
        "lumaPoolsStateGroup.breakableSecretWallA",
        UberIdentifier { uber_group: 5377, uber_id: 3831 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallB",
        UberIdentifier { uber_group: 5377, uber_id: 4463 },
    ),
    (
        "lumaPoolsStateGroup.leverAndDoor",
        UberIdentifier { uber_group: 5377, uber_id: 6398 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier { uber_group: 5377, uber_id: 6857 },
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholder",
        UberIdentifier { uber_group: 5377, uber_id: 7381 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 7540 },
    ),
    (
        "lumaPoolsStateGroup.breakRockDState",
        UberIdentifier { uber_group: 5377, uber_id: 8019 },
    ),
    (
        "lumaPoolsStateGroup.trunkState",
        UberIdentifier { uber_group: 5377, uber_id: 8294 },
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerBlocked",
        UberIdentifier { uber_group: 5377, uber_id: 8440 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier { uber_group: 5377, uber_id: 8451 },
    ),
    (
        "lumaPoolsStateGroup.expOrb",
        UberIdentifier { uber_group: 5377, uber_id: 8939 },
    ),
    (
        "lumaPoolsStateGroup.expOrbB",
        UberIdentifier { uber_group: 5377, uber_id: 9812 },
    ),
    (
        "lumaPoolsStateGroup.creepD",
        UberIdentifier { uber_group: 5377, uber_id: 10291 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallB",
        UberIdentifier { uber_group: 5377, uber_id: 10782 },
    ),
    (
        "lumaPoolsStateGroup.bombableWallA",
        UberIdentifier { uber_group: 5377, uber_id: 11049 },
    ),
    (
        "lumaPoolsStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 5377, uber_id: 12235 },
    ),
    (
        "lumaPoolsStateGroup.pressurePlateGate",
        UberIdentifier { uber_group: 5377, uber_id: 12826 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbD",
        UberIdentifier { uber_group: 5377, uber_id: 13832 },
    ),
    (
        "lumaPoolsStateGroup.leverAndDoor",
        UberIdentifier { uber_group: 5377, uber_id: 14488 },
    ),
    (
        "lumaPoolsStateGroup.spiritShard",
        UberIdentifier { uber_group: 5377, uber_id: 14664 },
    ),
    (
        "lumaPoolsStateGroup.drillableWall",
        UberIdentifier { uber_group: 5377, uber_id: 15383 },
    ),
    (
        "lumaPoolsStateGroup.dashDoor",
        UberIdentifier { uber_group: 5377, uber_id: 15402 },
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedA",
        UberIdentifier { uber_group: 5377, uber_id: 15754 },
    ),
    (
        "lumaPoolsStateGroup.keystoneB",
        UberIdentifier { uber_group: 5377, uber_id: 16426 },
    ),
    (
        "lumaPoolsStateGroup.breakRockFState",
        UberIdentifier { uber_group: 5377, uber_id: 16607 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbC",
        UberIdentifier { uber_group: 5377, uber_id: 17396 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 18345 },
    ),
    (
        "lumaPoolsStateGroup.areaText",
        UberIdentifier { uber_group: 5377, uber_id: 19132 },
    ),
    (
        "lumaPoolsStateGroup.pickupA",
        UberIdentifier { uber_group: 5377, uber_id: 19694 },
    ),
    (
        "lumaPoolsStateGroup.talkedToKwolok",
        UberIdentifier { uber_group: 5377, uber_id: 21700 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 21860 },
    ),
    (
        "lumaPoolsStateGroup.breakRockEState",
        UberIdentifier { uber_group: 5377, uber_id: 22047 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallB",
        UberIdentifier { uber_group: 5377, uber_id: 22978 },
    ),
    (
        "lumaPoolsStateGroup.hintZones",
        UberIdentifier { uber_group: 5377, uber_id: 24015 },
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedA",
        UberIdentifier { uber_group: 5377, uber_id: 24765 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 25391 },
    ),
    (
        "lumaPoolsStateGroup.bridgeState",
        UberIdentifier { uber_group: 5377, uber_id: 25612 },
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderB",
        UberIdentifier { uber_group: 5377, uber_id: 25633 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier { uber_group: 5377, uber_id: 26170 },
    ),
    (
        "lumaPoolsStateGroup.lagoonDoor",
        UberIdentifier { uber_group: 5377, uber_id: 26987 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 27204 },
    ),
    (
        "lumaPoolsStateGroup.bombableWall",
        UberIdentifier { uber_group: 5377, uber_id: 27558 },
    ),
    (
        "lumaPoolsStateGroup.breakRockCState",
        UberIdentifier { uber_group: 5377, uber_id: 29662 },
    ),
    (
        "lumaPoolsStateGroup.loweringWaterState",
        UberIdentifier { uber_group: 5377, uber_id: 29911 },
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderB",
        UberIdentifier { uber_group: 5377, uber_id: 30860 },
    ),
    (
        "lumaPoolsStateGroup.treeFallen",
        UberIdentifier { uber_group: 5377, uber_id: 31093 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier { uber_group: 5377, uber_id: 31145 },
    ),
    (
        "lumaPoolsStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 5377, uber_id: 31434 },
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerBlocked",
        UberIdentifier { uber_group: 5377, uber_id: 32210 },
    ),
    (
        "lumaPoolsStateGroup.creepB",
        UberIdentifier { uber_group: 5377, uber_id: 32685 },
    ),
    (
        "lumaPoolsStateGroup.energyContainerA",
        UberIdentifier { uber_group: 5377, uber_id: 32750 },
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderC",
        UberIdentifier { uber_group: 5377, uber_id: 33110 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 33180 },
    ),
    (
        "lumaPoolsStateGroup.breakRockAState",
        UberIdentifier { uber_group: 5377, uber_id: 33730 },
    ),
    (
        "lumaPoolsStateGroup.orePickupA",
        UberIdentifier { uber_group: 5377, uber_id: 34852 },
    ),
    (
        "lumaPoolsStateGroup.playedMokiVignette",
        UberIdentifier { uber_group: 5377, uber_id: 35023 },
    ),
    (
        "lumaPoolsStateGroup.keystoneA",
        UberIdentifier { uber_group: 5377, uber_id: 35091 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbB",
        UberIdentifier { uber_group: 5377, uber_id: 35440 },
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedB",
        UberIdentifier { uber_group: 5377, uber_id: 35751 },
    ),
    (
        "lumaPoolsStateGroup.expOrb",
        UberIdentifier { uber_group: 5377, uber_id: 35971 },
    ),
    (
        "lumaPoolsStateGroup.breakableFloorA",
        UberIdentifier { uber_group: 5377, uber_id: 36511 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 38515 },
    ),
    (
        "lumaPoolsStateGroup.spiritShard",
        UberIdentifier { uber_group: 5377, uber_id: 40328 },
    ),
    (
        "lumaPoolsStateGroup.keystoneD",
        UberIdentifier { uber_group: 5377, uber_id: 41881 },
    ),
    (
        "lumaPoolsStateGroup.mainPickup",
        UberIdentifier { uber_group: 5377, uber_id: 42145 },
    ),
    (
        "lumaPoolsStateGroup.mediumExpOrbPlaceholderA",
        UberIdentifier { uber_group: 5377, uber_id: 42553 },
    ),
    (
        "lumaPoolsStateGroup.creepA",
        UberIdentifier { uber_group: 5377, uber_id: 43134 },
    ),
    (
        "lumaPoolsStateGroup.optionalPickup",
        UberIdentifier { uber_group: 5377, uber_id: 43859 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 44122 },
    ),
    (
        "lumaPoolsStateGroup.expOrbA",
        UberIdentifier { uber_group: 5377, uber_id: 44777 },
    ),
    (
        "lumaPoolsStateGroup.switchesActivated",
        UberIdentifier { uber_group: 5377, uber_id: 45765 },
    ),
    (
        "lumaPoolsStateGroup.healthContainerA",
        UberIdentifier { uber_group: 5377, uber_id: 45774 },
    ),
    (
        "lumaPoolsStateGroup.fallingRockState",
        UberIdentifier { uber_group: 5377, uber_id: 46040 },
    ),
    (
        "lumaPoolsStateGroup.keystoneC",
        UberIdentifier { uber_group: 5377, uber_id: 46926 },
    ),
    (
        "lumaPoolsStateGroup.keystoneGate",
        UberIdentifier { uber_group: 5377, uber_id: 47621 },
    ),
    (
        "lumaPoolsStateGroup.splitPlatformState",
        UberIdentifier { uber_group: 5377, uber_id: 49394 },
    ),
    (
        "lumaPoolsStateGroup.pullWallRight",
        UberIdentifier { uber_group: 5377, uber_id: 49826 },
    ),
    (
        "lumaPoolsStateGroup.doorState",
        UberIdentifier { uber_group: 5377, uber_id: 52062 },
    ),
    (
        "lumaPoolsStateGroup.breakableLogA",
        UberIdentifier { uber_group: 5377, uber_id: 52133 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbB",
        UberIdentifier { uber_group: 5377, uber_id: 52791 },
    ),
    (
        "lumaPoolsStateGroup.bombableWallB",
        UberIdentifier { uber_group: 5377, uber_id: 53532 },
    ),
    (
        "lumaPoolsStateGroup.spiritShard",
        UberIdentifier { uber_group: 5377, uber_id: 56199 },
    ),
    (
        "lumaPoolsStateGroup.secretWallA",
        UberIdentifier { uber_group: 5377, uber_id: 56302 },
    ),
    (
        "lumaPoolsStateGroup.creepE",
        UberIdentifier { uber_group: 5377, uber_id: 57334 },
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblocked",
        UberIdentifier { uber_group: 5377, uber_id: 57453 },
    ),
    (
        "lumaPoolsStateGroup.kwolokChaseDoorState",
        UberIdentifier { uber_group: 5377, uber_id: 57929 },
    ),
    (
        "lumaPoolsStateGroup.bubbleMakerUnblockedB",
        UberIdentifier { uber_group: 5377, uber_id: 58278 },
    ),
    (
        "lumaPoolsStateGroup.doorState",
        UberIdentifier { uber_group: 5377, uber_id: 59514 },
    ),
    (
        "lumaPoolsStateGroup.pickupA",
        UberIdentifier { uber_group: 5377, uber_id: 61475 },
    ),
    (
        "lumaPoolsStateGroup.xpOrbB",
        UberIdentifier { uber_group: 5377, uber_id: 62180 },
    ),
    (
        "lumaPoolsStateGroup.waterLowered",
        UberIdentifier { uber_group: 5377, uber_id: 63173 },
    ),
    (
        "lumaPoolsStateGroup.healthContainerA",
        UberIdentifier { uber_group: 5377, uber_id: 63201 },
    ),
    (
        "lumaPoolsStateGroup.doorState",
        UberIdentifier { uber_group: 5377, uber_id: 63513 },
    ),
    (
        "lumaPoolsStateGroup.secretWallA",
        UberIdentifier { uber_group: 5377, uber_id: 63922 },
    ),
    (
        "lumaPoolsStateGroup.breakableWall",
        UberIdentifier { uber_group: 5377, uber_id: 64337 },
    ),
    (
        "lumaPoolsStateGroup.creepC",
        UberIdentifier { uber_group: 5377, uber_id: 64761 },
    ),
    (
        "lumaPoolsStateGroup.breakRockBState",
        UberIdentifier { uber_group: 5377, uber_id: 64827 },
    ),
    (
        "lumaPoolsStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 5377, uber_id: 65019 },
    ),
    (
        "lumaPoolsStateGroup.breakableWallA",
        UberIdentifier { uber_group: 5377, uber_id: 65413 },
    ),
    (
        "lumaPoolsStateGroup.healthPlantA",
        UberIdentifier { uber_group: 5377, uber_id: 47557 },
    ),
    (
        "lumaPoolsStateGroup.healthPlantA",
        UberIdentifier { uber_group: 5377, uber_id: 63230 },
    ),
    (
        "lumaPoolsStateGroup.arenaByteStateSerialized",
        UberIdentifier { uber_group: 5377, uber_id: 1373 },
    ),
    (
        "lumaPoolsStateGroup.arenaBByteStateSerialized",
        UberIdentifier { uber_group: 5377, uber_id: 53480 },
    ),
    (
        "testUberStateGroup.firePedestalBooleanUberState",
        UberIdentifier { uber_group: 6837, uber_id: 5475 },
    ),
    (
        "testUberStateGroup.kwolokCavernDoor2",
        UberIdentifier { uber_group: 6837, uber_id: 7403 },
    ),
    (
        "testUberStateGroup.desertShortcutWall",
        UberIdentifier { uber_group: 6837, uber_id: 10235 },
    ),
    (
        "testUberStateGroup.testDoorTwoSlotsBooleanUberState",
        UberIdentifier { uber_group: 6837, uber_id: 19173 },
    ),
    (
        "testUberStateGroup.testShrineUberStateDescriptor",
        UberIdentifier { uber_group: 6837, uber_id: 19701 },
    ),
    (
        "testUberStateGroup.arenaCompletedState",
        UberIdentifier { uber_group: 6837, uber_id: 31278 },
    ),
    (
        "testUberStateGroup.lianaHealLantern",
        UberIdentifier { uber_group: 6837, uber_id: 31353 },
    ),
    (
        "testUberStateGroup.willowsEndShortcutWall",
        UberIdentifier { uber_group: 6837, uber_id: 38771 },
    ),
    (
        "testUberStateGroup.swampShortcutWall",
        UberIdentifier { uber_group: 6837, uber_id: 40492 },
    ),
    (
        "testUberStateGroup.winterForestEnemyDoor",
        UberIdentifier { uber_group: 6837, uber_id: 44762 },
    ),
    (
        "testUberStateGroup.lagoonContactSwitch",
        UberIdentifier { uber_group: 6837, uber_id: 47735 },
    ),
    (
        "testUberStateGroup.watermillShortcutWall",
        UberIdentifier { uber_group: 6837, uber_id: 51086 },
    ),
    (
        "testUberStateGroup.kwolokCavernDoor",
        UberIdentifier { uber_group: 6837, uber_id: 54316 },
    ),
    (
        "testUberStateGroup.testLeverDescriptorDesertC",
        UberIdentifier { uber_group: 6837, uber_id: 54999 },
    ),
    (
        "testUberStateGroup.oneSideBreakableWall",
        UberIdentifier { uber_group: 6837, uber_id: 55663 },
    ),
    (
        "testUberStateGroup.testSecret",
        UberIdentifier { uber_group: 6837, uber_id: 60688 },
    ),
    (
        "testUberStateGroup.testBooleanUberStateDescriptor",
        UberIdentifier { uber_group: 6837, uber_id: 60823 },
    ),
    (
        "testUberStateGroup.cordycepsShortcutWall",
        UberIdentifier { uber_group: 6837, uber_id: 61703 },
    ),
    (
        "testUberStateGroup.kwolokCavernsPressurePlate",
        UberIdentifier { uber_group: 6837, uber_id: 62194 },
    ),
    (
        "testUberStateGroup.kwolokCavernsAttackableSwitch",
        UberIdentifier { uber_group: 6837, uber_id: 62909 },
    ),
    (
        "testUberStateGroup.lagoonShortcutWall",
        UberIdentifier { uber_group: 6837, uber_id: 64646 },
    ),
    (
        "testUberStateGroup.testBreakableWallInt",
        UberIdentifier { uber_group: 6837, uber_id: 37967 },
    ),
    (
        "testUberStateGroup.testBreakableWallIntB",
        UberIdentifier { uber_group: 6837, uber_id: 61358 },
    ),
    (
        "testUberStateGroup.serializedInt",
        UberIdentifier { uber_group: 6837, uber_id: 63967 },
    ),
    (
        "testUberStateGroup.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 6837, uber_id: 39815 },
    ),
    (
        "testUberStateGroup.testSerializedFloatUberState",
        UberIdentifier { uber_group: 6837, uber_id: 61561 },
    ),
    (
        "desertAGroup.collectableHDesertA",
        UberIdentifier { uber_group: 7228, uber_id: 1781 },
    ),
    (
        "desertAGroup.collectableEDesertA",
        UberIdentifier { uber_group: 7228, uber_id: 2996 },
    ),
    (
        "desertAGroup.secretWall",
        UberIdentifier { uber_group: 7228, uber_id: 4034 },
    ),
    (
        "desertAGroup.gorlekOre",
        UberIdentifier { uber_group: 7228, uber_id: 8370 },
    ),
    (
        "desertAGroup.keystoneAUberState",
        UberIdentifier { uber_group: 7228, uber_id: 20282 },
    ),
    (
        "desertAGroup.collectableFDesertA",
        UberIdentifier { uber_group: 7228, uber_id: 32434 },
    ),
    (
        "desertAGroup.expOrb",
        UberIdentifier { uber_group: 7228, uber_id: 35329 },
    ),
    (
        "desertAGroup.collectableDesertA",
        UberIdentifier { uber_group: 7228, uber_id: 36579 },
    ),
    (
        "desertAGroup.lifeCellBooleanUberState",
        UberIdentifier { uber_group: 7228, uber_id: 37885 },
    ),
    (
        "desertAGroup.xpOrbUberState",
        UberIdentifier { uber_group: 7228, uber_id: 45954 },
    ),
    (
        "desertAGroup.xpOrbBUberState",
        UberIdentifier { uber_group: 7228, uber_id: 48993 },
    ),
    (
        "desertAGroup.collectableCDesertA",
        UberIdentifier { uber_group: 7228, uber_id: 52086 },
    ),
    (
        "desertAGroup.xpOrbB",
        UberIdentifier { uber_group: 7228, uber_id: 54275 },
    ),
    (
        "desertAGroup.gorlekOre",
        UberIdentifier { uber_group: 7228, uber_id: 54494 },
    ),
    (
        "desertAGroup.collectableADesertA",
        UberIdentifier { uber_group: 7228, uber_id: 56821 },
    ),
    (
        "desertAGroup.collectableGDesertA",
        UberIdentifier { uber_group: 7228, uber_id: 60605 },
    ),
    (
        "desertAGroup.xpOrbAUberState",
        UberIdentifier { uber_group: 7228, uber_id: 61548 },
    ),
    (
        "desertAGroup.keystoneBUberState",
        UberIdentifier { uber_group: 7228, uber_id: 62117 },
    ),
    (
        "statsUberStateGroup.totalSpiritLightCollectedSerializedIntUberState",
        UberIdentifier { uber_group: 8246, uber_id: 5144 },
    ),
    (
        "statsUberStateGroup.fastTravelCountIntUberState",
        UberIdentifier { uber_group: 8246, uber_id: 7909 },
    ),
    (
        "statsUberStateGroup.enemiesPiercedAtOnceStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 7927 },
    ),
    (
        "statsUberStateGroup.deathFromEnemiesStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 12323 },
    ),
    (
        "statsUberStateGroup.npcsInHubStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 15506 },
    ),
    (
        "statsUberStateGroup.bashesStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 17772 },
    ),
    (
        "statsUberStateGroup.shardSlotUpgradesCollectedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 18554 },
    ),
    (
        "statsUberStateGroup.mostDefeatedEnemyEnumStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 26498 },
    ),
    (
        "statsUberStateGroup.totalDamageTakenStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 28073 },
    ),
    (
        "statsUberStateGroup.wallJumpsStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 30164 },
    ),
    (
        "statsUberStateGroup.spiritLightCollectedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 30251 },
    ),
    (
        "statsUberStateGroup.sideQuestsCompletedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 31056 },
    ),
    (
        "statsUberStateGroup.enemyVsEnemyKillsStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 31216 },
    ),
    (
        "statsUberStateGroup.enemiesDefeatedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 32860 },
    ),
    (
        "statsUberStateGroup.deathsStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 36466 },
    ),
    (
        "statsUberStateGroup.totalSpiritLightSpentSerializedIntUberState",
        UberIdentifier { uber_group: 8246, uber_id: 37583 },
    ),
    (
        "statsUberStateGroup.highestAmountOfDamageSerializedIntUberState",
        UberIdentifier { uber_group: 8246, uber_id: 40254 },
    ),
    (
        "statsUberStateGroup.totalHealthRegeneratedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 42772 },
    ),
    (
        "statsUberStateGroup.gardenerSeedsCollectedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 44318 },
    ),
    (
        "statsUberStateGroup.racesCompletedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 49162 },
    ),
    (
        "statsUberStateGroup.favoriteSkillEnumStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 49721 },
    ),
    (
        "statsUberStateGroup.shrinesDiscoveredStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 50096 },
    ),
    (
        "statsUberStateGroup.spiritLightSpentStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 50669 },
    ),
    (
        "statsUberStateGroup.dashesStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 50952 },
    ),
    (
        "statsUberStateGroup.racePedestalsActivatedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 54110 },
    ),
    (
        "statsUberStateGroup.deathsEnvironmentalStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 57639 },
    ),
    (
        "statsUberStateGroup.drowningDeathsStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 58048 },
    ),
    (
        "statsUberStateGroup.jumpsStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 58908 },
    ),
    (
        "statsUberStateGroup.shardsCollectedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 59865 },
    ),
    (
        "statsUberStateGroup.spiritWellsDiscoveredStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 60852 },
    ),
    (
        "statsUberStateGroup.mostDefeatedByEnemyEnumStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 62287 },
    ),
    (
        "statsUberStateGroup.shrinesCompletedStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 63037 },
    ),
    (
        "statsUberStateGroup.leashesStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 64519 },
    ),
    (
        "statsUberStateGroup.teleportCountStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 64778 },
    ),
    (
        "statsUberStateGroup.timeAirborneStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 3307 },
    ),
    (
        "statsUberStateGroup.timeGlowingStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 7293 },
    ),
    (
        "statsUberStateGroup.distanceSwamStatSettingSerializedFloatUberState",
        UberIdentifier { uber_group: 8246, uber_id: 8682 },
    ),
    (
        "statsUberStateGroup.distanceGlidedStatSettingSerializedFloatUberState",
        UberIdentifier { uber_group: 8246, uber_id: 16123 },
    ),
    (
        "statsUberStateGroup.distanceBurrowedStatSettingSerializedFloatUberState",
        UberIdentifier { uber_group: 8246, uber_id: 40261 },
    ),
    (
        "statsUberStateGroup.timeTotalPlaytimeStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 43418 },
    ),
    (
        "statsUberStateGroup.distanceFallingStatSettingSerializedFloatUberState",
        UberIdentifier { uber_group: 8246, uber_id: 44439 },
    ),
    (
        "statsUberStateGroup.timeAliveUberState",
        UberIdentifier { uber_group: 8246, uber_id: 47477 },
    ),
    (
        "statsUberStateGroup.timeLongestSingleAirborneStatSettingSerializedUberState",
        UberIdentifier { uber_group: 8246, uber_id: 49364 },
    ),
    (
        "statsUberStateGroup.energySpentSerializedFloatUberState",
        UberIdentifier { uber_group: 8246, uber_id: 60940 },
    ),
    (
        "statsUberStateGroup.distanceTravelledStatSettingSerializedFloatUberState",
        UberIdentifier { uber_group: 8246, uber_id: 62310 },
    ),
    (
        "inkwaterMarshStateGroup.mokiTorchPlayed",
        UberIdentifier { uber_group: 9593, uber_id: 3621 },
    ),
    (
        "inkwaterMarshStateGroup.expOrbA",
        UberIdentifier { uber_group: 9593, uber_id: 5253 },
    ),
    (
        "inkwaterMarshStateGroup.xpOrbA",
        UberIdentifier { uber_group: 9593, uber_id: 5929 },
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier { uber_group: 9593, uber_id: 7849 },
    ),
    (
        "inkwaterMarshStateGroup.lanternAndCreepA",
        UberIdentifier { uber_group: 9593, uber_id: 9229 },
    ),
    (
        "inkwaterMarshStateGroup.breakableLogB",
        UberIdentifier { uber_group: 9593, uber_id: 14616 },
    ),
    (
        "inkwaterMarshStateGroup.climbHintShown",
        UberIdentifier { uber_group: 9593, uber_id: 15672 },
    ),
    (
        "inkwaterMarshStateGroup.stompableFloor",
        UberIdentifier { uber_group: 9593, uber_id: 17659 },
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier { uber_group: 9593, uber_id: 17818 },
    ),
    (
        "inkwaterMarshStateGroup.lasersDiscovered",
        UberIdentifier { uber_group: 9593, uber_id: 17991 },
    ),
    (
        "inkwaterMarshStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 9593, uber_id: 20382 },
    ),
    (
        "inkwaterMarshStateGroup.energyVessel",
        UberIdentifier { uber_group: 9593, uber_id: 22802 },
    ),
    (
        "inkwaterMarshStateGroup.breakableWall",
        UberIdentifier { uber_group: 9593, uber_id: 23319 },
    ),
    (
        "inkwaterMarshStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 9593, uber_id: 23858 },
    ),
    (
        "inkwaterMarshStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 9593, uber_id: 25989 },
    ),
    (
        "inkwaterMarshStateGroup.lanternAndCreepB",
        UberIdentifier { uber_group: 9593, uber_id: 26238 },
    ),
    (
        "inkwaterMarshStateGroup.energyContainer",
        UberIdentifier { uber_group: 9593, uber_id: 26457 },
    ),
    (
        "inkwaterMarshStateGroup.halfEnergyCellA",
        UberIdentifier { uber_group: 9593, uber_id: 27562 },
    ),
    (
        "inkwaterMarshStateGroup.secretWallA",
        UberIdentifier { uber_group: 9593, uber_id: 34704 },
    ),
    (
        "inkwaterMarshStateGroup.xpOrb",
        UberIdentifier { uber_group: 9593, uber_id: 42047 },
    ),
    (
        "inkwaterMarshStateGroup.xpOrbB",
        UberIdentifier { uber_group: 9593, uber_id: 45321 },
    ),
    (
        "inkwaterMarshStateGroup.secretWallA",
        UberIdentifier { uber_group: 9593, uber_id: 47420 },
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier { uber_group: 9593, uber_id: 53947 },
    ),
    (
        "inkwaterMarshStateGroup.expOrb",
        UberIdentifier { uber_group: 9593, uber_id: 59344 },
    ),
    (
        "inkwaterMarshStateGroup.enemyRoom",
        UberIdentifier { uber_group: 9593, uber_id: 59418 },
    ),
    (
        "inkwaterMarshStateGroup.healthContainer",
        UberIdentifier { uber_group: 9593, uber_id: 61304 },
    ),
    (
        "inkwaterMarshStateGroup.lizardMultiWaveArenaInt",
        UberIdentifier { uber_group: 9593, uber_id: 25130 },
    ),
    (
        "inkwaterMarshStateGroup.swampArenaAInt",
        UberIdentifier { uber_group: 9593, uber_id: 31687 },
    ),
    (
        "inkwaterMarshStateGroup.swampArenaA",
        UberIdentifier { uber_group: 9593, uber_id: 45142 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 94 },
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierD",
        UberIdentifier { uber_group: 10289, uber_id: 1620 },
    ),
    (
        "windtornRuinsGroup.ruinsVisited",
        UberIdentifier { uber_group: 10289, uber_id: 3621 },
    ),
    (
        "windtornRuinsGroup.openedDesertRuins",
        UberIdentifier { uber_group: 10289, uber_id: 3804 },
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierB",
        UberIdentifier { uber_group: 10289, uber_id: 4154 },
    ),
    (
        "windtornRuinsGroup.baseKillzoneState",
        UberIdentifier { uber_group: 10289, uber_id: 7638 },
    ),
    (
        "windtornRuinsGroup.bombableWallDesertC",
        UberIdentifier { uber_group: 10289, uber_id: 8436 },
    ),
    (
        "windtornRuinsGroup.drillZoneD",
        UberIdentifier { uber_group: 10289, uber_id: 8533 },
    ),
    (
        "windtornRuinsGroup.drillZoneF",
        UberIdentifier { uber_group: 10289, uber_id: 10093 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 12859 },
    ),
    (
        "windtornRuinsGroup.sandwormActiveA",
        UberIdentifier { uber_group: 10289, uber_id: 13021 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 15867 },
    ),
    (
        "windtornRuinsGroup.escapeBridgeB",
        UberIdentifier { uber_group: 10289, uber_id: 16802 },
    ),
    (
        "windtornRuinsGroup.wispRewardPickup",
        UberIdentifier { uber_group: 10289, uber_id: 22102 },
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierC",
        UberIdentifier { uber_group: 10289, uber_id: 23922 },
    ),
    (
        "windtornRuinsGroup.collapseSequenceB",
        UberIdentifier { uber_group: 10289, uber_id: 27089 },
    ),
    (
        "windtornRuinsGroup.escapeRockI",
        UberIdentifier { uber_group: 10289, uber_id: 27929 },
    ),
    (
        "windtornRuinsGroup.escapeRockC",
        UberIdentifier { uber_group: 10289, uber_id: 28779 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 29069 },
    ),
    (
        "windtornRuinsGroup.fallingPillars",
        UberIdentifier { uber_group: 10289, uber_id: 29425 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 30540 },
    ),
    (
        "windtornRuinsGroup.windsweptWastesRuinsDoorCannotOpen",
        UberIdentifier { uber_group: 10289, uber_id: 31524 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 31750 },
    ),
    (
        "windtornRuinsGroup.escapeRockG",
        UberIdentifier { uber_group: 10289, uber_id: 32483 },
    ),
    (
        "windtornRuinsGroup.escapeRockE",
        UberIdentifier { uber_group: 10289, uber_id: 32833 },
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier { uber_group: 10289, uber_id: 36274 },
    ),
    (
        "windtornRuinsGroup.windtornRuinsAKeystoneDoor",
        UberIdentifier { uber_group: 10289, uber_id: 37849 },
    ),
    (
        "windtornRuinsGroup.drillZoneC",
        UberIdentifier { uber_group: 10289, uber_id: 38171 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 38721 },
    ),
    (
        "windtornRuinsGroup.escapeRockF",
        UberIdentifier { uber_group: 10289, uber_id: 40310 },
    ),
    (
        "windtornRuinsGroup.goldenSeinBarrierA",
        UberIdentifier { uber_group: 10289, uber_id: 40790 },
    ),
    (
        "windtornRuinsGroup.lever",
        UberIdentifier { uber_group: 10289, uber_id: 41277 },
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier { uber_group: 10289, uber_id: 41902 },
    ),
    (
        "windtornRuinsGroup.rootBreakPillarFall",
        UberIdentifier { uber_group: 10289, uber_id: 43103 },
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier { uber_group: 10289, uber_id: 44426 },
    ),
    (
        "windtornRuinsGroup.energyHalfCell",
        UberIdentifier { uber_group: 10289, uber_id: 44555 },
    ),
    (
        "windtornRuinsGroup.escapeEndRocks",
        UberIdentifier { uber_group: 10289, uber_id: 45179 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 45766 },
    ),
    (
        "windtornRuinsGroup.drillZoneB",
        UberIdentifier { uber_group: 10289, uber_id: 46316 },
    ),
    (
        "windtornRuinsGroup.healthHalfCell",
        UberIdentifier { uber_group: 10289, uber_id: 48372 },
    ),
    (
        "windtornRuinsGroup.keystoneDoor",
        UberIdentifier { uber_group: 10289, uber_id: 48604 },
    ),
    (
        "windtornRuinsGroup.escapeRockJ",
        UberIdentifier { uber_group: 10289, uber_id: 50961 },
    ),
    (
        "windtornRuinsGroup.wormBreakFloor",
        UberIdentifier { uber_group: 10289, uber_id: 52478 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 55317 },
    ),
    (
        "windtornRuinsGroup.drillZoneE",
        UberIdentifier { uber_group: 10289, uber_id: 55672 },
    ),
    (
        "windtornRuinsGroup.escapeRockH",
        UberIdentifier { uber_group: 10289, uber_id: 55692 },
    ),
    (
        "windtornRuinsGroup.bombableWall",
        UberIdentifier { uber_group: 10289, uber_id: 55787 },
    ),
    (
        "windtornRuinsGroup.escapeRockA",
        UberIdentifier { uber_group: 10289, uber_id: 57325 },
    ),
    (
        "windtornRuinsGroup.areaText",
        UberIdentifier { uber_group: 10289, uber_id: 61217 },
    ),
    (
        "windtornRuinsGroup.xpOrbA",
        UberIdentifier { uber_group: 10289, uber_id: 61615 },
    ),
    (
        "windtornRuinsGroup.drillZoneC",
        UberIdentifier { uber_group: 10289, uber_id: 62291 },
    ),
    (
        "windtornRuinsGroup.collapseSequenceA",
        UberIdentifier { uber_group: 10289, uber_id: 62926 },
    ),
    (
        "windtornRuinsGroup.escapeRockD",
        UberIdentifier { uber_group: 10289, uber_id: 63154 },
    ),
    (
        "windtornRuinsGroup.desertSruinsChaseSandWall",
        UberIdentifier { uber_group: 10289, uber_id: 63700 },
    ),
    (
        "windtornRuinsGroup.drillZoneA",
        UberIdentifier { uber_group: 10289, uber_id: 64240 },
    ),
    (
        "windtornRuinsGroup.escapeRockB",
        UberIdentifier { uber_group: 10289, uber_id: 65145 },
    ),
    (
        "windtornRuinsGroup.savePedestalUberState",
        UberIdentifier { uber_group: 10289, uber_id: 4928 },
    ),
    (
        "windtornRuinsGroup.DesertSavePedestal",
        UberIdentifier { uber_group: 10289, uber_id: 13937 },
    ),
    (
        "windtornRuinsGroup.savePedestalUberState",
        UberIdentifier { uber_group: 10289, uber_id: 40484 },
    ),
    (
        "windtornRuinsGroup.rotatingBlockSetupRotation",
        UberIdentifier { uber_group: 10289, uber_id: 93 },
    ),
    (
        "windtornRuinsGroup.powerLineIntUberStateA",
        UberIdentifier { uber_group: 10289, uber_id: 312 },
    ),
    (
        "windtornRuinsGroup.powerLineIntUberState",
        UberIdentifier { uber_group: 10289, uber_id: 3217 },
    ),
    (
        "windtornRuinsGroup.powerLineIntUberStateC",
        UberIdentifier { uber_group: 10289, uber_id: 3682 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateB",
        UberIdentifier { uber_group: 10289, uber_id: 6414 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateC",
        UberIdentifier { uber_group: 10289, uber_id: 12614 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateG",
        UberIdentifier { uber_group: 10289, uber_id: 16886 },
    ),
    (
        "windtornRuinsGroup.desertRuinsEscape",
        UberIdentifier { uber_group: 10289, uber_id: 19890 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateF",
        UberIdentifier { uber_group: 10289, uber_id: 23855 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateA",
        UberIdentifier { uber_group: 10289, uber_id: 27997 },
    ),
    (
        "windtornRuinsGroup.powerLineIntUberState",
        UberIdentifier { uber_group: 10289, uber_id: 35130 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateE",
        UberIdentifier { uber_group: 10289, uber_id: 45821 },
    ),
    (
        "windtornRuinsGroup.wormNodeState",
        UberIdentifier { uber_group: 10289, uber_id: 47857 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateD",
        UberIdentifier { uber_group: 10289, uber_id: 50264 },
    ),
    (
        "windtornRuinsGroup.wormNodeStateH",
        UberIdentifier { uber_group: 10289, uber_id: 56515 },
    ),
    (
        "windtornRuinsGroup.powerLineIntUberStateB",
        UberIdentifier { uber_group: 10289, uber_id: 58350 },
    ),
    (
        "windtornRuinsGroup.desertRuinsWispSequencePlayed",
        UberIdentifier { uber_group: 10289, uber_id: 60565 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateA",
        UberIdentifier { uber_group: 10289, uber_id: 5546 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateD",
        UberIdentifier { uber_group: 10289, uber_id: 5814 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateC",
        UberIdentifier { uber_group: 10289, uber_id: 10828 },
    ),
    (
        "windtornRuinsGroup.wormDistanceToNextNodeState",
        UberIdentifier { uber_group: 10289, uber_id: 35190 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateH",
        UberIdentifier { uber_group: 10289, uber_id: 36008 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateB",
        UberIdentifier { uber_group: 10289, uber_id: 51149 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateE",
        UberIdentifier { uber_group: 10289, uber_id: 52211 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateF",
        UberIdentifier { uber_group: 10289, uber_id: 58175 },
    ),
    (
        "windtornRuinsGroup.wormDistanceStateG",
        UberIdentifier { uber_group: 10289, uber_id: 63894 },
    ),
    (
        "howlsDenGRoup.hasOriUsedSavePedestal",
        UberIdentifier { uber_group: 11666, uber_id: 4220 },
    ),
    (
        "howlsDenGRoup.saveRoomDoor",
        UberIdentifier { uber_group: 11666, uber_id: 4932 },
    ),
    (
        "howlsDenGRoup.howlsDenLargeXPOrbA",
        UberIdentifier { uber_group: 11666, uber_id: 24943 },
    ),
    (
        "howlsDenGRoup.areaText",
        UberIdentifier { uber_group: 11666, uber_id: 42038 },
    ),
    (
        "howlsDenGRoup.savePedestalUberState",
        UberIdentifier { uber_group: 11666, uber_id: 16542 },
    ),
    (
        "howlsDenGRoup.savePedestalUberState",
        UberIdentifier { uber_group: 11666, uber_id: 20829 },
    ),
    (
        "howlsDenGRoup.savePedestal",
        UberIdentifier { uber_group: 11666, uber_id: 61594 },
    ),
    (
        "leaderboardsUberStateGroup.baursReachLeaderboardNotificationState",
        UberIdentifier { uber_group: 13298, uber_id: 54921 },
    ),
    (
        "leaderboardsUberStateGroup.baursReachLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 3608 },
    ),
    (
        "leaderboardsUberStateGroup.desertLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 4929 },
    ),
    (
        "leaderboardsUberStateGroup.hornbugBossLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 6736 },
    ),
    (
        "leaderboardsUberStateGroup.watermillEscapeLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 14784 },
    ),
    (
        "leaderboardsUberStateGroup.laserShooterMiniBossLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 20341 },
    ),
    (
        "leaderboardsUberStateGroup.kwolokBossLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 37881 },
    ),
    (
        "leaderboardsUberStateGroup.inkwaterLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 40104 },
    ),
    (
        "leaderboardsUberStateGroup.spiderBossLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 41733 },
    ),
    (
        "leaderboardsUberStateGroup.desertEscapeLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 44392 },
    ),
    (
        "leaderboardsUberStateGroup.kwoloksLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 53149 },
    ),
    (
        "leaderboardsUberStateGroup.avalancheEscapeLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 53528 },
    ),
    (
        "leaderboardsUberStateGroup.wellspringLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 53967 },
    ),
    (
        "leaderboardsUberStateGroup.silentWoodlandLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 55577 },
    ),
    (
        "leaderboardsUberStateGroup.lumaPoolsLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 58679 },
    ),
    (
        "leaderboardsUberStateGroup.mouldwoodLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 59179 },
    ),
    (
        "leaderboardsUberStateGroup.owlBossLeaderboardPlaceState",
        UberIdentifier { uber_group: 13298, uber_id: 64962 },
    ),
    (
        "bashIntroductionA__clone1Group.healthContainerA",
        UberIdentifier { uber_group: 13428, uber_id: 59730 },
    ),
    (
        "questUberStateGroup.gardenerHutDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 353 },
    ),
    (
        "questUberStateGroup.darkCaveQuestItemCollected",
        UberIdentifier { uber_group: 14019, uber_id: 2782 },
    ),
    (
        "questUberStateGroup.firstRaceDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 5662 },
    ),
    (
        "questUberStateGroup.gardenerSeedTreeCollected",
        UberIdentifier { uber_group: 14019, uber_id: 7470 },
    ),
    (
        "questUberStateGroup.gardenerSeedBashCollected",
        UberIdentifier { uber_group: 14019, uber_id: 8192 },
    ),
    (
        "questUberStateGroup.mapstoneDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 9874 },
    ),
    (
        "questUberStateGroup.mouldwoodDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 12642 },
    ),
    (
        "questUberStateGroup.lanternItemCollected",
        UberIdentifier { uber_group: 14019, uber_id: 14931 },
    ),
    (
        "questUberStateGroup.howlsOriginWellOpened",
        UberIdentifier { uber_group: 14019, uber_id: 20290 },
    ),
    (
        "questUberStateGroup.gardenerSeedFlowersCollected",
        UberIdentifier { uber_group: 14019, uber_id: 20601 },
    ),
    (
        "questUberStateGroup.gardenerSeedGrappleCollected",
        UberIdentifier { uber_group: 14019, uber_id: 24142 },
    ),
    (
        "questUberStateGroup.wellspringShrineDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 27270 },
    ),
    (
        "questUberStateGroup.braveMokiItemCollected",
        UberIdentifier { uber_group: 14019, uber_id: 27539 },
    ),
    (
        "questUberStateGroup.gardenerSeedGrassCollected",
        UberIdentifier { uber_group: 14019, uber_id: 28662 },
    ),
    (
        "questUberStateGroup.desertDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 29163 },
    ),
    (
        "questUberStateGroup.lagoonDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 29202 },
    ),
    (
        "questUberStateGroup.howlsOriginDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 30671 },
    ),
    (
        "questUberStateGroup.desertRuinsDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 31413 },
    ),
    (
        "questUberStateGroup.gardenerSeedSpringCollected",
        UberIdentifier { uber_group: 14019, uber_id: 32376 },
    ),
    (
        "questUberStateGroup.mapSecretsRevealed",
        UberIdentifier { uber_group: 14019, uber_id: 35534 },
    ),
    (
        "questUberStateGroup.howlsDenShrineDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 36248 },
    ),
    (
        "questUberStateGroup.inkwaterShrineDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 40630 },
    ),
    (
        "questUberStateGroup.baurDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 46529 },
    ),
    (
        "questUberStateGroup.discoveredWillowsEnd",
        UberIdentifier { uber_group: 14019, uber_id: 50847 },
    ),
    (
        "questUberStateGroup.silentWoodsShrineDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 52274 },
    ),
    (
        "questUberStateGroup.howlsOriginTreasureCollected",
        UberIdentifier { uber_group: 14019, uber_id: 52747 },
    ),
    (
        "questUberStateGroup.kwoloksWisdomItemCollected",
        UberIdentifier { uber_group: 14019, uber_id: 53103 },
    ),
    (
        "questUberStateGroup.mouldwoodShrineDiscovered",
        UberIdentifier { uber_group: 14019, uber_id: 54970 },
    ),
    (
        "questUberStateGroup.familyReunionItemCollected",
        UberIdentifier { uber_group: 14019, uber_id: 57399 },
    ),
    (
        "questUberStateGroup.mineGemItemCollected",
        UberIdentifier { uber_group: 14019, uber_id: 58342 },
    ),
    (
        "questUberStateGroup.inDangerBool",
        UberIdentifier { uber_group: 14019, uber_id: 60646 },
    ),
    (
        "questUberStateGroup.desertCogItemCollected",
        UberIdentifier { uber_group: 14019, uber_id: 63396 },
    ),
    (
        "questUberStateGroup.discoveredWeepingRidge",
        UberIdentifier { uber_group: 14019, uber_id: 63965 },
    ),
    (
        "questUberStateGroup.helpingHandQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 1341 },
    ),
    (
        "questUberStateGroup.reachWaterMillQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 5737 },
    ),
    (
        "questUberStateGroup.dialogQuest",
        UberIdentifier { uber_group: 14019, uber_id: 6284 },
    ),
    (
        "questUberStateGroup.winterForestWispQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 8973 },
    ),
    (
        "questUberStateGroup.baursReachJTokkInteractionQuest",
        UberIdentifier { uber_group: 14019, uber_id: 11308 },
    ),
    (
        "questUberStateGroup.howlsDenShrineRumorMokiState",
        UberIdentifier { uber_group: 14019, uber_id: 12437 },
    ),
    (
        "questUberStateGroup.mouldwoodRumorMokiState",
        UberIdentifier { uber_group: 14019, uber_id: 13512 },
    ),
    (
        "questUberStateGroup.braveMokiQuest",
        UberIdentifier { uber_group: 14019, uber_id: 15983 },
    ),
    (
        "questUberStateGroup.wellspringShrineRumorMokiState",
        UberIdentifier { uber_group: 14019, uber_id: 15995 },
    ),
    (
        "questUberStateGroup.wellspringShrineRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 16509 },
    ),
    (
        "questUberStateGroup.mouldwoodShrineRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 18061 },
    ),
    (
        "questUberStateGroup.lagoonRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 19024 },
    ),
    (
        "questUberStateGroup.desertRuinsRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 19060 },
    ),
    (
        "questUberStateGroup.brothersQuest",
        UberIdentifier { uber_group: 14019, uber_id: 19157 },
    ),
    (
        "questUberStateGroup.lostCompassQuest",
        UberIdentifier { uber_group: 14019, uber_id: 20667 },
    ),
    (
        "questUberStateGroup.gardenerIntroQuest",
        UberIdentifier { uber_group: 14019, uber_id: 23459 },
    ),
    (
        "questUberStateGroup.lastTreeQuest",
        UberIdentifier { uber_group: 14019, uber_id: 23787 },
    ),
    (
        "questUberStateGroup.inkwaterShrineRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 23863 },
    ),
    (
        "questUberStateGroup.optionalVSQuestAUberState",
        UberIdentifier { uber_group: 14019, uber_id: 24152 },
    ),
    (
        "questUberStateGroup.luposMapQuest",
        UberIdentifier { uber_group: 14019, uber_id: 24683 },
    ),
    (
        "questUberStateGroup.tradeSequenceQuest",
        UberIdentifier { uber_group: 14019, uber_id: 26318 },
    ),
    (
        "questUberStateGroup.regrowGladesQuest",
        UberIdentifier { uber_group: 14019, uber_id: 26394 },
    ),
    (
        "questUberStateGroup.silentWoodsShrineRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 27011 },
    ),
    (
        "questUberStateGroup.familyReunionQuest",
        UberIdentifier { uber_group: 14019, uber_id: 27804 },
    ),
    (
        "questUberStateGroup.howlsDenShrineRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 27822 },
    ),
    (
        "questUberStateGroup.gardenerHutRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 30596 },
    ),
    (
        "questUberStateGroup.freeGromQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 33762 },
    ),
    (
        "questUberStateGroup.darkCaveQuest",
        UberIdentifier { uber_group: 14019, uber_id: 33776 },
    ),
    (
        "questUberStateGroup.findKuQuest",
        UberIdentifier { uber_group: 14019, uber_id: 34504 },
    ),
    (
        "questUberStateGroup.lagoonWispQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 35087 },
    ),
    (
        "questUberStateGroup.desertWispQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 35399 },
    ),
    (
        "questUberStateGroup.mapstoneRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 39957 },
    ),
    (
        "questUberStateGroup.howlsOriginRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 40952 },
    ),
    (
        "questUberStateGroup.firstRaceRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 42501 },
    ),
    (
        "questUberStateGroup.findHelpQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 44059 },
    ),
    (
        "questUberStateGroup.baurRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 44184 },
    ),
    (
        "questUberStateGroup.lookForKuQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 44500 },
    ),
    (
        "questUberStateGroup.rebuildGladesQuest",
        UberIdentifier { uber_group: 14019, uber_id: 44578 },
    ),
    (
        "questUberStateGroup.mouldwoodDepthsWispQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 45931 },
    ),
    (
        "questUberStateGroup.lagoonRumorMokiState",
        UberIdentifier { uber_group: 14019, uber_id: 47774 },
    ),
    (
        "questUberStateGroup.findToadQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 48794 },
    ),
    (
        "questUberStateGroup.baurRumorMokiState",
        UberIdentifier { uber_group: 14019, uber_id: 50230 },
    ),
    (
        "questUberStateGroup.swampSpringIntroductionBOpherInteractionQuest",
        UberIdentifier { uber_group: 14019, uber_id: 50571 },
    ),
    (
        "questUberStateGroup.kwoloksWisdomQuest",
        UberIdentifier { uber_group: 14019, uber_id: 50597 },
    ),
    (
        "questUberStateGroup.mouldwoodRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 53066 },
    ),
    (
        "questUberStateGroup.lastGlobalEvent",
        UberIdentifier { uber_group: 14019, uber_id: 54675 },
    ),
    (
        "questUberStateGroup.killTentacleQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 57066 },
    ),
    (
        "questUberStateGroup.desertRumorState",
        UberIdentifier { uber_group: 14019, uber_id: 57552 },
    ),
    (
        "questUberStateGroup.searchForGrolQuest",
        UberIdentifier { uber_group: 14019, uber_id: 59705 },
    ),
    (
        "questUberStateGroup.treeKeeperQuest",
        UberIdentifier { uber_group: 14019, uber_id: 59708 },
    ),
    (
        "questUberStateGroup.desertCogQuest",
        UberIdentifier { uber_group: 14019, uber_id: 61011 },
    ),
    (
        "questUberStateGroup.getInitialWeaponQuestUberState",
        UberIdentifier { uber_group: 14019, uber_id: 62230 },
    ),
    (
        "questUberStateGroup.firstRaceRumorMokiState",
        UberIdentifier { uber_group: 14019, uber_id: 62288 },
    ),
    (
        "willowsEndGroup.expOrb",
        UberIdentifier { uber_group: 16155, uber_id: 2065 },
    ),
    (
        "willowsEndGroup.fallingPortalB",
        UberIdentifier { uber_group: 16155, uber_id: 2235 },
    ),
    (
        "willowsEndGroup.breakableWallA",
        UberIdentifier { uber_group: 16155, uber_id: 3096 },
    ),
    (
        "willowsEndGroup.vineEClear",
        UberIdentifier { uber_group: 16155, uber_id: 3588 },
    ),
    (
        "willowsEndGroup.arenaPlatform3Destroyed",
        UberIdentifier { uber_group: 16155, uber_id: 3670 },
    ),
    (
        "willowsEndGroup.arenaPlatform2Destroyed",
        UberIdentifier { uber_group: 16155, uber_id: 5826 },
    ),
    (
        "willowsEndGroup.gorlekOreA",
        UberIdentifier { uber_group: 16155, uber_id: 9230 },
    ),
    (
        "willowsEndGroup.fallingPortal",
        UberIdentifier { uber_group: 16155, uber_id: 18906 },
    ),
    (
        "willowsEndGroup.creepA",
        UberIdentifier { uber_group: 16155, uber_id: 20672 },
    ),
    (
        "willowsEndGroup.chaseSequenceG",
        UberIdentifier { uber_group: 16155, uber_id: 21083 },
    ),
    (
        "willowsEndGroup.breakableWallC",
        UberIdentifier { uber_group: 16155, uber_id: 21899 },
    ),
    (
        "willowsEndGroup.vineCClear",
        UberIdentifier { uber_group: 16155, uber_id: 24290 },
    ),
    (
        "willowsEndGroup.xpOrbA",
        UberIdentifier { uber_group: 16155, uber_id: 25259 },
    ),
    (
        "willowsEndGroup.chaseSequenceA",
        UberIdentifier { uber_group: 16155, uber_id: 27024 },
    ),
    (
        "willowsEndGroup.vineDClear",
        UberIdentifier { uber_group: 16155, uber_id: 28478 },
    ),
    (
        "willowsEndGroup.introPlayed",
        UberIdentifier { uber_group: 16155, uber_id: 32922 },
    ),
    (
        "willowsEndGroup.breakableWallC",
        UberIdentifier { uber_group: 16155, uber_id: 33738 },
    ),
    (
        "willowsEndGroup.secretWall",
        UberIdentifier { uber_group: 16155, uber_id: 36353 },
    ),
    (
        "willowsEndGroup.breakableWallA",
        UberIdentifier { uber_group: 16155, uber_id: 36873 },
    ),
    (
        "willowsEndGroup.breakableWallB",
        UberIdentifier { uber_group: 16155, uber_id: 37558 },
    ),
    (
        "willowsEndGroup.chaseSequenceC",
        UberIdentifier { uber_group: 16155, uber_id: 37648 },
    ),
    (
        "willowsEndGroup.chaseSequenceD",
        UberIdentifier { uber_group: 16155, uber_id: 38867 },
    ),
    (
        "willowsEndGroup.gorlekOreA",
        UberIdentifier { uber_group: 16155, uber_id: 38979 },
    ),
    (
        "willowsEndGroup.vineGClear",
        UberIdentifier { uber_group: 16155, uber_id: 41488 },
    ),
    (
        "willowsEndGroup.secretWallA",
        UberIdentifier { uber_group: 16155, uber_id: 42106 },
    ),
    (
        "willowsEndGroup.vineAClear",
        UberIdentifier { uber_group: 16155, uber_id: 42976 },
    ),
    (
        "willowsEndGroup.chaseSequenceB",
        UberIdentifier { uber_group: 16155, uber_id: 44311 },
    ),
    (
        "willowsEndGroup.arenaPlatform1Destroyed",
        UberIdentifier { uber_group: 16155, uber_id: 45630 },
    ),
    (
        "willowsEndGroup.healthCellA",
        UberIdentifier { uber_group: 16155, uber_id: 46270 },
    ),
    (
        "willowsEndGroup.expOrbB",
        UberIdentifier { uber_group: 16155, uber_id: 47690 },
    ),
    (
        "willowsEndGroup.expOrbA",
        UberIdentifier { uber_group: 16155, uber_id: 49381 },
    ),
    (
        "willowsEndGroup.chaseSequenceE",
        UberIdentifier { uber_group: 16155, uber_id: 49408 },
    ),
    (
        "willowsEndGroup.expOrb",
        UberIdentifier { uber_group: 16155, uber_id: 49457 },
    ),
    (
        "willowsEndGroup.chaseSequenceF",
        UberIdentifier { uber_group: 16155, uber_id: 49744 },
    ),
    (
        "willowsEndGroup.breakableWallB",
        UberIdentifier { uber_group: 16155, uber_id: 52848 },
    ),
    (
        "willowsEndGroup.groundDestroyed",
        UberIdentifier { uber_group: 16155, uber_id: 54148 },
    ),
    (
        "willowsEndGroup.vineBClear",
        UberIdentifier { uber_group: 16155, uber_id: 54940 },
    ),
    (
        "willowsEndGroup.xpOrbA",
        UberIdentifier { uber_group: 16155, uber_id: 55446 },
    ),
    (
        "willowsEndGroup.fallingPortalA",
        UberIdentifier { uber_group: 16155, uber_id: 55721 },
    ),
    (
        "willowsEndGroup.vineHClear",
        UberIdentifier { uber_group: 16155, uber_id: 60752 },
    ),
    (
        "willowsEndGroup.arenaPlatform4Destroyed",
        UberIdentifier { uber_group: 16155, uber_id: 63705 },
    ),
    (
        "willowsEndGroup.vineFClear",
        UberIdentifier { uber_group: 16155, uber_id: 65277 },
    ),
    (
        "willowsEndGroup.savePedestalUberState",
        UberIdentifier { uber_group: 16155, uber_id: 41465 },
    ),
    (
        "willowsEndGroup.savePedestalUberState",
        UberIdentifier { uber_group: 16155, uber_id: 50867 },
    ),
    (
        "willowsEndGroup.laserShooterBossState",
        UberIdentifier { uber_group: 16155, uber_id: 12971 },
    ),
    (
        "willowsEndGroup.petrifiedOwlBossState",
        UberIdentifier { uber_group: 16155, uber_id: 47278 },
    ),
    (
        "mouldwoodDepthsGroup.orePickupA",
        UberIdentifier { uber_group: 18793, uber_id: 836 },
    ),
    (
        "mouldwoodDepthsGroup.keystone",
        UberIdentifier { uber_group: 18793, uber_id: 1914 },
    ),
    (
        "mouldwoodDepthsGroup.expOrbB",
        UberIdentifier { uber_group: 18793, uber_id: 2881 },
    ),
    (
        "mouldwoodDepthsGroup.doorWithFourSlots",
        UberIdentifier { uber_group: 18793, uber_id: 3171 },
    ),
    (
        "mouldwoodDepthsGroup.blockerWallBroken",
        UberIdentifier { uber_group: 18793, uber_id: 4645 },
    ),
    (
        "mouldwoodDepthsGroup.creepDestroyedA",
        UberIdentifier { uber_group: 18793, uber_id: 4664 },
    ),
    (
        "mouldwoodDepthsGroup.secretWall",
        UberIdentifier { uber_group: 18793, uber_id: 5315 },
    ),
    (
        "mouldwoodDepthsGroup.expOrbB",
        UberIdentifier { uber_group: 18793, uber_id: 5797 },
    ),
    (
        "mouldwoodDepthsGroup.expOrbB",
        UberIdentifier { uber_group: 18793, uber_id: 6573 },
    ),
    (
        "mouldwoodDepthsGroup.expOrbA",
        UberIdentifier { uber_group: 18793, uber_id: 9251 },
    ),
    (
        "mouldwoodDepthsGroup.doorWithFourSlots",
        UberIdentifier { uber_group: 18793, uber_id: 10372 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsGDoorWithTwoSlotsOpened",
        UberIdentifier { uber_group: 18793, uber_id: 10758 },
    ),
    (
        "mouldwoodDepthsGroup.creepB",
        UberIdentifier { uber_group: 18793, uber_id: 11676 },
    ),
    (
        "mouldwoodDepthsGroup.shrineEnemies",
        UberIdentifier { uber_group: 18793, uber_id: 12512 },
    ),
    (
        "mouldwoodDepthsGroup.kwolokCavernsBreakableFloor",
        UberIdentifier { uber_group: 18793, uber_id: 13281 },
    ),
    (
        "mouldwoodDepthsGroup.shortcutWall",
        UberIdentifier { uber_group: 18793, uber_id: 13349 },
    ),
    (
        "mouldwoodDepthsGroup.darknessLiftedUberState",
        UberIdentifier { uber_group: 18793, uber_id: 13352 },
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepB",
        UberIdentifier { uber_group: 18793, uber_id: 13367 },
    ),
    (
        "mouldwoodDepthsGroup.leverAndDoorA",
        UberIdentifier { uber_group: 18793, uber_id: 14503 },
    ),
    (
        "mouldwoodDepthsGroup.xpOrbC",
        UberIdentifier { uber_group: 18793, uber_id: 15396 },
    ),
    (
        "mouldwoodDepthsGroup.arenaBottomBrokenFloor",
        UberIdentifier { uber_group: 18793, uber_id: 15422 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHBreakableWallB",
        UberIdentifier { uber_group: 18793, uber_id: 15855 },
    ),
    (
        "mouldwoodDepthsGroup.chamberWebFBroken",
        UberIdentifier { uber_group: 18793, uber_id: 18064 },
    ),
    (
        "mouldwoodDepthsGroup.XPOrbA",
        UberIdentifier { uber_group: 18793, uber_id: 18395 },
    ),
    (
        "mouldwoodDepthsGroup.chamberWebEBroken",
        UberIdentifier { uber_group: 18793, uber_id: 18563 },
    ),
    (
        "mouldwoodDepthsGroup.mediumExpA",
        UberIdentifier { uber_group: 18793, uber_id: 19004 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneBCollected",
        UberIdentifier { uber_group: 18793, uber_id: 20493 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneCCollected",
        UberIdentifier { uber_group: 18793, uber_id: 20959 },
    ),
    (
        "mouldwoodDepthsGroup.breakableWallA",
        UberIdentifier { uber_group: 18793, uber_id: 21022 },
    ),
    (
        "mouldwoodDepthsGroup.puzzleSolvedSequenceCompleted",
        UberIdentifier { uber_group: 18793, uber_id: 21994 },
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepA",
        UberIdentifier { uber_group: 18793, uber_id: 22368 },
    ),
    (
        "mouldwoodDepthsGroup.expOrbC",
        UberIdentifier { uber_group: 18793, uber_id: 23799 },
    ),
    (
        "mouldwoodDepthsGroup.areaText",
        UberIdentifier { uber_group: 18793, uber_id: 23953 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHKeystoneBCollected",
        UberIdentifier { uber_group: 18793, uber_id: 23986 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodGateOpen",
        UberIdentifier { uber_group: 18793, uber_id: 25789 },
    ),
    (
        "mouldwoodDepthsGroup.energyContainerA",
        UberIdentifier { uber_group: 18793, uber_id: 26618 },
    ),
    (
        "mouldwoodDepthsGroup.stompableFloor",
        UberIdentifier { uber_group: 18793, uber_id: 27207 },
    ),
    (
        "mouldwoodDepthsGroup.energyContainerA",
        UberIdentifier { uber_group: 18793, uber_id: 28175 },
    ),
    (
        "mouldwoodDepthsGroup.spiderIntereactedAfterFight",
        UberIdentifier { uber_group: 18793, uber_id: 28205 },
    ),
    (
        "mouldwoodDepthsGroup.breakableWebB",
        UberIdentifier { uber_group: 18793, uber_id: 28677 },
    ),
    (
        "mouldwoodDepthsGroup.brokenTrunkTop",
        UberIdentifier { uber_group: 18793, uber_id: 28692 },
    ),
    (
        "mouldwoodDepthsGroup.creepA",
        UberIdentifier { uber_group: 18793, uber_id: 29066 },
    ),
    (
        "mouldwoodDepthsGroup.mediumExpA",
        UberIdentifier { uber_group: 18793, uber_id: 29533 },
    ),
    (
        "mouldwoodDepthsGroup.expOrbA",
        UberIdentifier { uber_group: 18793, uber_id: 29979 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHPushBlockPushed",
        UberIdentifier { uber_group: 18793, uber_id: 30627 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneDCollected",
        UberIdentifier { uber_group: 18793, uber_id: 30708 },
    ),
    (
        "mouldwoodDepthsGroup.shardSlotUpgradePlaceholder",
        UberIdentifier { uber_group: 18793, uber_id: 31937 },
    ),
    (
        "mouldwoodDepthsGroup.chamberWebBBroken",
        UberIdentifier { uber_group: 18793, uber_id: 32305 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsJKeystoneACollected",
        UberIdentifier { uber_group: 18793, uber_id: 32441 },
    ),
    (
        "mouldwoodDepthsGroup.doorWithFourSlots",
        UberIdentifier { uber_group: 18793, uber_id: 33471 },
    ),
    (
        "mouldwoodDepthsGroup.orePickupA",
        UberIdentifier { uber_group: 18793, uber_id: 35351 },
    ),
    (
        "mouldwoodDepthsGroup.expOrb",
        UberIdentifier { uber_group: 18793, uber_id: 38941 },
    ),
    (
        "mouldwoodDepthsGroup.creepA",
        UberIdentifier { uber_group: 18793, uber_id: 39232 },
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepCTest",
        UberIdentifier { uber_group: 18793, uber_id: 40612 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHDoorWithFourSlotsOpened",
        UberIdentifier { uber_group: 18793, uber_id: 41544 },
    ),
    (
        "mouldwoodDepthsGroup.healthCellB",
        UberIdentifier { uber_group: 18793, uber_id: 42235 },
    ),
    (
        "mouldwoodDepthsGroup.mediumExpB",
        UberIdentifier { uber_group: 18793, uber_id: 42980 },
    ),
    (
        "mouldwoodDepthsGroup.webFallState",
        UberIdentifier { uber_group: 18793, uber_id: 44522 },
    ),
    (
        "mouldwoodDepthsGroup.leafPileA",
        UberIdentifier { uber_group: 18793, uber_id: 44773 },
    ),
    (
        "mouldwoodDepthsGroup.hintZoneFlash",
        UberIdentifier { uber_group: 18793, uber_id: 45899 },
    ),
    (
        "mouldwoodDepthsGroup.bottomRightSmallWall",
        UberIdentifier { uber_group: 18793, uber_id: 45963 },
    ),
    (
        "mouldwoodDepthsGroup.verticalFallingTrunk",
        UberIdentifier { uber_group: 18793, uber_id: 49362 },
    ),
    (
        "mouldwoodDepthsGroup.XPOrbB",
        UberIdentifier { uber_group: 18793, uber_id: 49526 },
    ),
    (
        "mouldwoodDepthsGroup.XPOrbA",
        UberIdentifier { uber_group: 18793, uber_id: 49759 },
    ),
    (
        "mouldwoodDepthsGroup.hintZoneFlashCharge",
        UberIdentifier { uber_group: 18793, uber_id: 50745 },
    ),
    (
        "mouldwoodDepthsGroup.arenaTrunkBroken",
        UberIdentifier { uber_group: 18793, uber_id: 53347 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHKeystoneACollected",
        UberIdentifier { uber_group: 18793, uber_id: 53953 },
    ),
    (
        "mouldwoodDepthsGroup.breakableWebA",
        UberIdentifier { uber_group: 18793, uber_id: 56320 },
    ),
    (
        "mouldwoodDepthsGroup.arenaBreakableA",
        UberIdentifier { uber_group: 18793, uber_id: 56666 },
    ),
    (
        "mouldwoodDepthsGroup.bottomLeftSmallWall",
        UberIdentifier { uber_group: 18793, uber_id: 56800 },
    ),
    (
        "mouldwoodDepthsGroup.keystoneA",
        UberIdentifier { uber_group: 18793, uber_id: 58148 },
    ),
    (
        "mouldwoodDepthsGroup.mediumExpC",
        UberIdentifier { uber_group: 18793, uber_id: 58342 },
    ),
    (
        "mouldwoodDepthsGroup.healthCellA",
        UberIdentifier { uber_group: 18793, uber_id: 62694 },
    ),
    (
        "mouldwoodDepthsGroup.bossReward",
        UberIdentifier { uber_group: 18793, uber_id: 63291 },
    ),
    (
        "mouldwoodDepthsGroup.arenaWallMid",
        UberIdentifier { uber_group: 18793, uber_id: 64305 },
    ),
    (
        "mouldwoodDepthsGroup.mouldwoodDepthsHBreakableWallA",
        UberIdentifier { uber_group: 18793, uber_id: 64772 },
    ),
    (
        "mouldwoodDepthsGroup.secretWallA",
        UberIdentifier { uber_group: 18793, uber_id: 65202 },
    ),
    (
        "mouldwoodDepthsGroup.savePedestalUberState",
        UberIdentifier { uber_group: 18793, uber_id: 38871 },
    ),
    (
        "mouldwoodDepthsGroup.savePedestalUberState",
        UberIdentifier { uber_group: 18793, uber_id: 39689 },
    ),
    (
        "mouldwoodDepthsGroup.spiderBossState",
        UberIdentifier { uber_group: 18793, uber_id: 26713 },
    ),
    (
        "mouldwoodDepthsGroup.lanternAndCreepBInt",
        UberIdentifier { uber_group: 18793, uber_id: 39667 },
    ),
    (
        "eventsUberStateGroup.gumoFreeUberState",
        UberIdentifier { uber_group: 19973, uber_id: 18551 },
    ),
    (
        "eventsUberStateGroup.spiritTreeReachedUberState",
        UberIdentifier { uber_group: 19973, uber_id: 22047 },
    ),
    (
        "eventsUberStateGroup.mistLiftedUberState",
        UberIdentifier { uber_group: 19973, uber_id: 23591 },
    ),
    (
        "eventsUberStateGroup.ginsoTreeKeyUberState",
        UberIdentifier { uber_group: 19973, uber_id: 30524 },
    ),
    (
        "eventsUberStateGroup.kwolokDeadUberState",
        UberIdentifier { uber_group: 19973, uber_id: 31305 },
    ),
    (
        "eventsUberStateGroup.mountHoruKeyUberState",
        UberIdentifier { uber_group: 19973, uber_id: 38631 },
    ),
    (
        "eventsUberStateGroup.kwolokLeftThroneUberState",
        UberIdentifier { uber_group: 19973, uber_id: 45830 },
    ),
    (
        "eventsUberStateGroup.gravityActivatedUberState",
        UberIdentifier { uber_group: 19973, uber_id: 49418 },
    ),
    (
        "eventsUberStateGroup.ginsoTreeEnteredUberState",
        UberIdentifier { uber_group: 19973, uber_id: 54999 },
    ),
    (
        "eventsUberStateGroup.windRestoredUberState",
        UberIdentifier { uber_group: 19973, uber_id: 59537 },
    ),
    (
        "eventsUberStateGroup.forlornRuinsKeyUberState",
        UberIdentifier { uber_group: 19973, uber_id: 61347 },
    ),
    (
        "windsweptWastesGroupDescriptor.expOrb",
        UberIdentifier { uber_group: 20120, uber_id: 224 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallA",
        UberIdentifier { uber_group: 20120, uber_id: 1348 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbG",
        UberIdentifier { uber_group: 20120, uber_id: 2013 },
    ),
    (
        "windsweptWastesGroupDescriptor.areaText",
        UberIdentifier { uber_group: 20120, uber_id: 2552 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbC",
        UberIdentifier { uber_group: 20120, uber_id: 3550 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 20120, uber_id: 8910 },
    ),
    (
        "windsweptWastesGroupDescriptor.breakableWall",
        UberIdentifier { uber_group: 20120, uber_id: 9095 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbB",
        UberIdentifier { uber_group: 20120, uber_id: 10397 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbD",
        UberIdentifier { uber_group: 20120, uber_id: 10801 },
    ),
    (
        "windsweptWastesGroupDescriptor.energyHalfCell",
        UberIdentifier { uber_group: 20120, uber_id: 11785 },
    ),
    (
        "windsweptWastesGroupDescriptor.leverStateA",
        UberIdentifier { uber_group: 20120, uber_id: 12902 },
    ),
    (
        "windsweptWastesGroupDescriptor.healthContainer",
        UberIdentifier { uber_group: 20120, uber_id: 12941 },
    ),
    (
        "windsweptWastesGroupDescriptor.projectileBreakableWall",
        UberIdentifier { uber_group: 20120, uber_id: 16172 },
    ),
    (
        "windsweptWastesGroupDescriptor.digHint",
        UberIdentifier { uber_group: 20120, uber_id: 16309 },
    ),
    (
        "windsweptWastesGroupDescriptor.expOrbE",
        UberIdentifier { uber_group: 20120, uber_id: 17798 },
    ),
    (
        "windsweptWastesGroupDescriptor.lifeHalfCell",
        UberIdentifier { uber_group: 20120, uber_id: 18965 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbB",
        UberIdentifier { uber_group: 20120, uber_id: 19113 },
    ),
    (
        "windsweptWastesGroupDescriptor.energyOrbA",
        UberIdentifier { uber_group: 20120, uber_id: 22354 },
    ),
    (
        "windsweptWastesGroupDescriptor.digDashHint",
        UberIdentifier { uber_group: 20120, uber_id: 24078 },
    ),
    (
        "windsweptWastesGroupDescriptor.drillableWallA",
        UberIdentifier { uber_group: 20120, uber_id: 24774 },
    ),
    (
        "windsweptWastesGroupDescriptor.e3DesertG_clone0_KeystoneDoor",
        UberIdentifier { uber_group: 20120, uber_id: 28786 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbE",
        UberIdentifier { uber_group: 20120, uber_id: 30358 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 20120, uber_id: 30740 },
    ),
    (
        "windsweptWastesGroupDescriptor.drillableBlockerA",
        UberIdentifier { uber_group: 20120, uber_id: 31180 },
    ),
    (
        "windsweptWastesGroupDescriptor.expOrbB",
        UberIdentifier { uber_group: 20120, uber_id: 33275 },
    ),
    (
        "windsweptWastesGroupDescriptor.shardA",
        UberIdentifier { uber_group: 20120, uber_id: 33292 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallC",
        UberIdentifier { uber_group: 20120, uber_id: 33294 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallB",
        UberIdentifier { uber_group: 20120, uber_id: 33775 },
    ),
    (
        "windsweptWastesGroupDescriptor.drillWallA",
        UberIdentifier { uber_group: 20120, uber_id: 36758 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 20120, uber_id: 36805 },
    ),
    (
        "windsweptWastesGroupDescriptor.gorlekOreB",
        UberIdentifier { uber_group: 20120, uber_id: 40245 },
    ),
    (
        "windsweptWastesGroupDescriptor.hintZoneA",
        UberIdentifier { uber_group: 20120, uber_id: 40816 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 20120, uber_id: 42393 },
    ),
    (
        "windsweptWastesGroupDescriptor.shootablePod",
        UberIdentifier { uber_group: 20120, uber_id: 43099 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWall",
        UberIdentifier { uber_group: 20120, uber_id: 43231 },
    ),
    (
        "windsweptWastesGroupDescriptor.gorlekOre",
        UberIdentifier { uber_group: 20120, uber_id: 46919 },
    ),
    (
        "windsweptWastesGroupDescriptor.verticalPlatformLeverA",
        UberIdentifier { uber_group: 20120, uber_id: 48009 },
    ),
    (
        "windsweptWastesGroupDescriptor.expOrbD",
        UberIdentifier { uber_group: 20120, uber_id: 48829 },
    ),
    (
        "windsweptWastesGroupDescriptor.hintZoneB",
        UberIdentifier { uber_group: 20120, uber_id: 49950 },
    ),
    (
        "windsweptWastesGroupDescriptor.shardA",
        UberIdentifier { uber_group: 20120, uber_id: 49985 },
    ),
    (
        "windsweptWastesGroupDescriptor.energyContainer",
        UberIdentifier { uber_group: 20120, uber_id: 50026 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallA",
        UberIdentifier { uber_group: 20120, uber_id: 51985 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrb",
        UberIdentifier { uber_group: 20120, uber_id: 52812 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWall",
        UberIdentifier { uber_group: 20120, uber_id: 54936 },
    ),
    (
        "windsweptWastesGroupDescriptor.creepA",
        UberIdentifier { uber_group: 20120, uber_id: 55057 },
    ),
    (
        "windsweptWastesGroupDescriptor.wispSequencePlayedOut",
        UberIdentifier { uber_group: 20120, uber_id: 55196 },
    ),
    (
        "windsweptWastesGroupDescriptor.gorlekOre",
        UberIdentifier { uber_group: 20120, uber_id: 55303 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallE",
        UberIdentifier { uber_group: 20120, uber_id: 55388 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 20120, uber_id: 57133 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbB",
        UberIdentifier { uber_group: 20120, uber_id: 57781 },
    ),
    (
        "windsweptWastesGroupDescriptor.halfLifeCell",
        UberIdentifier { uber_group: 20120, uber_id: 59046 },
    ),
    (
        "windsweptWastesGroupDescriptor.breakableWallA",
        UberIdentifier { uber_group: 20120, uber_id: 59275 },
    ),
    (
        "windsweptWastesGroupDescriptor.doorClosingPlayed",
        UberIdentifier { uber_group: 20120, uber_id: 60953 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallD",
        UberIdentifier { uber_group: 20120, uber_id: 60960 },
    ),
    (
        "windsweptWastesGroupDescriptor.bombableWallF",
        UberIdentifier { uber_group: 20120, uber_id: 61572 },
    ),
    (
        "windsweptWastesGroupDescriptor.lifeCellA",
        UberIdentifier { uber_group: 20120, uber_id: 62264 },
    ),
    (
        "windsweptWastesGroupDescriptor.xpOrbF",
        UberIdentifier { uber_group: 20120, uber_id: 63310 },
    ),
    (
        "windsweptWastesGroupDescriptor.savePedestalUberState",
        UberIdentifier { uber_group: 20120, uber_id: 41398 },
    ),
    (
        "windsweptWastesGroupDescriptor.savePedestalUberState",
        UberIdentifier { uber_group: 20120, uber_id: 49994 },
    ),
    (
        "uiGroup.displayedSpiritWellFirstUseHint",
        UberIdentifier { uber_group: 20190, uber_id: 31212 },
    ),
    (
        "minesUberStateGroup.stompableFloorB",
        UberIdentifier { uber_group: 21194, uber_id: 6799 },
    ),
    (
        "minesUberStateGroup.collectableC",
        UberIdentifier { uber_group: 21194, uber_id: 7318 },
    ),
    (
        "minesUberStateGroup.spiritShardA",
        UberIdentifier { uber_group: 21194, uber_id: 11371 },
    ),
    (
        "minesUberStateGroup.collectableA",
        UberIdentifier { uber_group: 21194, uber_id: 16526 },
    ),
    (
        "minesUberStateGroup.grolDefeated",
        UberIdentifier { uber_group: 21194, uber_id: 18508 },
    ),
    (
        "minesUberStateGroup.collectableB",
        UberIdentifier { uber_group: 21194, uber_id: 26302 },
    ),
    (
        "minesUberStateGroup.xpOrbA",
        UberIdentifier { uber_group: 21194, uber_id: 27102 },
    ),
    (
        "minesUberStateGroup.memoriesPlayed",
        UberIdentifier { uber_group: 21194, uber_id: 29515 },
    ),
    (
        "minesUberStateGroup.crusherActivated",
        UberIdentifier { uber_group: 21194, uber_id: 29822 },
    ),
    (
        "minesUberStateGroup.elevatorDoorsBottom",
        UberIdentifier { uber_group: 21194, uber_id: 35345 },
    ),
    (
        "minesUberStateGroup.stompableFloorA",
        UberIdentifier { uber_group: 21194, uber_id: 36700 },
    ),
    (
        "minesUberStateGroup.grolCuredIntroDialoguePlayed",
        UberIdentifier { uber_group: 21194, uber_id: 38411 },
    ),
    (
        "minesUberStateGroup.elevatorDoorsTop",
        UberIdentifier { uber_group: 21194, uber_id: 43575 },
    ),
    (
        "minesUberStateGroup.breakableWall",
        UberIdentifier { uber_group: 21194, uber_id: 48792 },
    ),
    (
        "minesUberStateGroup.enemyDoor",
        UberIdentifier { uber_group: 21194, uber_id: 52416 },
    ),
    (
        "minesUberStateGroup.leverA",
        UberIdentifier { uber_group: 21194, uber_id: 63648 },
    ),
    (
        "minesUberStateGroup.savePedestalUberState",
        UberIdentifier { uber_group: 21194, uber_id: 685 },
    ),
    (
        "minesUberStateGroup.savePedestalUberState",
        UberIdentifier { uber_group: 21194, uber_id: 63334 },
    ),
    (
        "minesUberStateGroup.gateState",
        UberIdentifier { uber_group: 21194, uber_id: 17773 },
    ),
    (
        "minesUberStateGroup.minesElevatorUberState",
        UberIdentifier { uber_group: 21194, uber_id: 34225 },
    ),
    (
        "swampStateGroup.boneBridgeBroken",
        UberIdentifier { uber_group: 21786, uber_id: 808 },
    ),
    (
        "swampStateGroup.creepDoorD",
        UberIdentifier { uber_group: 21786, uber_id: 876 },
    ),
    (
        "swampStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 21786, uber_id: 2046 },
    ),
    (
        "swampStateGroup.laserPuzzleSolved",
        UberIdentifier { uber_group: 21786, uber_id: 2852 },
    ),
    (
        "swampStateGroup.enemyRoom",
        UberIdentifier { uber_group: 21786, uber_id: 2869 },
    ),
    (
        "swampStateGroup.xpOrbA",
        UberIdentifier { uber_group: 21786, uber_id: 6987 },
    ),
    (
        "swampStateGroup.stompableFloor",
        UberIdentifier { uber_group: 21786, uber_id: 6994 },
    ),
    (
        "swampStateGroup.halfEnergyCellA",
        UberIdentifier { uber_group: 21786, uber_id: 7152 },
    ),
    (
        "swampStateGroup.mediumExpA",
        UberIdentifier { uber_group: 21786, uber_id: 7709 },
    ),
    (
        "swampStateGroup.mediumExpC",
        UberIdentifier { uber_group: 21786, uber_id: 7871 },
    ),
    (
        "swampStateGroup.shardSlotA",
        UberIdentifier { uber_group: 21786, uber_id: 9270 },
    ),
    (
        "swampStateGroup.creepDoorB",
        UberIdentifier { uber_group: 21786, uber_id: 9402 },
    ),
    (
        "swampStateGroup.energyHalfCellA",
        UberIdentifier { uber_group: 21786, uber_id: 10295 },
    ),
    (
        "swampStateGroup.largeExpA",
        UberIdentifier { uber_group: 21786, uber_id: 10413 },
    ),
    (
        "swampStateGroup.attackableSwitchA",
        UberIdentifier { uber_group: 21786, uber_id: 10467 },
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier { uber_group: 21786, uber_id: 11343 },
    ),
    (
        "swampStateGroup.spiritShardPickupA",
        UberIdentifier { uber_group: 21786, uber_id: 12077 },
    ),
    (
        "swampStateGroup.xpOrbC",
        UberIdentifier { uber_group: 21786, uber_id: 16206 },
    ),
    (
        "swampStateGroup.energyHalfCell",
        UberIdentifier { uber_group: 21786, uber_id: 17920 },
    ),
    (
        "swampStateGroup.areaText",
        UberIdentifier { uber_group: 21786, uber_id: 17957 },
    ),
    (
        "swampStateGroup.shardSlotUpgradePlaceholder",
        UberIdentifier { uber_group: 21786, uber_id: 18109 },
    ),
    (
        "swampStateGroup.mediumExpB",
        UberIdentifier { uber_group: 21786, uber_id: 19679 },
    ),
    (
        "swampStateGroup.creepTreeC",
        UberIdentifier { uber_group: 21786, uber_id: 20144 },
    ),
    (
        "swampStateGroup.xpOrbB",
        UberIdentifier { uber_group: 21786, uber_id: 20160 },
    ),
    (
        "swampStateGroup.lifeCellA",
        UberIdentifier { uber_group: 21786, uber_id: 20194 },
    ),
    (
        "swampStateGroup.hintZone",
        UberIdentifier { uber_group: 21786, uber_id: 20615 },
    ),
    (
        "swampStateGroup.largeExpA",
        UberIdentifier { uber_group: 21786, uber_id: 21727 },
    ),
    (
        "swampStateGroup.keyStone",
        UberIdentifier { uber_group: 21786, uber_id: 22068 },
    ),
    (
        "swampStateGroup.playedOutKeystoneSequence",
        UberIdentifier { uber_group: 21786, uber_id: 22367 },
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier { uber_group: 21786, uber_id: 22570 },
    ),
    (
        "swampStateGroup.mediumExpA",
        UberIdentifier { uber_group: 21786, uber_id: 23154 },
    ),
    (
        "swampStateGroup.creepDoor",
        UberIdentifier { uber_group: 21786, uber_id: 23177 },
    ),
    (
        "swampStateGroup.nightCrawlerEscaped",
        UberIdentifier { uber_group: 21786, uber_id: 25095 },
    ),
    (
        "swampStateGroup.creepDoorA",
        UberIdentifier { uber_group: 21786, uber_id: 25147 },
    ),
    (
        "swampStateGroup.creepDoorE",
        UberIdentifier { uber_group: 21786, uber_id: 25291 },
    ),
    (
        "swampStateGroup.healthContainerA",
        UberIdentifier { uber_group: 21786, uber_id: 25761 },
    ),
    (
        "swampStateGroup.interactedWithOpher",
        UberIdentifier { uber_group: 21786, uber_id: 26462 },
    ),
    (
        "swampStateGroup.keyStoneA",
        UberIdentifier { uber_group: 21786, uber_id: 27433 },
    ),
    (
        "swampStateGroup.halfHealthCellA",
        UberIdentifier { uber_group: 21786, uber_id: 28908 },
    ),
    (
        "swampStateGroup.creepDoorB",
        UberIdentifier { uber_group: 21786, uber_id: 29636 },
    ),
    (
        "swampStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 21786, uber_id: 29892 },
    ),
    (
        "swampStateGroup.spiritShardA",
        UberIdentifier { uber_group: 21786, uber_id: 30305 },
    ),
    (
        "swampStateGroup.nightCrawlerChaseStarted",
        UberIdentifier { uber_group: 21786, uber_id: 30656 },
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier { uber_group: 21786, uber_id: 30928 },
    ),
    (
        "swampStateGroup.hintZone",
        UberIdentifier { uber_group: 21786, uber_id: 31430 },
    ),
    (
        "swampStateGroup.enemyDoorA",
        UberIdentifier { uber_group: 21786, uber_id: 32430 },
    ),
    (
        "swampStateGroup.creepTreeD",
        UberIdentifier { uber_group: 21786, uber_id: 32463 },
    ),
    (
        "swampStateGroup.creepDoor",
        UberIdentifier { uber_group: 21786, uber_id: 33430 },
    ),
    (
        "swampStateGroup.breakableWallB",
        UberIdentifier { uber_group: 21786, uber_id: 34008 },
    ),
    (
        "swampStateGroup.creepDoorA",
        UberIdentifier { uber_group: 21786, uber_id: 35166 },
    ),
    (
        "swampStateGroup.creepDoorC",
        UberIdentifier { uber_group: 21786, uber_id: 35260 },
    ),
    (
        "swampStateGroup.attackableSwitchC",
        UberIdentifier { uber_group: 21786, uber_id: 35350 },
    ),
    (
        "swampStateGroup.enemyArenaComplete",
        UberIdentifier { uber_group: 21786, uber_id: 35598 },
    ),
    (
        "swampStateGroup.breakableWallA",
        UberIdentifier { uber_group: 21786, uber_id: 35925 },
    ),
    (
        "swampStateGroup.keyStoneB",
        UberIdentifier { uber_group: 21786, uber_id: 37225 },
    ),
    (
        "swampStateGroup.creepTreeE",
        UberIdentifier { uber_group: 21786, uber_id: 37833 },
    ),
    (
        "swampStateGroup.hintZone",
        UberIdentifier { uber_group: 21786, uber_id: 38342 },
    ),
    (
        "swampStateGroup.bladeRitualFinished",
        UberIdentifier { uber_group: 21786, uber_id: 38475 },
    ),
    (
        "swampStateGroup.springCreep",
        UberIdentifier { uber_group: 21786, uber_id: 39804 },
    ),
    (
        "swampStateGroup.nightCrawlerDefeated",
        UberIdentifier { uber_group: 21786, uber_id: 40322 },
    ),
    (
        "swampStateGroup.breakableWall",
        UberIdentifier { uber_group: 21786, uber_id: 40424 },
    ),
    (
        "swampStateGroup.doorBState",
        UberIdentifier { uber_group: 21786, uber_id: 41817 },
    ),
    (
        "swampStateGroup.swampTorchIntroductionADoorWithTwoSlotsBooleanDescriptor",
        UberIdentifier { uber_group: 21786, uber_id: 42309 },
    ),
    (
        "swampStateGroup.watermillDiscovered",
        UberIdentifier { uber_group: 21786, uber_id: 43216 },
    ),
    (
        "swampStateGroup.xpOrbB",
        UberIdentifier { uber_group: 21786, uber_id: 43668 },
    ),
    (
        "swampStateGroup.energyContainerA",
        UberIdentifier { uber_group: 21786, uber_id: 44157 },
    ),
    (
        "swampStateGroup.secretWall",
        UberIdentifier { uber_group: 21786, uber_id: 44253 },
    ),
    (
        "swampStateGroup.creepTreeA",
        UberIdentifier { uber_group: 21786, uber_id: 44431 },
    ),
    (
        "swampStateGroup.attackableSwitchB",
        UberIdentifier { uber_group: 21786, uber_id: 45648 },
    ),
    (
        "swampStateGroup.nightcrawlerTeaseTimelinePlayed",
        UberIdentifier { uber_group: 21786, uber_id: 46536 },
    ),
    (
        "swampStateGroup.swampNightcrawlerCavernADoorWithTwoSlotsBooleanDescriptor",
        UberIdentifier { uber_group: 21786, uber_id: 47445 },
    ),
    (
        "swampStateGroup.torchHolded",
        UberIdentifier { uber_group: 21786, uber_id: 47458 },
    ),
    (
        "swampStateGroup.creepDoorB",
        UberIdentifier { uber_group: 21786, uber_id: 47644 },
    ),
    (
        "swampStateGroup.finishedIntroTop",
        UberIdentifier { uber_group: 21786, uber_id: 48748 },
    ),
    (
        "swampStateGroup.smallExpA",
        UberIdentifier { uber_group: 21786, uber_id: 49485 },
    ),
    (
        "swampStateGroup.mediumExpA",
        UberIdentifier { uber_group: 21786, uber_id: 50255 },
    ),
    (
        "swampStateGroup.swampWalljumpChallengeBKeystoneACollected",
        UberIdentifier { uber_group: 21786, uber_id: 50281 },
    ),
    (
        "swampStateGroup.leverA",
        UberIdentifier { uber_group: 21786, uber_id: 50432 },
    ),
    (
        "swampStateGroup.leverAndDoor",
        UberIdentifier { uber_group: 21786, uber_id: 50453 },
    ),
    (
        "swampStateGroup.doorAState",
        UberIdentifier { uber_group: 21786, uber_id: 50691 },
    ),
    (
        "swampStateGroup.nightcrawlerBridgeBrokenA",
        UberIdentifier { uber_group: 21786, uber_id: 50994 },
    ),
    (
        "swampStateGroup.powlTeaseTriggered",
        UberIdentifier { uber_group: 21786, uber_id: 51018 },
    ),
    (
        "swampStateGroup.smallExpA",
        UberIdentifier { uber_group: 21786, uber_id: 52026 },
    ),
    (
        "swampStateGroup.leverGateinkwaterMarsh",
        UberIdentifier { uber_group: 21786, uber_id: 52815 },
    ),
    (
        "swampStateGroup.creepDoorA",
        UberIdentifier { uber_group: 21786, uber_id: 53932 },
    ),
    (
        "swampStateGroup.elevatorDown",
        UberIdentifier { uber_group: 21786, uber_id: 55881 },
    ),
    (
        "swampStateGroup.gateUberState",
        UberIdentifier { uber_group: 21786, uber_id: 58612 },
    ),
    (
        "swampStateGroup.expOrb",
        UberIdentifier { uber_group: 21786, uber_id: 59513 },
    ),
    (
        "swampStateGroup.breakableBridgeBroken",
        UberIdentifier { uber_group: 21786, uber_id: 59922 },
    ),
    (
        "swampStateGroup.doorWithTwoSlots",
        UberIdentifier { uber_group: 21786, uber_id: 59990 },
    ),
    (
        "swampStateGroup.healthContainerA",
        UberIdentifier { uber_group: 21786, uber_id: 60210 },
    ),
    (
        "swampStateGroup.doorFourSlots",
        UberIdentifier { uber_group: 21786, uber_id: 60616 },
    ),
    (
        "swampStateGroup.ottersLeadToSpiritBlade",
        UberIdentifier { uber_group: 21786, uber_id: 61644 },
    ),
    (
        "swampStateGroup.halfEnergyCellA",
        UberIdentifier { uber_group: 21786, uber_id: 61706 },
    ),
    (
        "swampStateGroup.stompableFloor",
        UberIdentifier { uber_group: 21786, uber_id: 61900 },
    ),
    (
        "swampStateGroup.xpOrbA",
        UberIdentifier { uber_group: 21786, uber_id: 63072 },
    ),
    (
        "swampStateGroup.spiritShardA",
        UberIdentifier { uber_group: 21786, uber_id: 63545 },
    ),
    (
        "swampStateGroup.swampWalljumpChallengeBKeystoneBCollected",
        UberIdentifier { uber_group: 21786, uber_id: 64677 },
    ),
    (
        "swampStateGroup.creepTreeB",
        UberIdentifier { uber_group: 21786, uber_id: 65235 },
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier { uber_group: 21786, uber_id: 3714 },
    ),
    (
        "swampStateGroup.savePedestalSwampIntroTop",
        UberIdentifier { uber_group: 21786, uber_id: 10185 },
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier { uber_group: 21786, uber_id: 12914 },
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier { uber_group: 21786, uber_id: 38720 },
    ),
    (
        "swampStateGroup.savePedestalUberState",
        UberIdentifier { uber_group: 21786, uber_id: 38941 },
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier { uber_group: 21786, uber_id: 50820 },
    ),
    (
        "swampStateGroup.savePedestal",
        UberIdentifier { uber_group: 21786, uber_id: 56901 },
    ),
    (
        "swampStateGroup.pushBlockState",
        UberIdentifier { uber_group: 21786, uber_id: 22091 },
    ),
    (
        "pickupsGroup.hollowEnergyShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 897 },
    ),
    (
        "pickupsGroup.spiritPowerShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 986 },
    ),
    (
        "pickupsGroup.recklessShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 9864 },
    ),
    (
        "pickupsGroup.ultraLeashShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 12104 },
    ),
    (
        "pickupsGroup.energyCell",
        UberIdentifier { uber_group: 23987, uber_id: 12746 },
    ),
    (
        "pickupsGroup.focusShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 14014 },
    ),
    (
        "pickupsGroup.secretShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 14832 },
    ),
    (
        "pickupsGroup.untouchableShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 19630 },
    ),
    (
        "pickupsGroup.spiritMagnetShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 20915 },
    ),
    (
        "pickupsGroup.chainLightningPickup",
        UberIdentifier { uber_group: 23987, uber_id: 23015 },
    ),
    (
        "pickupsGroup.recycleShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 25183 },
    ),
    (
        "pickupsGroup.ultraBashShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 25996 },
    ),
    (
        "pickupsGroup.glueShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 27134 },
    ),
    (
        "pickupsGroup.counterstrikeShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 31426 },
    ),
    (
        "pickupsGroup.fractureShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 36359 },
    ),
    (
        "pickupsGroup.energyEfficiencyShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 46461 },
    ),
    (
        "pickupsGroup.aggressorShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 48605 },
    ),
    (
        "pickupsGroup.lastResortShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 50364 },
    ),
    (
        "pickupsGroup.bloodPactShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 50415 },
    ),
    (
        "pickupsGroup.vitalityLuckShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 53934 },
    ),
    (
        "pickupsGroup.barrierShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 59173 },
    ),
    (
        "pickupsGroup.frenzyShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 61017 },
    ),
    (
        "pickupsGroup.splinterShardPickup",
        UberIdentifier { uber_group: 23987, uber_id: 62973 },
    ),
    (
        "howlsOriginGroup.secretWallA",
        UberIdentifier { uber_group: 24922, uber_id: 2524 },
    ),
    (
        "howlsOriginGroup.expOrbA",
        UberIdentifier { uber_group: 24922, uber_id: 8568 },
    ),
    (
        "howlsOriginGroup.bellPuzzleSolved",
        UberIdentifier { uber_group: 24922, uber_id: 13349 },
    ),
    (
        "howlsOriginGroup.xpOrbA",
        UberIdentifier { uber_group: 24922, uber_id: 13921 },
    ),
    (
        "howlsOriginGroup.shardSlotUpgradePlaceholder",
        UberIdentifier { uber_group: 24922, uber_id: 13993 },
    ),
    (
        "howlsOriginGroup.portalsLifted",
        UberIdentifier { uber_group: 24922, uber_id: 16603 },
    ),
    (
        "howlsOriginGroup.smallExpA",
        UberIdentifier { uber_group: 24922, uber_id: 32076 },
    ),
    (
        "howlsOriginGroup.keystoneB",
        UberIdentifier { uber_group: 24922, uber_id: 33535 },
    ),
    (
        "howlsOriginGroup.keystoneA",
        UberIdentifier { uber_group: 24922, uber_id: 34250 },
    ),
    (
        "howlsOriginGroup.shrineArena",
        UberIdentifier { uber_group: 24922, uber_id: 45011 },
    ),
    (
        "howlsOriginGroup.interactedWithTokk",
        UberIdentifier { uber_group: 24922, uber_id: 45740 },
    ),
    (
        "howlsOriginGroup.spiritShard",
        UberIdentifier { uber_group: 24922, uber_id: 46311 },
    ),
    (
        "howlsOriginGroup.keystoneA",
        UberIdentifier { uber_group: 24922, uber_id: 47244 },
    ),
    (
        "howlsOriginGroup.breakableWallA",
        UberIdentifier { uber_group: 24922, uber_id: 50740 },
    ),
    (
        "howlsOriginGroup.bellPuzzleBSolved",
        UberIdentifier { uber_group: 24922, uber_id: 59146 },
    ),
    (
        "howlsOriginGroup.keystoneA",
        UberIdentifier { uber_group: 24922, uber_id: 60358 },
    ),
    (
        "howlsOriginGroup.largeExpA",
        UberIdentifier { uber_group: 24922, uber_id: 62138 },
    ),
    (
        "howlsOriginGroup.howlOriginEntranceSavePedestal",
        UberIdentifier { uber_group: 24922, uber_id: 42531 },
    ),
    (
        "convertedSetupsGymGroup.blowableFlameToggle",
        UberIdentifier { uber_group: 26019, uber_id: 971 },
    ),
    (
        "convertedSetupsGymGroup.secretWallA",
        UberIdentifier { uber_group: 26019, uber_id: 1274 },
    ),
    (
        "convertedSetupsGymGroup.horizontalDoorState",
        UberIdentifier { uber_group: 26019, uber_id: 4052 },
    ),
    (
        "convertedSetupsGymGroup.creepDoorD",
        UberIdentifier { uber_group: 26019, uber_id: 4231 },
    ),
    (
        "convertedSetupsGymGroup.secretWall",
        UberIdentifier { uber_group: 26019, uber_id: 5259 },
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsLever",
        UberIdentifier { uber_group: 26019, uber_id: 6406 },
    ),
    (
        "convertedSetupsGymGroup.creepD",
        UberIdentifier { uber_group: 26019, uber_id: 7636 },
    ),
    (
        "convertedSetupsGymGroup.mediumExpOrb",
        UberIdentifier { uber_group: 26019, uber_id: 10086 },
    ),
    (
        "convertedSetupsGymGroup.snowPileB",
        UberIdentifier { uber_group: 26019, uber_id: 11133 },
    ),
    (
        "convertedSetupsGymGroup.elevatorLever",
        UberIdentifier { uber_group: 26019, uber_id: 11592 },
    ),
    (
        "convertedSetupsGymGroup.stompableFloor",
        UberIdentifier { uber_group: 26019, uber_id: 12371 },
    ),
    (
        "convertedSetupsGymGroup.creepDoorA",
        UberIdentifier { uber_group: 26019, uber_id: 13586 },
    ),
    (
        "convertedSetupsGymGroup.desertBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 14277 },
    ),
    (
        "convertedSetupsGymGroup.creepC",
        UberIdentifier { uber_group: 26019, uber_id: 15381 },
    ),
    (
        "convertedSetupsGymGroup.stompableFloorB",
        UberIdentifier { uber_group: 26019, uber_id: 18425 },
    ),
    (
        "convertedSetupsGymGroup.cordycepsBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 21522 },
    ),
    (
        "convertedSetupsGymGroup.watermillEnemyDoor",
        UberIdentifier { uber_group: 26019, uber_id: 23282 },
    ),
    (
        "convertedSetupsGymGroup.leverAndDoor",
        UberIdentifier { uber_group: 26019, uber_id: 23382 },
    ),
    (
        "convertedSetupsGymGroup.weepingRidgeBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 25103 },
    ),
    (
        "convertedSetupsGymGroup.petrifiedForestBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 26714 },
    ),
    (
        "convertedSetupsGymGroup.leafPile",
        UberIdentifier { uber_group: 26019, uber_id: 27176 },
    ),
    (
        "convertedSetupsGymGroup.enemyDoor",
        UberIdentifier { uber_group: 26019, uber_id: 28367 },
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 28678 },
    ),
    (
        "convertedSetupsGymGroup.enemyDoorA",
        UberIdentifier { uber_group: 26019, uber_id: 29970 },
    ),
    (
        "convertedSetupsGymGroup.keyStoneYesCheckpoint",
        UberIdentifier { uber_group: 26019, uber_id: 30549 },
    ),
    (
        "convertedSetupsGymGroup.watermillBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 32221 },
    ),
    (
        "convertedSetupsGymGroup.watermillBreakableWallUnderwater",
        UberIdentifier { uber_group: 26019, uber_id: 33339 },
    ),
    (
        "convertedSetupsGymGroup.lagoonEnemyDoor",
        UberIdentifier { uber_group: 26019, uber_id: 33392 },
    ),
    (
        "convertedSetupsGymGroup.desertRuinsBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 33510 },
    ),
    (
        "convertedSetupsGymGroup.energyContainer",
        UberIdentifier { uber_group: 26019, uber_id: 34752 },
    ),
    (
        "convertedSetupsGymGroup.creepDoorC",
        UberIdentifier { uber_group: 26019, uber_id: 34818 },
    ),
    (
        "convertedSetupsGymGroup.snowPileA",
        UberIdentifier { uber_group: 26019, uber_id: 35001 },
    ),
    (
        "convertedSetupsGymGroup.classicShootableCreepDoor",
        UberIdentifier { uber_group: 26019, uber_id: 37244 },
    ),
    (
        "convertedSetupsGymGroup.snowPile",
        UberIdentifier { uber_group: 26019, uber_id: 38710 },
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsBreakableWallLock",
        UberIdentifier { uber_group: 26019, uber_id: 38743 },
    ),
    (
        "convertedSetupsGymGroup.keyStoneNoCheckpoint",
        UberIdentifier { uber_group: 26019, uber_id: 38761 },
    ),
    (
        "convertedSetupsGymGroup.kwolokCavernsBreakableWall2",
        UberIdentifier { uber_group: 26019, uber_id: 40296 },
    ),
    (
        "convertedSetupsGymGroup.winterForestBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 40553 },
    ),
    (
        "convertedSetupsGymGroup.creepDoorB",
        UberIdentifier { uber_group: 26019, uber_id: 44556 },
    ),
    (
        "convertedSetupsGymGroup.creepE",
        UberIdentifier { uber_group: 26019, uber_id: 47055 },
    ),
    (
        "convertedSetupsGymGroup.secretWallWithLock",
        UberIdentifier { uber_group: 26019, uber_id: 47874 },
    ),
    (
        "convertedSetupsGymGroup.skillPointOrb",
        UberIdentifier { uber_group: 26019, uber_id: 49127 },
    ),
    (
        "convertedSetupsGymGroup.springCreep",
        UberIdentifier { uber_group: 26019, uber_id: 51496 },
    ),
    (
        "convertedSetupsGymGroup.enemyDoor",
        UberIdentifier { uber_group: 26019, uber_id: 52684 },
    ),
    (
        "convertedSetupsGymGroup.spiritShardPickup",
        UberIdentifier { uber_group: 26019, uber_id: 53374 },
    ),
    (
        "convertedSetupsGymGroup.drillZone",
        UberIdentifier { uber_group: 26019, uber_id: 53543 },
    ),
    (
        "convertedSetupsGymGroup.snowPileA",
        UberIdentifier { uber_group: 26019, uber_id: 54405 },
    ),
    (
        "convertedSetupsGymGroup.fourSlotDoor",
        UberIdentifier { uber_group: 26019, uber_id: 54578 },
    ),
    (
        "convertedSetupsGymGroup.minesBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 58058 },
    ),
    (
        "convertedSetupsGymGroup.creepB",
        UberIdentifier { uber_group: 26019, uber_id: 58116 },
    ),
    (
        "convertedSetupsGymGroup.creepA",
        UberIdentifier { uber_group: 26019, uber_id: 60202 },
    ),
    (
        "convertedSetupsGymGroup.creepDoorE",
        UberIdentifier { uber_group: 26019, uber_id: 60454 },
    ),
    (
        "convertedSetupsGymGroup.lagoonBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 62800 },
    ),
    (
        "convertedSetupsGymGroup.twoSlotDoor",
        UberIdentifier { uber_group: 26019, uber_id: 62962 },
    ),
    (
        "convertedSetupsGymGroup.swampBreakableWall",
        UberIdentifier { uber_group: 26019, uber_id: 63056 },
    ),
    (
        "convertedSetupsGymGroup.largeExpOrb",
        UberIdentifier { uber_group: 26019, uber_id: 64001 },
    ),
    (
        "convertedSetupsGymGroup.smallExpOrb",
        UberIdentifier { uber_group: 26019, uber_id: 64961 },
    ),
    (
        "convertedSetupsGymGroup.willowsEndSecretWall",
        UberIdentifier { uber_group: 26019, uber_id: 65139 },
    ),
    (
        "convertedSetupsGymGroup.xpOrbB",
        UberIdentifier { uber_group: 26019, uber_id: 65172 },
    ),
    (
        "convertedSetupsGymGroup.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 26019, uber_id: 13498 },
    ),
    (
        "winterForestGroupDescriptor.breakableFloorB",
        UberIdentifier { uber_group: 28287, uber_id: 3938 },
    ),
    (
        "winterForestGroupDescriptor.springBlossomTimelinePlayed",
        UberIdentifier { uber_group: 28287, uber_id: 6764 },
    ),
    (
        "winterForestGroupDescriptor.boxA",
        UberIdentifier { uber_group: 28287, uber_id: 10460 },
    ),
    (
        "winterForestGroupDescriptor.breakableFloor",
        UberIdentifier { uber_group: 28287, uber_id: 11124 },
    ),
    (
        "winterForestGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 28287, uber_id: 12866 },
    ),
    (
        "winterForestGroupDescriptor.breakableRocksA",
        UberIdentifier { uber_group: 28287, uber_id: 13168 },
    ),
    (
        "winterForestGroupDescriptor.mediumExpC",
        UberIdentifier { uber_group: 28287, uber_id: 15252 },
    ),
    (
        "winterForestGroupDescriptor.thawStateDescriptor",
        UberIdentifier { uber_group: 28287, uber_id: 16339 },
    ),
    (
        "winterForestGroupDescriptor.pressurePlatePuzzleSolved",
        UberIdentifier { uber_group: 28287, uber_id: 22713 },
    ),
    (
        "winterForestGroupDescriptor.breakableWallA",
        UberIdentifier { uber_group: 28287, uber_id: 24327 },
    ),
    (
        "winterForestGroupDescriptor.breakableFloorA",
        UberIdentifier { uber_group: 28287, uber_id: 26844 },
    ),
    (
        "winterForestGroupDescriptor.mediumExpB",
        UberIdentifier { uber_group: 28287, uber_id: 28525 },
    ),
    (
        "winterForestGroupDescriptor.secretWallThaw",
        UberIdentifier { uber_group: 28287, uber_id: 30157 },
    ),
    (
        "winterForestGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 28287, uber_id: 32414 },
    ),
    (
        "winterForestGroupDescriptor.hammerWall",
        UberIdentifier { uber_group: 28287, uber_id: 40607 },
    ),
    (
        "winterForestGroupDescriptor.breakableWallC",
        UberIdentifier { uber_group: 28287, uber_id: 43506 },
    ),
    (
        "winterForestGroupDescriptor.breakableWallB",
        UberIdentifier { uber_group: 28287, uber_id: 51721 },
    ),
    (
        "winterForestGroupDescriptor.boxB",
        UberIdentifier { uber_group: 28287, uber_id: 52043 },
    ),
    (
        "winterForestGroupDescriptor.leafPileA",
        UberIdentifier { uber_group: 28287, uber_id: 55131 },
    ),
    (
        "winterForestGroupDescriptor.spiritShardA",
        UberIdentifier { uber_group: 28287, uber_id: 55482 },
    ),
    (
        "winterForestGroupDescriptor.stompableFloor",
        UberIdentifier { uber_group: 28287, uber_id: 57792 },
    ),
    (
        "winterForestGroupDescriptor.leafPileB",
        UberIdentifier { uber_group: 28287, uber_id: 61490 },
    ),
    (
        "winterForestGroupDescriptor.breakableRocksB",
        UberIdentifier { uber_group: 28287, uber_id: 62332 },
    ),
    (
        "winterForestGroupDescriptor.savePedestalUberState",
        UberIdentifier { uber_group: 28287, uber_id: 64528 },
    ),
    (
        "baursReachGroup.keystoneB",
        UberIdentifier { uber_group: 28895, uber_id: 1053 },
    ),
    (
        "baursReachGroup.powlTeaseTriggered",
        UberIdentifier { uber_group: 28895, uber_id: 2108 },
    ),
    (
        "baursReachGroup.largeExpOrb",
        UberIdentifier { uber_group: 28895, uber_id: 2129 },
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 2462 },
    ),
    (
        "baursReachGroup.stompableFloorA",
        UberIdentifier { uber_group: 28895, uber_id: 2896 },
    ),
    (
        "baursReachGroup.breakableWallB",
        UberIdentifier { uber_group: 28895, uber_id: 2931 },
    ),
    (
        "baursReachGroup.mediumExpOrb",
        UberIdentifier { uber_group: 28895, uber_id: 3777 },
    ),
    (
        "baursReachGroup.doorWithFourSlots",
        UberIdentifier { uber_group: 28895, uber_id: 4290 },
    ),
    (
        "baursReachGroup.xpOrbF",
        UberIdentifier { uber_group: 28895, uber_id: 4301 },
    ),
    (
        "baursReachGroup.frozenMokiInteracted",
        UberIdentifier { uber_group: 28895, uber_id: 7152 },
    ),
    (
        "baursReachGroup.smallExpB",
        UberIdentifier { uber_group: 28895, uber_id: 7597 },
    ),
    (
        "baursReachGroup.breakableRocksA",
        UberIdentifier { uber_group: 28895, uber_id: 7616 },
    ),
    (
        "baursReachGroup.breakableRocksK",
        UberIdentifier { uber_group: 28895, uber_id: 7703 },
    ),
    (
        "baursReachGroup.stompableFloorA",
        UberIdentifier { uber_group: 28895, uber_id: 8664 },
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 8834 },
    ),
    (
        "baursReachGroup.breakableWallB",
        UberIdentifier { uber_group: 28895, uber_id: 8934 },
    ),
    (
        "baursReachGroup.expOrbD",
        UberIdentifier { uber_group: 28895, uber_id: 9321 },
    ),
    (
        "baursReachGroup.keystoneC",
        UberIdentifier { uber_group: 28895, uber_id: 9949 },
    ),
    (
        "baursReachGroup.keystoneC",
        UberIdentifier { uber_group: 28895, uber_id: 10823 },
    ),
    (
        "baursReachGroup.energyHalfCell",
        UberIdentifier { uber_group: 28895, uber_id: 10840 },
    ),
    (
        "baursReachGroup.breakableRocksG",
        UberIdentifier { uber_group: 28895, uber_id: 11936 },
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 12140 },
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier { uber_group: 28895, uber_id: 14264 },
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier { uber_group: 28895, uber_id: 17510 },
    ),
    (
        "baursReachGroup.afterMoraDeathRetalkPlayed",
        UberIdentifier { uber_group: 28895, uber_id: 17914 },
    ),
    (
        "baursReachGroup.keystoneD",
        UberIdentifier { uber_group: 28895, uber_id: 18358 },
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier { uber_group: 28895, uber_id: 19041 },
    ),
    (
        "baursReachGroup.expOrbE",
        UberIdentifier { uber_group: 28895, uber_id: 19077 },
    ),
    (
        "baursReachGroup.breakableRockWall",
        UberIdentifier { uber_group: 28895, uber_id: 20731 },
    ),
    (
        "baursReachGroup.stompableGroundA",
        UberIdentifier { uber_group: 28895, uber_id: 22127 },
    ),
    (
        "baursReachGroup.keystoneA",
        UberIdentifier { uber_group: 28895, uber_id: 22382 },
    ),
    (
        "baursReachGroup.placedCoal",
        UberIdentifier { uber_group: 28895, uber_id: 22695 },
    ),
    (
        "baursReachGroup.winterForestBonfire",
        UberIdentifier { uber_group: 28895, uber_id: 22758 },
    ),
    (
        "baursReachGroup.mediumExpOrb",
        UberIdentifier { uber_group: 28895, uber_id: 22761 },
    ),
    (
        "baursReachGroup.expOrbC",
        UberIdentifier { uber_group: 28895, uber_id: 22959 },
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 23605 },
    ),
    (
        "baursReachGroup.breakableRocksH",
        UberIdentifier { uber_group: 28895, uber_id: 23678 },
    ),
    (
        "baursReachGroup.gorlekOreA",
        UberIdentifier { uber_group: 28895, uber_id: 23795 },
    ),
    (
        "baursReachGroup.smallXPOrbB",
        UberIdentifier { uber_group: 28895, uber_id: 24533 },
    ),
    (
        "baursReachGroup.afterKwolokDeathRetalkPlayed",
        UberIdentifier { uber_group: 28895, uber_id: 25315 },
    ),
    (
        "baursReachGroup.wispRewardPickup",
        UberIdentifier { uber_group: 28895, uber_id: 25522 },
    ),
    (
        "baursReachGroup.energyUpgrade",
        UberIdentifier { uber_group: 28895, uber_id: 27476 },
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier { uber_group: 28895, uber_id: 27787 },
    ),
    (
        "baursReachGroup.seedPodBroken",
        UberIdentifier { uber_group: 28895, uber_id: 28059 },
    ),
    (
        "baursReachGroup.keystoneA",
        UberIdentifier { uber_group: 28895, uber_id: 29898 },
    ),
    (
        "baursReachGroup.grenadeLanternsHint",
        UberIdentifier { uber_group: 28895, uber_id: 30189 },
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier { uber_group: 28895, uber_id: 30566 },
    ),
    (
        "baursReachGroup.breakableWall",
        UberIdentifier { uber_group: 28895, uber_id: 30794 },
    ),
    (
        "baursReachGroup.fallingBranch",
        UberIdentifier { uber_group: 28895, uber_id: 31575 },
    ),
    (
        "baursReachGroup.areaTextZone",
        UberIdentifier { uber_group: 28895, uber_id: 32092 },
    ),
    (
        "baursReachGroup.creepDoor",
        UberIdentifier { uber_group: 28895, uber_id: 32340 },
    ),
    (
        "baursReachGroup.closingGate",
        UberIdentifier { uber_group: 28895, uber_id: 32443 },
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier { uber_group: 28895, uber_id: 33337 },
    ),
    (
        "baursReachGroup.xpOrbE",
        UberIdentifier { uber_group: 28895, uber_id: 33846 },
    ),
    (
        "baursReachGroup.breakableWall",
        UberIdentifier { uber_group: 28895, uber_id: 34098 },
    ),
    (
        "baursReachGroup.breakableRocksJ",
        UberIdentifier { uber_group: 28895, uber_id: 34461 },
    ),
    (
        "baursReachGroup.healthCellA",
        UberIdentifier { uber_group: 28895, uber_id: 34534 },
    ),
    (
        "baursReachGroup.smallXPOrbB",
        UberIdentifier { uber_group: 28895, uber_id: 35045 },
    ),
    (
        "baursReachGroup.memoriesPlayedOut",
        UberIdentifier { uber_group: 28895, uber_id: 35436 },
    ),
    (
        "baursReachGroup.hintZoneA",
        UberIdentifier { uber_group: 28895, uber_id: 35874 },
    ),
    (
        "baursReachGroup.xpOrbC",
        UberIdentifier { uber_group: 28895, uber_id: 36231 },
    ),
    (
        "baursReachGroup.smallExpA",
        UberIdentifier { uber_group: 28895, uber_id: 36378 },
    ),
    (
        "baursReachGroup.secretWallA",
        UberIdentifier { uber_group: 28895, uber_id: 36649 },
    ),
    (
        "baursReachGroup.grenadeSwitchA",
        UberIdentifier { uber_group: 28895, uber_id: 37287 },
    ),
    (
        "baursReachGroup.keystoneB",
        UberIdentifier { uber_group: 28895, uber_id: 37444 },
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier { uber_group: 28895, uber_id: 38049 },
    ),
    (
        "baursReachGroup.breakableRocksE",
        UberIdentifier { uber_group: 28895, uber_id: 38120 },
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 38143 },
    ),
    (
        "baursReachGroup.breakableRocksF",
        UberIdentifier { uber_group: 28895, uber_id: 38525 },
    ),
    (
        "baursReachGroup.gorlekOreA",
        UberIdentifier { uber_group: 28895, uber_id: 39291 },
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier { uber_group: 28895, uber_id: 40089 },
    ),
    (
        "baursReachGroup.xpOrbC",
        UberIdentifier { uber_group: 28895, uber_id: 40242 },
    ),
    (
        "baursReachGroup.healthCellA",
        UberIdentifier { uber_group: 28895, uber_id: 40744 },
    ),
    (
        "baursReachGroup.afterAvalancheRetalkPlayed",
        UberIdentifier { uber_group: 28895, uber_id: 41299 },
    ),
    (
        "baursReachGroup.hintZone",
        UberIdentifier { uber_group: 28895, uber_id: 41777 },
    ),
    (
        "baursReachGroup.enemyArenaComplete",
        UberIdentifier { uber_group: 28895, uber_id: 42209 },
    ),
    (
        "baursReachGroup.firePedestal",
        UberIdentifier { uber_group: 28895, uber_id: 43977 },
    ),
    (
        "baursReachGroup.xpOrbE",
        UberIdentifier { uber_group: 28895, uber_id: 45066 },
    ),
    (
        "baursReachGroup.largeXPOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 45337 },
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier { uber_group: 28895, uber_id: 46293 },
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier { uber_group: 28895, uber_id: 46404 },
    ),
    (
        "baursReachGroup.xpOrbB",
        UberIdentifier { uber_group: 28895, uber_id: 46711 },
    ),
    (
        "baursReachGroup.breakableRocksI",
        UberIdentifier { uber_group: 28895, uber_id: 46875 },
    ),
    (
        "baursReachGroup.orePlaceholder",
        UberIdentifier { uber_group: 28895, uber_id: 47529 },
    ),
    (
        "baursReachGroup.creepA",
        UberIdentifier { uber_group: 28895, uber_id: 48186 },
    ),
    (
        "baursReachGroup.grenadeSwitchA",
        UberIdentifier { uber_group: 28895, uber_id: 48757 },
    ),
    (
        "baursReachGroup.breakyBridge",
        UberIdentifier { uber_group: 28895, uber_id: 49329 },
    ),
    (
        "baursReachGroup.keystoneGate",
        UberIdentifier { uber_group: 28895, uber_id: 49900 },
    ),
    (
        "baursReachGroup.breakableRocksB",
        UberIdentifier { uber_group: 28895, uber_id: 49997 },
    ),
    (
        "baursReachGroup.keystoneD",
        UberIdentifier { uber_group: 28895, uber_id: 50368 },
    ),
    (
        "baursReachGroup.blowableFlameB",
        UberIdentifier { uber_group: 28895, uber_id: 51471 },
    ),
    (
        "baursReachGroup.healthHalfContainer",
        UberIdentifier { uber_group: 28895, uber_id: 51853 },
    ),
    (
        "baursReachGroup.kindledFire",
        UberIdentifier { uber_group: 28895, uber_id: 52440 },
    ),
    (
        "baursReachGroup.seenLoremasterMenu",
        UberIdentifier { uber_group: 28895, uber_id: 53166 },
    ),
    (
        "baursReachGroup.xpOrbC",
        UberIdentifier { uber_group: 28895, uber_id: 53283 },
    ),
    (
        "baursReachGroup.smallXPOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 54373 },
    ),
    (
        "baursReachGroup.smallXPOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 55384 },
    ),
    (
        "baursReachGroup.breakableWallA",
        UberIdentifier { uber_group: 28895, uber_id: 56062 },
    ),
    (
        "baursReachGroup.secretWallBaur",
        UberIdentifier { uber_group: 28895, uber_id: 57743 },
    ),
    (
        "baursReachGroup.doorOpened",
        UberIdentifier { uber_group: 28895, uber_id: 58337 },
    ),
    (
        "baursReachGroup.gorlekOreA",
        UberIdentifier { uber_group: 28895, uber_id: 58675 },
    ),
    (
        "baursReachGroup.expOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 58848 },
    ),
    (
        "baursReachGroup.talkedToSleepingBaur",
        UberIdentifier { uber_group: 28895, uber_id: 59287 },
    ),
    (
        "baursReachGroup.grenadeSwitchA",
        UberIdentifier { uber_group: 28895, uber_id: 59394 },
    ),
    (
        "baursReachGroup.interactedWithCampfire",
        UberIdentifier { uber_group: 28895, uber_id: 59955 },
    ),
    (
        "baursReachGroup.xpOrbA",
        UberIdentifier { uber_group: 28895, uber_id: 61536 },
    ),
    (
        "baursReachGroup.firePedestalBooleanUberState",
        UberIdentifier { uber_group: 28895, uber_id: 61789 },
    ),
    (
        "baursReachGroup.frozenMokiIceBroken",
        UberIdentifier { uber_group: 28895, uber_id: 61852 },
    ),
    (
        "baursReachGroup.breakableRocksC",
        UberIdentifier { uber_group: 28895, uber_id: 61896 },
    ),
    (
        "baursReachGroup.afterGoldenSeinRetalkPlayed",
        UberIdentifier { uber_group: 28895, uber_id: 61976 },
    ),
    (
        "baursReachGroup.leverSetup",
        UberIdentifier { uber_group: 28895, uber_id: 62198 },
    ),
    (
        "baursReachGroup.breakableRocksD",
        UberIdentifier { uber_group: 28895, uber_id: 62643 },
    ),
    (
        "baursReachGroup.orePickup",
        UberIdentifier { uber_group: 28895, uber_id: 64226 },
    ),
    (
        "baursReachGroup.blowableFlameA",
        UberIdentifier { uber_group: 28895, uber_id: 64742 },
    ),
    (
        "baursReachGroup.mediumExpA",
        UberIdentifier { uber_group: 28895, uber_id: 65235 },
    ),
    (
        "baursReachGroup.savePedestalUberState",
        UberIdentifier { uber_group: 28895, uber_id: 18910 },
    ),
    (
        "baursReachGroup.savePedestalUberState",
        UberIdentifier { uber_group: 28895, uber_id: 54235 },
    ),
    (
        "baursReachGroup.interactedWithTokk",
        UberIdentifier { uber_group: 28895, uber_id: 13636 },
    ),
    (
        "baursReachGroup.baurNpcState",
        UberIdentifier { uber_group: 28895, uber_id: 29098 },
    ),
    (
        "baursReachGroup.mokiNpcState",
        UberIdentifier { uber_group: 28895, uber_id: 12170 },
    ),
    (
        "weepingRidgeElevatorFightGroup.willowsEndGateOpened",
        UberIdentifier { uber_group: 31136, uber_id: 3441 },
    ),
    (
        "weepingRidgeElevatorFightGroup.areaText",
        UberIdentifier { uber_group: 31136, uber_id: 59099 },
    ),
    (
        "achievementsGroup.spiritBladeCollected",
        UberIdentifier { uber_group: 33399, uber_id: 17893 },
    ),
    (
        "achievementsGroup.gotHitBySpider",
        UberIdentifier { uber_group: 33399, uber_id: 28382 },
    ),
    (
        "achievementsGroup.shardEverEquipped",
        UberIdentifier { uber_group: 33399, uber_id: 34522 },
    ),
    (
        "achievementsGroup.spiritLightEverSpent",
        UberIdentifier { uber_group: 33399, uber_id: 50709 },
    ),
    (
        "achievementsGroup.poisonousWaterTouched",
        UberIdentifier { uber_group: 33399, uber_id: 58955 },
    ),
    (
        "achievementsGroup.enemiesKilledByHazards",
        UberIdentifier { uber_group: 33399, uber_id: 17398 },
    ),
    (
        "achievementsGroup.spiritLightGainedCounter",
        UberIdentifier { uber_group: 33399, uber_id: 36285 },
    ),
    (
        "achievementsGroup.energyContainersCounter",
        UberIdentifier { uber_group: 33399, uber_id: 41928 },
    ),
    (
        "achievementsGroup.healthContainersCounter",
        UberIdentifier { uber_group: 33399, uber_id: 52378 },
    ),
    (
        "achievementsGroup.collectablesCounter",
        UberIdentifier { uber_group: 33399, uber_id: 61261 },
    ),
    (
        "gameStateGroup.gameFinished",
        UberIdentifier { uber_group: 34543, uber_id: 11226 },
    ),
    (
        "gameStateGroup.gameDifficultyMode",
        UberIdentifier { uber_group: 34543, uber_id: 30984 },
    ),
    (
        "corruptedPeakGroup.spineStateB",
        UberIdentifier { uber_group: 36153, uber_id: 2824 },
    ),
    (
        "corruptedPeakGroup.gorlekOreA",
        UberIdentifier { uber_group: 36153, uber_id: 3013 },
    ),
    (
        "corruptedPeakGroup.expOrbB",
        UberIdentifier { uber_group: 36153, uber_id: 3662 },
    ),
    (
        "corruptedPeakGroup.weepingRidgeGetChargeJump",
        UberIdentifier { uber_group: 36153, uber_id: 5369 },
    ),
    (
        "corruptedPeakGroup.mediumExpA",
        UberIdentifier { uber_group: 36153, uber_id: 5552 },
    ),
    (
        "corruptedPeakGroup.xpOrbA",
        UberIdentifier { uber_group: 36153, uber_id: 6682 },
    ),
    (
        "corruptedPeakGroup.spineStateA",
        UberIdentifier { uber_group: 36153, uber_id: 8434 },
    ),
    (
        "corruptedPeakGroup.xpOrbB",
        UberIdentifier { uber_group: 36153, uber_id: 12077 },
    ),
    (
        "corruptedPeakGroup.healthContainerA",
        UberIdentifier { uber_group: 36153, uber_id: 12457 },
    ),
    (
        "corruptedPeakGroup.pressurePlatePuzzleA",
        UberIdentifier { uber_group: 36153, uber_id: 14400 },
    ),
    (
        "corruptedPeakGroup.pressurePlatePuzzleB",
        UberIdentifier { uber_group: 36153, uber_id: 17818 },
    ),
    (
        "corruptedPeakGroup.mediumExpOrbPlaceholder",
        UberIdentifier { uber_group: 36153, uber_id: 18750 },
    ),
    (
        "corruptedPeakGroup.breakableWallD",
        UberIdentifier { uber_group: 36153, uber_id: 18883 },
    ),
    (
        "corruptedPeakGroup.spineStateC",
        UberIdentifier { uber_group: 36153, uber_id: 20307 },
    ),
    (
        "corruptedPeakGroup.breakableRockB",
        UberIdentifier { uber_group: 36153, uber_id: 22461 },
    ),
    (
        "corruptedPeakGroup.elevatorCompleteState",
        UberIdentifier { uber_group: 36153, uber_id: 23584 },
    ),
    (
        "corruptedPeakGroup.expOrbA",
        UberIdentifier { uber_group: 36153, uber_id: 23902 },
    ),
    (
        "corruptedPeakGroup.corruptedPeakSecretWallB",
        UberIdentifier { uber_group: 36153, uber_id: 25095 },
    ),
    (
        "corruptedPeakGroup.spellPickup",
        UberIdentifier { uber_group: 36153, uber_id: 30728 },
    ),
    (
        "corruptedPeakGroup.expOrb",
        UberIdentifier { uber_group: 36153, uber_id: 36521 },
    ),
    (
        "corruptedPeakGroup.spineStateD",
        UberIdentifier { uber_group: 36153, uber_id: 42305 },
    ),
    (
        "corruptedPeakGroup.expOrbC",
        UberIdentifier { uber_group: 36153, uber_id: 42589 },
    ),
    (
        "corruptedPeakGroup.corruptedPeakSecretWallA",
        UberIdentifier { uber_group: 36153, uber_id: 44835 },
    ),
    (
        "corruptedPeakGroup.breakableRockA",
        UberIdentifier { uber_group: 36153, uber_id: 47520 },
    ),
    (
        "corruptedPeakGroup.stompableFloorA",
        UberIdentifier { uber_group: 36153, uber_id: 48472 },
    ),
    (
        "corruptedPeakGroup.stomperStateB",
        UberIdentifier { uber_group: 36153, uber_id: 51042 },
    ),
    (
        "corruptedPeakGroup.expOrb",
        UberIdentifier { uber_group: 36153, uber_id: 53032 },
    ),
    (
        "corruptedPeakGroup.stompableFloorC",
        UberIdentifier { uber_group: 36153, uber_id: 55011 },
    ),
    (
        "corruptedPeakGroup.mediumExpA",
        UberIdentifier { uber_group: 36153, uber_id: 56157 },
    ),
    (
        "corruptedPeakGroup.stompableFloorB",
        UberIdentifier { uber_group: 36153, uber_id: 57716 },
    ),
    (
        "corruptedPeakGroup.breakableWall",
        UberIdentifier { uber_group: 36153, uber_id: 60795 },
    ),
    (
        "corruptedPeakGroup.stomperStateA",
        UberIdentifier { uber_group: 36153, uber_id: 62883 },
    ),
    (
        "corruptedPeakGroup.savePedestalUberState",
        UberIdentifier { uber_group: 36153, uber_id: 43597 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallADestroyedArt",
        UberIdentifier { uber_group: 37858, uber_id: 2615 },
    ),
    (
        "waterMillStateGroupDescriptor.smallExpOrb",
        UberIdentifier { uber_group: 37858, uber_id: 2797 },
    ),
    (
        "waterMillStateGroupDescriptor.hornbugBreakWall",
        UberIdentifier { uber_group: 37858, uber_id: 3421 },
    ),
    (
        "waterMillStateGroupDescriptor.shardSlotUpgrade",
        UberIdentifier { uber_group: 37858, uber_id: 3685 },
    ),
    (
        "waterMillStateGroupDescriptor.dashDoor",
        UberIdentifier { uber_group: 37858, uber_id: 6338 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 37858, uber_id: 8344 },
    ),
    (
        "waterMillStateGroupDescriptor.exitDoorOpen",
        UberIdentifier { uber_group: 37858, uber_id: 9487 },
    ),
    (
        "waterMillStateGroupDescriptor.orePickupA",
        UberIdentifier { uber_group: 37858, uber_id: 11418 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillBossRoomBarrierOpen",
        UberIdentifier { uber_group: 37858, uber_id: 11772 },
    ),
    (
        "waterMillStateGroupDescriptor.finishedWatermillEscape",
        UberIdentifier { uber_group: 37858, uber_id: 12379 },
    ),
    (
        "waterMillStateGroupDescriptor.displayedFlingHint",
        UberIdentifier { uber_group: 37858, uber_id: 13968 },
    ),
    (
        "waterMillStateGroupDescriptor.spiritShardA",
        UberIdentifier { uber_group: 37858, uber_id: 15961 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillEntranceFallingDiscUberStateDescriptor",
        UberIdentifier { uber_group: 37858, uber_id: 16604 },
    ),
    (
        "waterMillStateGroupDescriptor.expOrbA",
        UberIdentifier { uber_group: 37858, uber_id: 16611 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 37858, uber_id: 19347 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallBDestroyed",
        UberIdentifier { uber_group: 37858, uber_id: 21874 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 37858, uber_id: 22107 },
    ),
    (
        "waterMillStateGroupDescriptor.playedNaruGumoCutaway",
        UberIdentifier { uber_group: 37858, uber_id: 23225 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillEntranceDoorUberStateDescriptor",
        UberIdentifier { uber_group: 37858, uber_id: 23644 },
    ),
    (
        "waterMillStateGroupDescriptor.rescuedOpher",
        UberIdentifier { uber_group: 37858, uber_id: 25031 },
    ),
    (
        "waterMillStateGroupDescriptor.healthContainerA",
        UberIdentifier { uber_group: 37858, uber_id: 25833 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillVisited",
        UberIdentifier { uber_group: 37858, uber_id: 26885 },
    ),
    (
        "waterMillStateGroupDescriptor.poleLowered",
        UberIdentifier { uber_group: 37858, uber_id: 31104 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 37858, uber_id: 31136 },
    ),
    (
        "waterMillStateGroupDescriptor.recedingWater",
        UberIdentifier { uber_group: 37858, uber_id: 31187 },
    ),
    (
        "waterMillStateGroupDescriptor.wheelsActivated",
        UberIdentifier { uber_group: 37858, uber_id: 31584 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillBEntranceTriggerUberStateDescriptor",
        UberIdentifier { uber_group: 37858, uber_id: 31962 },
    ),
    (
        "waterMillStateGroupDescriptor.doorWithTwoSlotsBooleanDescriptor",
        UberIdentifier { uber_group: 37858, uber_id: 31966 },
    ),
    (
        "waterMillStateGroupDescriptor.keystoneA",
        UberIdentifier { uber_group: 37858, uber_id: 32628 },
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOreA",
        UberIdentifier { uber_group: 37858, uber_id: 32932 },
    ),
    (
        "waterMillStateGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 37858, uber_id: 33063 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpB",
        UberIdentifier { uber_group: 37858, uber_id: 33642 },
    ),
    (
        "waterMillStateGroupDescriptor.wheelLever",
        UberIdentifier { uber_group: 37858, uber_id: 34433 },
    ),
    (
        "waterMillStateGroupDescriptor.enemyDoor",
        UberIdentifier { uber_group: 37858, uber_id: 34619 },
    ),
    (
        "waterMillStateGroupDescriptor.spiritShardA",
        UberIdentifier { uber_group: 37858, uber_id: 34646 },
    ),
    (
        "waterMillStateGroupDescriptor.enemyDoorA",
        UberIdentifier { uber_group: 37858, uber_id: 37323 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 37858, uber_id: 41380 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpB",
        UberIdentifier { uber_group: 37858, uber_id: 41911 },
    ),
    (
        "waterMillStateGroupDescriptor.keystoneA",
        UberIdentifier { uber_group: 37858, uber_id: 43893 },
    ),
    (
        "waterMillStateGroupDescriptor.xpOrbWater",
        UberIdentifier { uber_group: 37858, uber_id: 45656 },
    ),
    (
        "waterMillStateGroupDescriptor.smallExpA",
        UberIdentifier { uber_group: 37858, uber_id: 45906 },
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOraA",
        UberIdentifier { uber_group: 37858, uber_id: 47533 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpC",
        UberIdentifier { uber_group: 37858, uber_id: 50064 },
    ),
    (
        "waterMillStateGroupDescriptor.watermillEntranceTalkedToOpher",
        UberIdentifier { uber_group: 37858, uber_id: 50780 },
    ),
    (
        "waterMillStateGroupDescriptor.wheelAActive",
        UberIdentifier { uber_group: 37858, uber_id: 50902 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpB",
        UberIdentifier { uber_group: 37858, uber_id: 52110 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillBossRoomWheelFelt",
        UberIdentifier { uber_group: 37858, uber_id: 52129 },
    ),
    (
        "waterMillStateGroupDescriptor.hintZone",
        UberIdentifier { uber_group: 37858, uber_id: 54231 },
    ),
    (
        "waterMillStateGroupDescriptor.xpOrbA",
        UberIdentifier { uber_group: 37858, uber_id: 55450 },
    ),
    (
        "waterMillStateGroupDescriptor.expOrbB",
        UberIdentifier { uber_group: 37858, uber_id: 55499 },
    ),
    (
        "waterMillStateGroupDescriptor.xpOrb",
        UberIdentifier { uber_group: 37858, uber_id: 56444 },
    ),
    (
        "waterMillStateGroupDescriptor.energyVessel",
        UberIdentifier { uber_group: 37858, uber_id: 57552 },
    ),
    (
        "waterMillStateGroupDescriptor.arenaWheelsActivated",
        UberIdentifier { uber_group: 37858, uber_id: 58000 },
    ),
    (
        "waterMillStateGroupDescriptor.smallExpAArt",
        UberIdentifier { uber_group: 37858, uber_id: 58220 },
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOreA",
        UberIdentifier { uber_group: 37858, uber_id: 58286 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallADestroyedArtB",
        UberIdentifier { uber_group: 37858, uber_id: 58736 },
    ),
    (
        "waterMillStateGroupDescriptor.gorlekOreB",
        UberIdentifier { uber_group: 37858, uber_id: 58846 },
    ),
    (
        "waterMillStateGroupDescriptor.shardSlotExpansion",
        UberIdentifier { uber_group: 37858, uber_id: 58947 },
    ),
    (
        "waterMillStateGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 37858, uber_id: 59022 },
    ),
    (
        "waterMillStateGroupDescriptor.wheelBActive",
        UberIdentifier { uber_group: 37858, uber_id: 60716 },
    ),
    (
        "waterMillStateGroupDescriptor.waterMillSecretWallADestroyed",
        UberIdentifier { uber_group: 37858, uber_id: 61481 },
    ),
    (
        "waterMillStateGroupDescriptor.wheelsActivatedEntry",
        UberIdentifier { uber_group: 37858, uber_id: 64055 },
    ),
    (
        "waterMillStateGroupDescriptor.expOrb",
        UberIdentifier { uber_group: 37858, uber_id: 64086 },
    ),
    (
        "waterMillStateGroupDescriptor.spiritShardA",
        UberIdentifier { uber_group: 37858, uber_id: 64961 },
    ),
    (
        "waterMillStateGroupDescriptor.lifeCellA",
        UberIdentifier { uber_group: 37858, uber_id: 65187 },
    ),
    (
        "waterMillStateGroupDescriptor.rotatingEnemyArenaStates",
        UberIdentifier { uber_group: 37858, uber_id: 8487 },
    ),
    (
        "waterMillStateGroupDescriptor.watermillEscapeState",
        UberIdentifier { uber_group: 37858, uber_id: 10720 },
    ),
    (
        "waterMillStateGroupDescriptor.rotatingEnemyArenaRotationStateController",
        UberIdentifier { uber_group: 37858, uber_id: 34636 },
    ),
    (
        "waterMillStateGroupDescriptor.rotationState",
        UberIdentifier { uber_group: 37858, uber_id: 36070 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsARespawnTimer",
        UberIdentifier { uber_group: 37858, uber_id: 5107 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 8675 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 17790 },
    ),
    (
        "waterMillStateGroupDescriptor.healthPlantB",
        UberIdentifier { uber_group: 37858, uber_id: 22902 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsA",
        UberIdentifier { uber_group: 37858, uber_id: 24680 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 28311 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 38044 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 44551 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsARespawnTimerB",
        UberIdentifier { uber_group: 37858, uber_id: 48554 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 54186 },
    ),
    (
        "waterMillStateGroupDescriptor.healthPlant",
        UberIdentifier { uber_group: 37858, uber_id: 57762 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbsB",
        UberIdentifier { uber_group: 37858, uber_id: 60777 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 61727 },
    ),
    (
        "waterMillStateGroupDescriptor.landOnAndSpawnOrbs",
        UberIdentifier { uber_group: 37858, uber_id: 62404 },
    ),
    (
        "spiderBatTestGroup.roundOneDefeated",
        UberIdentifier { uber_group: 42171, uber_id: 14000 },
    ),
    (
        "spiderBatTestGroup.arenaDoorClosed",
        UberIdentifier { uber_group: 42171, uber_id: 26771 },
    ),
    (
        "spiderBatTestGroup.allRoundsDefeated",
        UberIdentifier { uber_group: 42171, uber_id: 32228 },
    ),
    (
        "spiderBatTestGroup.roundTwoDefeated",
        UberIdentifier { uber_group: 42171, uber_id: 43227 },
    ),
    (
        "spiderBatTestGroup.roundThreeDefeated",
        UberIdentifier { uber_group: 42171, uber_id: 56229 },
    ),
    (
        "spiderBatTestGroup.enemyArenaState",
        UberIdentifier { uber_group: 42171, uber_id: 53383 },
    ),
    (
        "hubUberStateGroup.leafPileB",
        UberIdentifier { uber_group: 42178, uber_id: 3295 },
    ),
    (
        "hubUberStateGroup.mediumExpD",
        UberIdentifier { uber_group: 42178, uber_id: 4125 },
    ),
    (
        "hubUberStateGroup.leafPileC",
        UberIdentifier { uber_group: 42178, uber_id: 5630 },
    ),
    (
        "hubUberStateGroup.stompableFloorEnterHub",
        UberIdentifier { uber_group: 42178, uber_id: 5815 },
    ),
    (
        "hubUberStateGroup.mediumExpG",
        UberIdentifier { uber_group: 42178, uber_id: 6117 },
    ),
    (
        "hubUberStateGroup.woodCrateC",
        UberIdentifier { uber_group: 42178, uber_id: 9319 },
    ),
    (
        "hubUberStateGroup.mediumExpE",
        UberIdentifier { uber_group: 42178, uber_id: 9780 },
    ),
    (
        "hubUberStateGroup.leafPileA",
        UberIdentifier { uber_group: 42178, uber_id: 10035 },
    ),
    (
        "hubUberStateGroup.hutBExpOrb",
        UberIdentifier { uber_group: 42178, uber_id: 13327 },
    ),
    (
        "hubUberStateGroup.mediumExpA",
        UberIdentifier { uber_group: 42178, uber_id: 14903 },
    ),
    (
        "hubUberStateGroup.woodCrateA",
        UberIdentifier { uber_group: 42178, uber_id: 15685 },
    ),
    (
        "hubUberStateGroup.woodCrateC",
        UberIdentifier { uber_group: 42178, uber_id: 17158 },
    ),
    (
        "hubUberStateGroup.smallExpE",
        UberIdentifier { uber_group: 42178, uber_id: 17489 },
    ),
    (
        "hubUberStateGroup.drillableWallB",
        UberIdentifier { uber_group: 42178, uber_id: 17732 },
    ),
    (
        "hubUberStateGroup.mediumExpF",
        UberIdentifier { uber_group: 42178, uber_id: 18448 },
    ),
    (
        "hubUberStateGroup.areaText",
        UberIdentifier { uber_group: 42178, uber_id: 19692 },
    ),
    (
        "hubUberStateGroup.woodCrateE",
        UberIdentifier { uber_group: 42178, uber_id: 21105 },
    ),
    (
        "hubUberStateGroup.gorlekOreA",
        UberIdentifier { uber_group: 42178, uber_id: 23125 },
    ),
    (
        "hubUberStateGroup.hubSpritWellIconVisible",
        UberIdentifier { uber_group: 42178, uber_id: 23193 },
    ),
    (
        "hubUberStateGroup.woodCrateC",
        UberIdentifier { uber_group: 42178, uber_id: 26189 },
    ),
    (
        "hubUberStateGroup.woodCrateA",
        UberIdentifier { uber_group: 42178, uber_id: 26365 },
    ),
    (
        "hubUberStateGroup.gorlekOreB",
        UberIdentifier { uber_group: 42178, uber_id: 27110 },
    ),
    (
        "hubUberStateGroup.warpHintShowed",
        UberIdentifier { uber_group: 42178, uber_id: 27777 },
    ),
    (
        "hubUberStateGroup.smallExpA",
        UberIdentifier { uber_group: 42178, uber_id: 30206 },
    ),
    (
        "hubUberStateGroup.hutDExpOrbB",
        UberIdentifier { uber_group: 42178, uber_id: 30520 },
    ),
    (
        "hubUberStateGroup.drillableWallA",
        UberIdentifier { uber_group: 42178, uber_id: 31795 },
    ),
    (
        "hubUberStateGroup.smallExpD",
        UberIdentifier { uber_group: 42178, uber_id: 35232 },
    ),
    (
        "hubUberStateGroup.woodCrateE",
        UberIdentifier { uber_group: 42178, uber_id: 35855 },
    ),
    (
        "hubUberStateGroup.mediumExpA",
        UberIdentifier { uber_group: 42178, uber_id: 36085 },
    ),
    (
        "hubUberStateGroup.woodCrateD",
        UberIdentifier { uber_group: 42178, uber_id: 36464 },
    ),
    (
        "hubUberStateGroup.fatherMokiGone",
        UberIdentifier { uber_group: 42178, uber_id: 36609 },
    ),
    (
        "hubUberStateGroup.smallExpB",
        UberIdentifier { uber_group: 42178, uber_id: 37028 },
    ),
    (
        "hubUberStateGroup.mediumExpC",
        UberIdentifier { uber_group: 42178, uber_id: 38743 },
    ),
    (
        "hubUberStateGroup.pyreA",
        UberIdentifier { uber_group: 42178, uber_id: 38905 },
    ),
    (
        "hubUberStateGroup.mediumExpB",
        UberIdentifier { uber_group: 42178, uber_id: 40609 },
    ),
    (
        "hubUberStateGroup.smallExpH",
        UberIdentifier { uber_group: 42178, uber_id: 42762 },
    ),
    (
        "hubUberStateGroup.smallExpG",
        UberIdentifier { uber_group: 42178, uber_id: 44748 },
    ),
    (
        "hubUberStateGroup.woodCrateB",
        UberIdentifier { uber_group: 42178, uber_id: 47152 },
    ),
    (
        "hubUberStateGroup.woodCrateA",
        UberIdentifier { uber_group: 42178, uber_id: 50325 },
    ),
    (
        "hubUberStateGroup.gromIntroSequencePlayed",
        UberIdentifier { uber_group: 42178, uber_id: 50418 },
    ),
    (
        "hubUberStateGroup.woodCrateB",
        UberIdentifier { uber_group: 42178, uber_id: 51080 },
    ),
    (
        "hubUberStateGroup.hutAExpOrb",
        UberIdentifier { uber_group: 42178, uber_id: 51468 },
    ),
    (
        "hubUberStateGroup.hutEExpOrb",
        UberIdentifier { uber_group: 42178, uber_id: 51934 },
    ),
    (
        "hubUberStateGroup.hutDExpOrb",
        UberIdentifier { uber_group: 42178, uber_id: 52497 },
    ),
    (
        "hubUberStateGroup.energyCellA",
        UberIdentifier { uber_group: 42178, uber_id: 52786 },
    ),
    (
        "hubUberStateGroup.woodCrateD",
        UberIdentifier { uber_group: 42178, uber_id: 56980 },
    ),
    (
        "hubUberStateGroup.hutCExpOrb",
        UberIdentifier { uber_group: 42178, uber_id: 57455 },
    ),
    (
        "hubUberStateGroup.largeExpA",
        UberIdentifier { uber_group: 42178, uber_id: 57675 },
    ),
    (
        "hubUberStateGroup.mediumExpB",
        UberIdentifier { uber_group: 42178, uber_id: 59623 },
    ),
    (
        "hubUberStateGroup.woodCrateB",
        UberIdentifier { uber_group: 42178, uber_id: 63260 },
    ),
    (
        "hubUberStateGroup.smallExpC",
        UberIdentifier { uber_group: 42178, uber_id: 63404 },
    ),
    (
        "hubUberStateGroup.woodCrateF",
        UberIdentifier { uber_group: 42178, uber_id: 63819 },
    ),
    (
        "hubUberStateGroup.savePedestal",
        UberIdentifier { uber_group: 42178, uber_id: 42096 },
    ),
    (
        "hubUberStateGroup.shardPurchaseCount",
        UberIdentifier { uber_group: 42178, uber_id: 38 },
    ),
    (
        "hubUberStateGroup.craftCutsceneState",
        UberIdentifier { uber_group: 42178, uber_id: 2654 },
    ),
    (
        "hubUberStateGroup.builderProjectShardShop",
        UberIdentifier { uber_group: 42178, uber_id: 7528 },
    ),
    (
        "hubUberStateGroup.builderProjectBeautify",
        UberIdentifier { uber_group: 42178, uber_id: 15068 },
    ),
    (
        "hubUberStateGroup.gardenerProjectFlowers",
        UberIdentifier { uber_group: 42178, uber_id: 16254 },
    ),
    (
        "hubUberStateGroup.builderProjectOpenCave",
        UberIdentifier { uber_group: 42178, uber_id: 16586 },
    ),
    (
        "hubUberStateGroup.builderProjectSpiritWell",
        UberIdentifier { uber_group: 42178, uber_id: 16825 },
    ),
    (
        "hubUberStateGroup.builderProjectRemoveThorns",
        UberIdentifier { uber_group: 42178, uber_id: 18751 },
    ),
    (
        "hubUberStateGroup.builderProjectHousesB",
        UberIdentifier { uber_group: 42178, uber_id: 23607 },
    ),
    (
        "hubUberStateGroup.gardenerProjectGrapplePlants",
        UberIdentifier { uber_group: 42178, uber_id: 33011 },
    ),
    (
        "hubUberStateGroup.gardenerProjectSpringPlants",
        UberIdentifier { uber_group: 42178, uber_id: 38393 },
    ),
    (
        "hubUberStateGroup.gardenerProjectTree",
        UberIdentifier { uber_group: 42178, uber_id: 40006 },
    ),
    (
        "hubUberStateGroup.builderProjectHousesC",
        UberIdentifier { uber_group: 42178, uber_id: 40448 },
    ),
    (
        "hubUberStateGroup.gardenerProjectBashPlants",
        UberIdentifier { uber_group: 42178, uber_id: 47651 },
    ),
    (
        "hubUberStateGroup.builderProjectHouses",
        UberIdentifier { uber_group: 42178, uber_id: 51230 },
    ),
    (
        "hubUberStateGroup.shardShopState",
        UberIdentifier { uber_group: 42178, uber_id: 61304 },
    ),
    (
        "hubUberStateGroup.gardenerProjectGrass",
        UberIdentifier { uber_group: 42178, uber_id: 64583 },
    ),
    (
        "wellspringGladesGroup.stompableFloorC",
        UberIdentifier { uber_group: 44310, uber_id: 125 },
    ),
    (
        "wellspringGladesGroup.smallExpA",
        UberIdentifier { uber_group: 44310, uber_id: 1647 },
    ),
    (
        "wellspringGladesGroup.shardSlotUpgrade",
        UberIdentifier { uber_group: 44310, uber_id: 9902 },
    ),
    (
        "wellspringGladesGroup.shardTraderState",
        UberIdentifier { uber_group: 44310, uber_id: 15689 },
    ),
    (
        "wellspringGladesGroup.lifeVesselB",
        UberIdentifier { uber_group: 44310, uber_id: 17523 },
    ),
    (
        "wellspringGladesGroup.lifeVesselA",
        UberIdentifier { uber_group: 44310, uber_id: 29043 },
    ),
    (
        "wellspringGladesGroup.lifeVesselA",
        UberIdentifier { uber_group: 44310, uber_id: 36911 },
    ),
    (
        "wellspringGladesGroup.blowableFlame",
        UberIdentifier { uber_group: 44310, uber_id: 47361 },
    ),
    (
        "wellspringGladesGroup.largeExpA",
        UberIdentifier { uber_group: 44310, uber_id: 47723 },
    ),
    (
        "wellspringGladesGroup.mediumExpA",
        UberIdentifier { uber_group: 44310, uber_id: 47923 },
    ),
    (
        "wellspringGladesGroup.stompableFloorA",
        UberIdentifier { uber_group: 44310, uber_id: 55192 },
    ),
    (
        "wellspringGladesGroup.stompableFloorB",
        UberIdentifier { uber_group: 44310, uber_id: 57009 },
    ),
    (
        "wellspringGladesGroup.shrineEnemyRoom",
        UberIdentifier { uber_group: 44310, uber_id: 58796 },
    ),
    (
        "raceGroup.firstRaceUnlockedMessagePlayed",
        UberIdentifier { uber_group: 44964, uber_id: 8328 },
    ),
    (
        "raceGroup.wellspringRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 12682 },
    ),
    (
        "raceGroup.baursReachWindTunnelRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 33045 },
    ),
    (
        "raceGroup.silentWoodlandRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 34110 },
    ),
    (
        "raceGroup.desertRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 38162 },
    ),
    (
        "raceGroup.mouldwoodDepthsRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 40578 },
    ),
    (
        "raceGroup.inkwaterMarshRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 50495 },
    ),
    (
        "raceGroup.lumaPoolsRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 56533 },
    ),
    (
        "raceGroup.kwolokDropRaceIcon",
        UberIdentifier { uber_group: 44964, uber_id: 63031 },
    ),
    (
        "raceGroup.raceLeaderboardFilterState",
        UberIdentifier { uber_group: 44964, uber_id: 3798 },
    ),
    (
        "raceGroup.wellspringRace",
        UberIdentifier { uber_group: 44964, uber_id: 11512 },
    ),
    (
        "raceGroup.silentWoodlandRace",
        UberIdentifier { uber_group: 44964, uber_id: 22703 },
    ),
    (
        "raceGroup.baursReachWindTunnelRace",
        UberIdentifier { uber_group: 44964, uber_id: 23661 },
    ),
    (
        "raceGroup.kwolokDropRace",
        UberIdentifier { uber_group: 44964, uber_id: 25545 },
    ),
    (
        "raceGroup.mouldwoodDepthsRace",
        UberIdentifier { uber_group: 44964, uber_id: 28552 },
    ),
    (
        "raceGroup.desertRace",
        UberIdentifier { uber_group: 44964, uber_id: 30767 },
    ),
    (
        "raceGroup.inkwaterMarshRace",
        UberIdentifier { uber_group: 44964, uber_id: 45951 },
    ),
    (
        "raceGroup.testRace",
        UberIdentifier { uber_group: 44964, uber_id: 50634 },
    ),
    (
        "raceGroup.lumaPoolsRace",
        UberIdentifier { uber_group: 44964, uber_id: 54686 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.mediumExpA",
        UberIdentifier { uber_group: 46462, uber_id: 3872 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.spiritShardA",
        UberIdentifier { uber_group: 46462, uber_id: 9440 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.interactedWithMourningMoki",
        UberIdentifier { uber_group: 46462, uber_id: 20733 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.smallExpA",
        UberIdentifier { uber_group: 46462, uber_id: 20780 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.bombableDoor",
        UberIdentifier { uber_group: 46462, uber_id: 26623 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.largeExpA",
        UberIdentifier { uber_group: 46462, uber_id: 29054 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.leafPileA",
        UberIdentifier { uber_group: 46462, uber_id: 31447 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.questRewardOrb",
        UberIdentifier { uber_group: 46462, uber_id: 31575 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.bombableWallA",
        UberIdentifier { uber_group: 46462, uber_id: 34885 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.gorlekOreA",
        UberIdentifier { uber_group: 46462, uber_id: 37897 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.leafPileC",
        UberIdentifier { uber_group: 46462, uber_id: 56958 },
    ),
    (
        "kwoloksCavernThroneRoomGroup.wispRewardPickup",
        UberIdentifier { uber_group: 46462, uber_id: 59806 },
    ),
    (
        "npcsStateGroup.windtornRuinsWispTeaser",
        UberIdentifier { uber_group: 48248, uber_id: 1350 },
    ),
    (
        "npcsStateGroup.hasMapLumaPools",
        UberIdentifier { uber_group: 48248, uber_id: 1557 },
    ),
    (
        "npcsStateGroup.hasMapWellspring",
        UberIdentifier { uber_group: 48248, uber_id: 1590 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWeepingRidge",
        UberIdentifier { uber_group: 48248, uber_id: 2253 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspringGlades",
        UberIdentifier { uber_group: 48248, uber_id: 2285 },
    ),
    (
        "npcsStateGroup.treekeeperBRetalk",
        UberIdentifier { uber_group: 48248, uber_id: 3492 },
    ),
    (
        "npcsStateGroup.hasMapKwoloksHollow",
        UberIdentifier { uber_group: 48248, uber_id: 3638 },
    ),
    (
        "npcsStateGroup.stenchTease",
        UberIdentifier { uber_group: 48248, uber_id: 3846 },
    ),
    (
        "npcsStateGroup.hasMapWillowsEnd",
        UberIdentifier { uber_group: 48248, uber_id: 4045 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspringAfterQuest",
        UberIdentifier { uber_group: 48248, uber_id: 4306 },
    ),
    (
        "npcsStateGroup.lupoWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 4510 },
    ),
    (
        "npcsStateGroup.tuleyMentionedSeed",
        UberIdentifier { uber_group: 48248, uber_id: 5060 },
    ),
    (
        "npcsStateGroup.gromMentionedOre",
        UberIdentifier { uber_group: 48248, uber_id: 5186 },
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispE",
        UberIdentifier { uber_group: 48248, uber_id: 5269 },
    ),
    (
        "npcsStateGroup.metOpherHubAfterWatermill",
        UberIdentifier { uber_group: 48248, uber_id: 5982 },
    ),
    (
        "npcsStateGroup.twillenGaveRumor",
        UberIdentifier { uber_group: 48248, uber_id: 6194 },
    ),
    (
        "npcsStateGroup.interactedWindsweptWastesCondition",
        UberIdentifier { uber_group: 48248, uber_id: 6730 },
    ),
    (
        "npcsStateGroup.lupoEncounteredSilentWoodlands",
        UberIdentifier { uber_group: 48248, uber_id: 6992 },
    ),
    (
        "npcsStateGroup.lupoEncounteredMouldwoodDepths",
        UberIdentifier { uber_group: 48248, uber_id: 7056 },
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispA",
        UberIdentifier { uber_group: 48248, uber_id: 7160 },
    ),
    (
        "npcsStateGroup.gromTalkedAboutBaur",
        UberIdentifier { uber_group: 48248, uber_id: 7321 },
    ),
    (
        "npcsStateGroup.gromGaveWarning",
        UberIdentifier { uber_group: 48248, uber_id: 7646 },
    ),
    (
        "npcsStateGroup.willowsEndSeirExitCutscene",
        UberIdentifier { uber_group: 48248, uber_id: 8985 },
    ),
    (
        "npcsStateGroup.metGrom",
        UberIdentifier { uber_group: 48248, uber_id: 9394 },
    ),
    (
        "npcsStateGroup.hasMapGorlekMines",
        UberIdentifier { uber_group: 48248, uber_id: 9750 },
    ),
    (
        "npcsStateGroup.mouldwoodDepthWispTeaser",
        UberIdentifier { uber_group: 48248, uber_id: 11223 },
    ),
    (
        "npcsStateGroup.lupoEncounteredBaursReach",
        UberIdentifier { uber_group: 48248, uber_id: 12352 },
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispD",
        UberIdentifier { uber_group: 48248, uber_id: 13320 },
    ),
    (
        "npcsStateGroup.interactedAfterWellOpened",
        UberIdentifier { uber_group: 48248, uber_id: 14878 },
    ),
    (
        "npcsStateGroup.hasMapWindtornRuins",
        UberIdentifier { uber_group: 48248, uber_id: 14995 },
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispB",
        UberIdentifier { uber_group: 48248, uber_id: 15833 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWillowsEnd",
        UberIdentifier { uber_group: 48248, uber_id: 16157 },
    ),
    (
        "npcsStateGroup.lumaPoolsWispSpotted",
        UberIdentifier { uber_group: 48248, uber_id: 18425 },
    ),
    (
        "npcsStateGroup.hasMapInkwaterMarsh",
        UberIdentifier { uber_group: 48248, uber_id: 18767 },
    ),
    (
        "npcsStateGroup.kiiWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 19551 },
    ),
    (
        "npcsStateGroup.Has bought everything",
        UberIdentifier { uber_group: 48248, uber_id: 20000 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspringValley",
        UberIdentifier { uber_group: 48248, uber_id: 21009 },
    ),
    (
        "npcsStateGroup.metOpherLibrary",
        UberIdentifier { uber_group: 48248, uber_id: 22890 },
    ),
    (
        "npcsStateGroup.motayWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 24328 },
    ),
    (
        "npcsStateGroup.tokkWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 25629 },
    ),
    (
        "npcsStateGroup.lupoEncounteredKwoloksHollow",
        UberIdentifier { uber_group: 48248, uber_id: 26627 },
    ),
    (
        "npcsStateGroup.lupoEncounteredGorlekMines",
        UberIdentifier { uber_group: 48248, uber_id: 27701 },
    ),
    (
        "npcsStateGroup.tuleyWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 28327 },
    ),
    (
        "npcsStateGroup.windsweptWastesRuinsDoor",
        UberIdentifier { uber_group: 48248, uber_id: 28782 },
    ),
    (
        "npcsStateGroup.hasMapBaursReach",
        UberIdentifier { uber_group: 48248, uber_id: 29604 },
    ),
    (
        "npcsStateGroup.gromTalkedAboutLagoon",
        UberIdentifier { uber_group: 48248, uber_id: 30073 },
    ),
    (
        "npcsStateGroup.interactedKwoloksCavern",
        UberIdentifier { uber_group: 48248, uber_id: 32549 },
    ),
    (
        "npcsStateGroup.baurReachWispTease",
        UberIdentifier { uber_group: 48248, uber_id: 32918 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWindsweptWastes",
        UberIdentifier { uber_group: 48248, uber_id: 34318 },
    ),
    (
        "npcsStateGroup.twillenMournedKu",
        UberIdentifier { uber_group: 48248, uber_id: 34756 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWindtornRuins",
        UberIdentifier { uber_group: 48248, uber_id: 35651 },
    ),
    (
        "npcsStateGroup.mouldwoodDepthsWisptIntro",
        UberIdentifier { uber_group: 48248, uber_id: 37364 },
    ),
    (
        "npcsStateGroup.hasMapWeepingRidge",
        UberIdentifier { uber_group: 48248, uber_id: 37481 },
    ),
    (
        "npcsStateGroup.treekeeperARetalk",
        UberIdentifier { uber_group: 48248, uber_id: 37606 },
    ),
    (
        "npcsStateGroup.lupoEncounteredUberState",
        UberIdentifier { uber_group: 48248, uber_id: 40170 },
    ),
    (
        "npcsStateGroup.baurReachWispIntro",
        UberIdentifier { uber_group: 48248, uber_id: 40451 },
    ),
    (
        "npcsStateGroup.lupoEncounteredBaursReachAfterThaw",
        UberIdentifier { uber_group: 48248, uber_id: 41206 },
    ),
    (
        "npcsStateGroup.tokkIntroduced",
        UberIdentifier { uber_group: 48248, uber_id: 42584 },
    ),
    (
        "npcsStateGroup.gromWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 43860 },
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispF",
        UberIdentifier { uber_group: 48248, uber_id: 44446 },
    ),
    (
        "npcsStateGroup.hasMapHowlsOrigin",
        UberIdentifier { uber_group: 48248, uber_id: 45538 },
    ),
    (
        "npcsStateGroup.willowsEndSeirIntro",
        UberIdentifier { uber_group: 48248, uber_id: 45600 },
    ),
    (
        "npcsStateGroup.interactedBeforeMill",
        UberIdentifier { uber_group: 48248, uber_id: 45664 },
    ),
    (
        "npcsStateGroup.gromTalkedAboutDesert",
        UberIdentifier { uber_group: 48248, uber_id: 45751 },
    ),
    (
        "npcsStateGroup.gromTalkedAboutMouldwoodGate",
        UberIdentifier { uber_group: 48248, uber_id: 46471 },
    ),
    (
        "npcsStateGroup.opherMentiodedWatermill",
        UberIdentifier { uber_group: 48248, uber_id: 46745 },
    ),
    (
        "npcsStateGroup.gromInteractedOnce",
        UberIdentifier { uber_group: 48248, uber_id: 46863 },
    ),
    (
        "npcsStateGroup.hasMapWellspringValley",
        UberIdentifier { uber_group: 48248, uber_id: 47517 },
    ),
    (
        "npcsStateGroup.lupoEncounteredHowlsOrigin",
        UberIdentifier { uber_group: 48248, uber_id: 47546 },
    ),
    (
        "npcsStateGroup.feedingGroundsWispIntro",
        UberIdentifier { uber_group: 48248, uber_id: 47785 },
    ),
    (
        "npcsStateGroup.hasMapMouldwoodDepths",
        UberIdentifier { uber_group: 48248, uber_id: 48423 },
    ),
    (
        "npcsStateGroup.lupoEncounteredInkwaterMarsh",
        UberIdentifier { uber_group: 48248, uber_id: 48619 },
    ),
    (
        "npcsStateGroup.interactedBeforeKwolok",
        UberIdentifier { uber_group: 48248, uber_id: 50408 },
    ),
    (
        "npcsStateGroup.opherWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 51005 },
    ),
    (
        "npcsStateGroup.desertRuinsLoreWispC",
        UberIdentifier { uber_group: 48248, uber_id: 52065 },
    ),
    (
        "npcsStateGroup.metMotay",
        UberIdentifier { uber_group: 48248, uber_id: 53028 },
    ),
    (
        "npcsStateGroup.hasMapWellspringGlades",
        UberIdentifier { uber_group: 48248, uber_id: 54647 },
    ),
    (
        "npcsStateGroup.gromTalkedAboutWatermill",
        UberIdentifier { uber_group: 48248, uber_id: 54806 },
    ),
    (
        "npcsStateGroup.metOpherHubBeforeWatermill",
        UberIdentifier { uber_group: 48248, uber_id: 55122 },
    ),
    (
        "npcsStateGroup.lupoEncounteredLumaPools",
        UberIdentifier { uber_group: 48248, uber_id: 55617 },
    ),
    (
        "npcsStateGroup.metOpherHub",
        UberIdentifier { uber_group: 48248, uber_id: 56448 },
    ),
    (
        "npcsStateGroup.twilenWantsToTalkToYou",
        UberIdentifier { uber_group: 48248, uber_id: 60805 },
    ),
    (
        "npcsStateGroup.hasMapWindsweptWastes",
        UberIdentifier { uber_group: 48248, uber_id: 61146 },
    ),
    (
        "npcsStateGroup.hasMapSilentWoodlands",
        UberIdentifier { uber_group: 48248, uber_id: 61819 },
    ),
    (
        "npcsStateGroup.lupoEncounteredWellspring",
        UberIdentifier { uber_group: 48248, uber_id: 61868 },
    ),
    (
        "npcsStateGroup.tokkLagoonDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 2131 },
    ),
    (
        "npcsStateGroup.talkedInHub",
        UberIdentifier { uber_group: 48248, uber_id: 10337 },
    ),
    (
        "npcsStateGroup.twillenHubDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 12799 },
    ),
    (
        "npcsStateGroup.inkwaterWellQuest",
        UberIdentifier { uber_group: 48248, uber_id: 18458 },
    ),
    (
        "npcsStateGroup.HCMapIconCost",
        UberIdentifier { uber_group: 48248, uber_id: 19397 },
    ),
    (
        "npcsStateGroup.twillenKwolokDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 25267 },
    ),
    (
        "npcsStateGroup.watermillCEntranceInteraction",
        UberIdentifier { uber_group: 48248, uber_id: 26696 },
    ),
    (
        "npcsStateGroup.childMokiDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 28897 },
    ),
    (
        "npcsStateGroup.wandererNeedleQuest",
        UberIdentifier { uber_group: 48248, uber_id: 32160 },
    ),
    (
        "npcsStateGroup.tokkKwolokDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 33981 },
    ),
    (
        "npcsStateGroup.ShardMapIconCost",
        UberIdentifier { uber_group: 48248, uber_id: 41667 },
    ),
    (
        "npcsStateGroup.frozenMokiDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 42865 },
    ),
    (
        "npcsStateGroup.marshKeystoneQuest",
        UberIdentifier { uber_group: 48248, uber_id: 51645 },
    ),
    (
        "npcsStateGroup.iceFisherDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 54962 },
    ),
    (
        "npcsStateGroup.mouldwoodMokiDialogState",
        UberIdentifier { uber_group: 48248, uber_id: 57674 },
    ),
    (
        "npcsStateGroup.ECMapIconCost",
        UberIdentifier { uber_group: 48248, uber_id: 57988 },
    ),
    (
        "npcsStateGroup.lupoIntroState",
        UberIdentifier { uber_group: 48248, uber_id: 62835 },
    ),
    (
        "npcsStateGroup.tokkState",
        UberIdentifier { uber_group: 48248, uber_id: 15642 },
    ),
    (
        "npcsStateGroup.fastTravelEnabledUberState",
        UberIdentifier { uber_group: 48248, uber_id: 16489 },
    ),
    (
        "npcsStateGroup.mapmakerShowMapIconEnergyUberState",
        UberIdentifier { uber_group: 48248, uber_id: 19396 },
    ),
    (
        "npcsStateGroup.ShowMapIconCreepheartUberState",
        UberIdentifier { uber_group: 48248, uber_id: 38077 },
    ),
    (
        "npcsStateGroup.mapmakerShowMapIconShardUberState",
        UberIdentifier { uber_group: 48248, uber_id: 41666 },
    ),
    (
        "npcsStateGroup.mapmakerShowMapIconHealthUberState",
        UberIdentifier { uber_group: 48248, uber_id: 57987 },
    ),
    (
        "wellspringGroupDescriptor.energyVesselA",
        UberIdentifier { uber_group: 53632, uber_id: 1911 },
    ),
    (
        "wellspringGroupDescriptor.lanternAndCreepA",
        UberIdentifier { uber_group: 53632, uber_id: 2522 },
    ),
    (
        "wellspringGroupDescriptor.pushBlockPuzzleA",
        UberIdentifier { uber_group: 53632, uber_id: 3195 },
    ),
    (
        "wellspringGroupDescriptor.secretWallB",
        UberIdentifier { uber_group: 53632, uber_id: 3382 },
    ),
    (
        "wellspringGroupDescriptor.leafPileA",
        UberIdentifier { uber_group: 53632, uber_id: 3622 },
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderG",
        UberIdentifier { uber_group: 53632, uber_id: 6500 },
    ),
    (
        "wellspringGroupDescriptor.energyVesselB",
        UberIdentifier { uber_group: 53632, uber_id: 6869 },
    ),
    (
        "wellspringGroupDescriptor.secretWallC",
        UberIdentifier { uber_group: 53632, uber_id: 9366 },
    ),
    (
        "wellspringGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 53632, uber_id: 12019 },
    ),
    (
        "wellspringGroupDescriptor.lifeVesselA",
        UberIdentifier { uber_group: 53632, uber_id: 17403 },
    ),
    (
        "wellspringGroupDescriptor.orePickupA",
        UberIdentifier { uber_group: 53632, uber_id: 21124 },
    ),
    (
        "wellspringGroupDescriptor.smallExpA",
        UberIdentifier { uber_group: 53632, uber_id: 21790 },
    ),
    (
        "wellspringGroupDescriptor.wispSequencePlayed",
        UberIdentifier { uber_group: 53632, uber_id: 22486 },
    ),
    (
        "wellspringGroupDescriptor.orePickupB",
        UberIdentifier { uber_group: 53632, uber_id: 25556 },
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 53632, uber_id: 25817 },
    ),
    (
        "wellspringGroupDescriptor.leafPileA",
        UberIdentifier { uber_group: 53632, uber_id: 32197 },
    ),
    (
        "wellspringGroupDescriptor.expOrbG",
        UberIdentifier { uber_group: 53632, uber_id: 32785 },
    ),
    (
        "wellspringGroupDescriptor.spiritShard",
        UberIdentifier { uber_group: 53632, uber_id: 33168 },
    ),
    (
        "wellspringGroupDescriptor.secretWallB",
        UberIdentifier { uber_group: 53632, uber_id: 40587 },
    ),
    (
        "wellspringGroupDescriptor.questItemCompass",
        UberIdentifier { uber_group: 53632, uber_id: 41227 },
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderF",
        UberIdentifier { uber_group: 53632, uber_id: 42264 },
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderE",
        UberIdentifier { uber_group: 53632, uber_id: 51706 },
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 53632, uber_id: 51735 },
    ),
    (
        "wellspringGroupDescriptor.xpOrbUberState",
        UberIdentifier { uber_group: 53632, uber_id: 54915 },
    ),
    (
        "wellspringGroupDescriptor.mediumExpA",
        UberIdentifier { uber_group: 53632, uber_id: 56829 },
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 53632, uber_id: 58126 },
    ),
    (
        "wellspringGroupDescriptor.rotatingWheel",
        UberIdentifier { uber_group: 53632, uber_id: 61074 },
    ),
    (
        "wellspringGroupDescriptor.spiritShardA",
        UberIdentifier { uber_group: 53632, uber_id: 61128 },
    ),
    (
        "wellspringGroupDescriptor.mediumExpOrbPlaceholderC",
        UberIdentifier { uber_group: 53632, uber_id: 62356 },
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 53632, uber_id: 62781 },
    ),
    (
        "wellspringGroupDescriptor.secretWallA",
        UberIdentifier { uber_group: 53632, uber_id: 64763 },
    ),
    (
        "wellspringGroupDescriptor.savePedestal",
        UberIdentifier { uber_group: 53632, uber_id: 14947 },
    ),
    (
        "wellspringGroupDescriptor.savePedestalUberState",
        UberIdentifier { uber_group: 53632, uber_id: 18181 },
    ),
    (
        "wellspringGroupDescriptor.savePedestal",
        UberIdentifier { uber_group: 53632, uber_id: 53974 },
    ),
    (
        "wellspringGroupDescriptor.savePedestalUberState",
        UberIdentifier { uber_group: 53632, uber_id: 63074 },
    ),
    (
        "wellspringGroupDescriptor.showDoorCutsceneState",
        UberIdentifier { uber_group: 53632, uber_id: 26178 },
    ),
    (
        "prologueGroup.areaText",
        UberIdentifier { uber_group: 54846, uber_id: 27125 },
    ),
    (
        "_petrifiedForestGroup.xpOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 193 },
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 595 },
    ),
    (
        "_petrifiedForestGroup.keyStoneD",
        UberIdentifier { uber_group: 58674, uber_id: 780 },
    ),
    (
        "_petrifiedForestGroup.creepBlocker",
        UberIdentifier { uber_group: 58674, uber_id: 902 },
    ),
    (
        "_petrifiedForestGroup.keystoneDUberState",
        UberIdentifier { uber_group: 58674, uber_id: 1816 },
    ),
    (
        "_petrifiedForestGroup.keystoneBUberState",
        UberIdentifier { uber_group: 58674, uber_id: 2169 },
    ),
    (
        "_petrifiedForestGroup.keyStoneA",
        UberIdentifier { uber_group: 58674, uber_id: 2227 },
    ),
    (
        "_petrifiedForestGroup.areaText",
        UberIdentifier { uber_group: 58674, uber_id: 2317 },
    ),
    (
        "_petrifiedForestGroup.stompableFloorA",
        UberIdentifier { uber_group: 58674, uber_id: 2797 },
    ),
    (
        "_petrifiedForestGroup.stompableFloorB",
        UberIdentifier { uber_group: 58674, uber_id: 3577 },
    ),
    (
        "_petrifiedForestGroup.blowableFlameA",
        UberIdentifier { uber_group: 58674, uber_id: 5285 },
    ),
    (
        "_petrifiedForestGroup.xpOrbUberState",
        UberIdentifier { uber_group: 58674, uber_id: 6936 },
    ),
    (
        "_petrifiedForestGroup.petrifiedOwlStalkSequenceCompleted",
        UberIdentifier { uber_group: 58674, uber_id: 7636 },
    ),
    (
        "_petrifiedForestGroup.CollectibleXpA",
        UberIdentifier { uber_group: 58674, uber_id: 8487 },
    ),
    (
        "_petrifiedForestGroup.drillableWallA",
        UberIdentifier { uber_group: 58674, uber_id: 8810 },
    ),
    (
        "_petrifiedForestGroup.leafPileB",
        UberIdentifier { uber_group: 58674, uber_id: 9239 },
    ),
    (
        "_petrifiedForestGroup.energyContainerA",
        UberIdentifier { uber_group: 58674, uber_id: 9583 },
    ),
    (
        "_petrifiedForestGroup.expOrbC",
        UberIdentifier { uber_group: 58674, uber_id: 9881 },
    ),
    (
        "_petrifiedForestGroup.narratorLineShownHowl",
        UberIdentifier { uber_group: 58674, uber_id: 10677 },
    ),
    (
        "_petrifiedForestGroup.blowableFlameA",
        UberIdentifier { uber_group: 58674, uber_id: 10685 },
    ),
    (
        "_petrifiedForestGroup.leafPile",
        UberIdentifier { uber_group: 58674, uber_id: 10877 },
    ),
    (
        "_petrifiedForestGroup.stompableFloor",
        UberIdentifier { uber_group: 58674, uber_id: 11400 },
    ),
    (
        "_petrifiedForestGroup.keyStoneC",
        UberIdentifier { uber_group: 58674, uber_id: 11736 },
    ),
    (
        "_petrifiedForestGroup.hutDoorUnlocked",
        UberIdentifier { uber_group: 58674, uber_id: 14313 },
    ),
    (
        "_petrifiedForestGroup.powlVignettePlayed",
        UberIdentifier { uber_group: 58674, uber_id: 14539 },
    ),
    (
        "_petrifiedForestGroup.mediumPickupB",
        UberIdentifier { uber_group: 58674, uber_id: 14590 },
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier { uber_group: 58674, uber_id: 14593 },
    ),
    (
        "_petrifiedForestGroup.displayedGlideHint",
        UberIdentifier { uber_group: 58674, uber_id: 14912 },
    ),
    (
        "_petrifiedForestGroup.playedEpilogue",
        UberIdentifier { uber_group: 58674, uber_id: 15269 },
    ),
    (
        "_petrifiedForestGroup.keyStoneB",
        UberIdentifier { uber_group: 58674, uber_id: 17420 },
    ),
    (
        "_petrifiedForestGroup.blowableFlame",
        UberIdentifier { uber_group: 58674, uber_id: 17742 },
    ),
    (
        "_petrifiedForestGroup.smallPickupA",
        UberIdentifier { uber_group: 58674, uber_id: 17974 },
    ),
    (
        "_petrifiedForestGroup.lifeCellUberState",
        UberIdentifier { uber_group: 58674, uber_id: 18735 },
    ),
    (
        "_petrifiedForestGroup.mediumPickupA",
        UberIdentifier { uber_group: 58674, uber_id: 18924 },
    ),
    (
        "_petrifiedForestGroup.keyStoneB",
        UberIdentifier { uber_group: 58674, uber_id: 19769 },
    ),
    (
        "_petrifiedForestGroup.leafPile",
        UberIdentifier { uber_group: 58674, uber_id: 20143 },
    ),
    (
        "_petrifiedForestGroup.gorlekOreA",
        UberIdentifier { uber_group: 58674, uber_id: 20713 },
    ),
    (
        "_petrifiedForestGroup.boolean_gasBallBridge",
        UberIdentifier { uber_group: 58674, uber_id: 20724 },
    ),
    (
        "_petrifiedForestGroup.keystoneCUberState",
        UberIdentifier { uber_group: 58674, uber_id: 20944 },
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 20983 },
    ),
    (
        "_petrifiedForestGroup.leverGateA",
        UberIdentifier { uber_group: 58674, uber_id: 21139 },
    ),
    (
        "_petrifiedForestGroup.narratorLineShriekAttackShown",
        UberIdentifier { uber_group: 58674, uber_id: 21385 },
    ),
    (
        "_petrifiedForestGroup.doorState",
        UberIdentifier { uber_group: 58674, uber_id: 21500 },
    ),
    (
        "_petrifiedForestGroup.narratorLineShown",
        UberIdentifier { uber_group: 58674, uber_id: 22056 },
    ),
    (
        "_petrifiedForestGroup.xpOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 22472 },
    ),
    (
        "_petrifiedForestGroup.xpOrbUberState",
        UberIdentifier { uber_group: 58674, uber_id: 22503 },
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 23186 },
    ),
    (
        "_petrifiedForestGroup.lagoonBreakableFloor",
        UberIdentifier { uber_group: 58674, uber_id: 24457 },
    ),
    (
        "_petrifiedForestGroup.xpOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 24911 },
    ),
    (
        "_petrifiedForestGroup.gorlekOreA",
        UberIdentifier { uber_group: 58674, uber_id: 26274 },
    ),
    (
        "_petrifiedForestGroup.shardSlotA",
        UberIdentifier { uber_group: 58674, uber_id: 26282 },
    ),
    (
        "_petrifiedForestGroup.smallExpOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 26639 },
    ),
    (
        "_petrifiedForestGroup.gorlekOreA",
        UberIdentifier { uber_group: 58674, uber_id: 28710 },
    ),
    (
        "_petrifiedForestGroup.narrationPetrifiedOwlStalk",
        UberIdentifier { uber_group: 58674, uber_id: 29035 },
    ),
    (
        "_petrifiedForestGroup.shardSlotUpgradePlaceholder",
        UberIdentifier { uber_group: 58674, uber_id: 29265 },
    ),
    (
        "_petrifiedForestGroup.stompableFloor",
        UberIdentifier { uber_group: 58674, uber_id: 29622 },
    ),
    (
        "_petrifiedForestGroup.shardPickupA",
        UberIdentifier { uber_group: 58674, uber_id: 30377 },
    ),
    (
        "_petrifiedForestGroup.areaText",
        UberIdentifier { uber_group: 58674, uber_id: 30897 },
    ),
    (
        "_petrifiedForestGroup.expOrbC",
        UberIdentifier { uber_group: 58674, uber_id: 30908 },
    ),
    (
        "_petrifiedForestGroup.powlVignettePlayed",
        UberIdentifier { uber_group: 58674, uber_id: 32369 },
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 32647 },
    ),
    (
        "_petrifiedForestGroup.expOrbD",
        UberIdentifier { uber_group: 58674, uber_id: 33893 },
    ),
    (
        "_petrifiedForestGroup.breakableWallA",
        UberIdentifier { uber_group: 58674, uber_id: 33965 },
    ),
    (
        "_petrifiedForestGroup.diggableWallA",
        UberIdentifier { uber_group: 58674, uber_id: 34799 },
    ),
    (
        "_petrifiedForestGroup.expOrbB",
        UberIdentifier { uber_group: 58674, uber_id: 36199 },
    ),
    (
        "_petrifiedForestGroup.floatZoneState",
        UberIdentifier { uber_group: 58674, uber_id: 36832 },
    ),
    (
        "_petrifiedForestGroup.featherVignettePlayed",
        UberIdentifier { uber_group: 58674, uber_id: 36965 },
    ),
    (
        "_petrifiedForestGroup.expOrbB",
        UberIdentifier { uber_group: 58674, uber_id: 37006 },
    ),
    (
        "_petrifiedForestGroup.leafPile",
        UberIdentifier { uber_group: 58674, uber_id: 37037 },
    ),
    (
        "_petrifiedForestGroup.lifeCellUberState",
        UberIdentifier { uber_group: 58674, uber_id: 37128 },
    ),
    (
        "_petrifiedForestGroup.stompableFloorC",
        UberIdentifier { uber_group: 58674, uber_id: 37636 },
    ),
    (
        "_petrifiedForestGroup.mokiCleanWaterVignetteTriggered",
        UberIdentifier { uber_group: 58674, uber_id: 37811 },
    ),
    (
        "_petrifiedForestGroup.expOrb",
        UberIdentifier { uber_group: 58674, uber_id: 37885 },
    ),
    (
        "_petrifiedForestGroup.shardA",
        UberIdentifier { uber_group: 58674, uber_id: 38285 },
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier { uber_group: 58674, uber_id: 39950 },
    ),
    (
        "_petrifiedForestGroup.boolean_skeletonBoneA",
        UberIdentifier { uber_group: 58674, uber_id: 40066 },
    ),
    (
        "_petrifiedForestGroup.keyStoneD",
        UberIdentifier { uber_group: 58674, uber_id: 40073 },
    ),
    (
        "_petrifiedForestGroup.mokiFoulWaterVignetteTriggered",
        UberIdentifier { uber_group: 58674, uber_id: 41644 },
    ),
    (
        "_petrifiedForestGroup.expOrbC",
        UberIdentifier { uber_group: 58674, uber_id: 42158 },
    ),
    (
        "_petrifiedForestGroup.keyStoneA",
        UberIdentifier { uber_group: 58674, uber_id: 42531 },
    ),
    (
        "_petrifiedForestGroup.keyStoneC",
        UberIdentifier { uber_group: 58674, uber_id: 43033 },
    ),
    (
        "_petrifiedForestGroup.keystoneAUberState",
        UberIdentifier { uber_group: 58674, uber_id: 44215 },
    ),
    (
        "_petrifiedForestGroup.creepA",
        UberIdentifier { uber_group: 58674, uber_id: 44324 },
    ),
    (
        "_petrifiedForestGroup.escapeRocks",
        UberIdentifier { uber_group: 58674, uber_id: 44864 },
    ),
    (
        "_petrifiedForestGroup.collapsingSkeletonA",
        UberIdentifier { uber_group: 58674, uber_id: 46547 },
    ),
    (
        "_petrifiedForestGroup.petrifiedForestNewTransitionOriVignettePlayed",
        UberIdentifier { uber_group: 58674, uber_id: 46980 },
    ),
    (
        "_petrifiedForestGroup.setupDownB",
        UberIdentifier { uber_group: 58674, uber_id: 47179 },
    ),
    (
        "_petrifiedForestGroup.breakableGroundA",
        UberIdentifier { uber_group: 58674, uber_id: 47751 },
    ),
    (
        "_petrifiedForestGroup.creebBulb",
        UberIdentifier { uber_group: 58674, uber_id: 48394 },
    ),
    (
        "_petrifiedForestGroup.creepBall",
        UberIdentifier { uber_group: 58674, uber_id: 49272 },
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 49535 },
    ),
    (
        "_petrifiedForestGroup.boolean_skeletonBoneC",
        UberIdentifier { uber_group: 58674, uber_id: 50410 },
    ),
    (
        "_petrifiedForestGroup.boolean_skeletonBoneB",
        UberIdentifier { uber_group: 58674, uber_id: 51501 },
    ),
    (
        "_petrifiedForestGroup.shownHint",
        UberIdentifier { uber_group: 58674, uber_id: 51890 },
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier { uber_group: 58674, uber_id: 52280 },
    ),
    (
        "_petrifiedForestGroup.patrifiedForestBreakableFloor",
        UberIdentifier { uber_group: 58674, uber_id: 52345 },
    ),
    (
        "_petrifiedForestGroup.mediumPickupC",
        UberIdentifier { uber_group: 58674, uber_id: 54516 },
    ),
    (
        "_petrifiedForestGroup.stompableFloor",
        UberIdentifier { uber_group: 58674, uber_id: 54560 },
    ),
    (
        "_petrifiedForestGroup.breakableWall",
        UberIdentifier { uber_group: 58674, uber_id: 54686 },
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 55650 },
    ),
    (
        "_petrifiedForestGroup.enemyRoom",
        UberIdentifier { uber_group: 58674, uber_id: 56043 },
    ),
    (
        "_petrifiedForestGroup.wispCutscenePlayed",
        UberIdentifier { uber_group: 58674, uber_id: 58268 },
    ),
    (
        "_petrifiedForestGroup.shownFlapEnemyHint",
        UberIdentifier { uber_group: 58674, uber_id: 58684 },
    ),
    (
        "_petrifiedForestGroup.xpOrbB",
        UberIdentifier { uber_group: 58674, uber_id: 59372 },
    ),
    (
        "_petrifiedForestGroup.CollectibleXPB",
        UberIdentifier { uber_group: 58674, uber_id: 59691 },
    ),
    (
        "_petrifiedForestGroup.expOrbD",
        UberIdentifier { uber_group: 58674, uber_id: 59714 },
    ),
    (
        "_petrifiedForestGroup.skeletonState",
        UberIdentifier { uber_group: 58674, uber_id: 61252 },
    ),
    (
        "_petrifiedForestGroup.setupDownA",
        UberIdentifier { uber_group: 58674, uber_id: 61327 },
    ),
    (
        "_petrifiedForestGroup.stomableFloorB",
        UberIdentifier { uber_group: 58674, uber_id: 61391 },
    ),
    (
        "_petrifiedForestGroup.breakableHiddenWall",
        UberIdentifier { uber_group: 58674, uber_id: 61577 },
    ),
    (
        "_petrifiedForestGroup.clothBroken",
        UberIdentifier { uber_group: 58674, uber_id: 63837 },
    ),
    (
        "_petrifiedForestGroup.expOrbA",
        UberIdentifier { uber_group: 58674, uber_id: 64057 },
    ),
    (
        "_petrifiedForestGroup.expOrb",
        UberIdentifier { uber_group: 58674, uber_id: 64484 },
    ),
    (
        "_petrifiedForestGroup.secretWallA",
        UberIdentifier { uber_group: 58674, uber_id: 64690 },
    ),
    (
        "_petrifiedForestGroup.stomableFloorA",
        UberIdentifier { uber_group: 58674, uber_id: 65519 },
    ),
    (
        "_petrifiedForestGroup.savePedestalA",
        UberIdentifier { uber_group: 58674, uber_id: 1965 },
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier { uber_group: 58674, uber_id: 7071 },
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier { uber_group: 58674, uber_id: 10029 },
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier { uber_group: 58674, uber_id: 10997 },
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier { uber_group: 58674, uber_id: 11221 },
    ),
    (
        "_petrifiedForestGroup.savePedestalUberState",
        UberIdentifier { uber_group: 58674, uber_id: 36061 },
    ),
    (
        "_petrifiedForestGroup.chaseState",
        UberIdentifier { uber_group: 58674, uber_id: 32810 },
    ),
    (
        "_petrifiedForestGroup.petrifiedForestNewTransitionKuVignettePlayed",
        UberIdentifier { uber_group: 58674, uber_id: 44798 },
    ),
    (
        "_petrifiedForestGroup.petrifiedOwlClothState",
        UberIdentifier { uber_group: 58674, uber_id: 45819 },
    ),
    (
        "_petrifiedForestGroup.petrifiedOwlState",
        UberIdentifier { uber_group: 58674, uber_id: 61616 },
    ),
    (
        "shrineGroup.shrineLaser",
        UberIdentifier { uber_group: 61306, uber_id: 2129 },
    ),
    (
        "shrineGroup.shrineProjectile",
        UberIdentifier { uber_group: 61306, uber_id: 2239 },
    ),
    (
        "shrineGroup.shrineMouldwoodDepths",
        UberIdentifier { uber_group: 61306, uber_id: 18888 },
    ),
    (
        "shrineGroup.shrineHammer",
        UberIdentifier { uber_group: 61306, uber_id: 26590 },
    ),
    (
        "shrineGroup.shrinePortal",
        UberIdentifier { uber_group: 61306, uber_id: 40441 },
    ),
    (
        "shrineGroup.shrineTeleport",
        UberIdentifier { uber_group: 61306, uber_id: 52344 },
    ),
    (
        "shrineGroup.shrineOfFall",
        UberIdentifier { uber_group: 61306, uber_id: 56122 },
    ),
    (
        "spiderGroupDescriptor.spiderlingsQuestUberState",
        UberIdentifier { uber_group: 61314, uber_id: 55764 },
    ),
    (
        "spiderGroupDescriptor.spiderNpcState",
        UberIdentifier { uber_group: 61314, uber_id: 61458 },
    ),
    (
        "testUberStateGroupDescriptor.floatUberStateDescriptor",
        UberIdentifier { uber_group: 63018, uber_id: 22925 },
    ),
];
