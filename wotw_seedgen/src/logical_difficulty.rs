use std::{iter::Copied, slice};

use wotw_seedgen_data::{Difficulty, Shard, Skill, UberIdentifier};

pub trait LogicalDifficulty: Sized {
    /// Allow using Triple Jump
    fn triple_jump(self) -> bool;

    /// Allow using Resilience
    fn resilience(self) -> bool;

    /// Allow using Vitality
    fn vitality(self) -> bool;

    /// Allow using Energy Shard
    fn energy_shard(self) -> bool;

    /// Allow using Damage Buffs
    fn damage_buffs(self) -> bool;

    /// Allow using Overcharge
    fn overcharge(self) -> bool;

    /// Allow using Life Pact
    fn life_pact(self) -> bool;

    /// Allow using Ultra Bash
    fn ultra_bash(self) -> bool;

    /// Allow using Overflow
    fn overflow(self) -> bool;

    /// Allow using Thorn
    fn thorn(self) -> bool;

    /// Allow using Catalyst
    fn catalyst(self) -> bool;

    /// Allow charging Grenade
    fn charge_grenade(self) -> bool;

    fn may_increase_orbs(self, uber_identifier: UberIdentifier) -> bool;

    /// Allowed spawns on this difficulty when using the random spawn setting
    fn spawn_locations(self) -> &'static [&'static str];

    /// Allowed weapons on this difficulty
    fn weapons<const TARGET_IS_WALL: bool>(self) -> &'static [Skill];

    /// Allowed weapons on this difficulty
    fn weapons_iter<'a, const TARGET_IS_WALL: bool>(self) -> Copied<slice::Iter<'a, Skill>> {
        self.weapons::<TARGET_IS_WALL>().iter().copied()
    }

    /// Allowed ranged weapons on this difficulty
    fn ranged_weapons(self) -> &'static [Skill];

    /// Allowed ranged weapons on this difficulty
    fn ranged_weapons_iter<'a>(self) -> Copied<slice::Iter<'a, Skill>> {
        self.ranged_weapons().iter().copied()
    }
}

impl LogicalDifficulty for Difficulty {
    fn triple_jump(self) -> bool {
        self >= Difficulty::Gorlek
    }

    fn resilience(self) -> bool {
        self >= Difficulty::Gorlek
    }

    fn vitality(self) -> bool {
        self >= Difficulty::Gorlek
    }

    fn energy_shard(self) -> bool {
        self >= Difficulty::Gorlek
    }

    fn damage_buffs(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn overcharge(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn life_pact(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn ultra_bash(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn overflow(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn thorn(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn catalyst(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn charge_grenade(self) -> bool {
        self >= Difficulty::Unsafe
    }

    fn may_increase_orbs(self, uber_identifier: UberIdentifier) -> bool {
        match uber_identifier {
            UberIdentifier::MAX_HEALTH | UberIdentifier::MAX_ENERGY | Skill::REGENERATE_ID => true,
            Shard::RESILIENCE_ID => self.resilience(),
            Shard::VITALITY_ID => self.vitality(),
            Shard::ENERGY_ID => self.energy_shard(),
            Shard::OVERCHARGE_ID => self.overcharge(),
            Shard::LIFE_PACT_ID => self.life_pact(),
            Shard::OVERFLOW_ID => self.overflow(),
            Shard::CATALYST_ID => self.catalyst(),
            _ => false,
        }
    }

    // TODO seeing all these hardcoded strings makes me sad
    fn spawn_locations(self) -> &'static [&'static str] {
        match self {
            Difficulty::Moki => &[
                "MarshSpawn.Main",
                "HowlsDen.Teleporter",
                "GladesTown.Teleporter",
                "InnerWellspring.Teleporter",
                "MidnightBurrows.Teleporter",
            ],
            _ => &[
                "MarshSpawn.Main",
                "HowlsDen.Teleporter",
                "EastHollow.Teleporter",
                "GladesTown.Teleporter",
                "InnerWellspring.Teleporter",
                "MidnightBurrows.Teleporter",
                "WoodsEntry.Teleporter",
                "WoodsMain.Teleporter",
                "LowerReach.Teleporter",
                "UpperDepths.Teleporter",
                "EastPools.Teleporter",
                "LowerWastes.WestTP",
                "LowerWastes.EastTP",
            ],
        }
    }

    fn weapons<const TARGET_IS_WALL: bool>(self) -> &'static [Skill] {
        const WEAPONS: [Skill; 9] = [
            Skill::Sword,
            Skill::Hammer,
            Skill::Bow,
            Skill::Grenade,
            Skill::Shuriken,
            Skill::Blaze,
            Skill::Spear,
            Skill::Flash,
            Skill::Sentry,
        ];

        const NO_SENTRY: [Skill; WEAPONS.len() - 1] = *WEAPONS.first_chunk().unwrap();

        const NO_FLASH: [Skill; WEAPONS.len() - 1] = {
            let mut no_flash = NO_SENTRY;
            no_flash[7] = Skill::Sentry;
            no_flash
        };

        const NO_FLASH_OR_SENTRY: [Skill; WEAPONS.len() - 2] = *WEAPONS.first_chunk().unwrap();

        match (self >= Difficulty::Unsafe, TARGET_IS_WALL) {
            (false, false) => &NO_SENTRY,
            (false, true) => &NO_FLASH_OR_SENTRY,
            (true, false) => &NO_FLASH,
            (true, true) => &WEAPONS,
        }
    }

    fn ranged_weapons(self) -> &'static [Skill] {
        const WEAPONS: [Skill; 6] = [
            Skill::Bow,
            Skill::Spear,
            Skill::Grenade,
            Skill::Shuriken,
            Skill::Flash,
            Skill::Blaze,
        ];

        const NO_SHORT_RANGE: [Skill; 4] = *WEAPONS.first_chunk().unwrap();

        const NO_MID_RANGE: [Skill; 2] = *NO_SHORT_RANGE.first_chunk().unwrap();

        if self >= Difficulty::Gorlek {
            if self >= Difficulty::Unsafe {
                &WEAPONS
            } else {
                &NO_SHORT_RANGE
            }
        } else {
            &NO_MID_RANGE
        }
    }
}

pub const SHIELD_WEAPONS: [Skill; 4] = [Skill::Hammer, Skill::Launch, Skill::Grenade, Skill::Spear];
