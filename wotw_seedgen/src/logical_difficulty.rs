use smallvec::{smallvec, SmallVec};
use wotw_seedgen_data::{Difficulty, Skill};

// TODO make trait and put on Difficulty?

pub const TRIPLE_JUMP: Difficulty = Difficulty::Gorlek;
pub const RESILIENCE: Difficulty = Difficulty::Gorlek;
pub const VITALITY: Difficulty = Difficulty::Gorlek;
pub const ENERGY_SHARD: Difficulty = Difficulty::Gorlek;
pub const DAMAGE_BUFFS: Difficulty = Difficulty::Unsafe;
pub const OVERCHARGE: Difficulty = Difficulty::Unsafe;
pub const LIFE_PACT: Difficulty = Difficulty::Unsafe;
pub const ULTRA_BASH: Difficulty = Difficulty::Unsafe;
pub const OVERFLOW: Difficulty = Difficulty::Unsafe;
pub const THORN: Difficulty = Difficulty::Unsafe;
pub const CATALYST: Difficulty = Difficulty::Unsafe;

pub const CHARGE_GRENADE: Difficulty = Difficulty::Unsafe;

// TODO seeing all these hardcoded strings makes me sad
/// Allowed spawns on this difficulty when using the random spawn setting
pub const fn spawn_locations(difficulty: Difficulty) -> &'static [&'static str] {
    match difficulty {
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

// TODO would it be worth to precompile the resulting slices for all variants?
/// Allowed weapons on this difficulty
pub fn weapons<const TARGET_IS_WALL: bool>(difficulty: Difficulty) -> SmallVec<[Skill; 9]> {
    let mut weapons = smallvec![
        Skill::Sword,
        Skill::Hammer,
        Skill::Bow,
        Skill::Grenade,
        Skill::Shuriken,
        Skill::Blaze,
        Skill::Spear,
    ];
    if !TARGET_IS_WALL {
        weapons.push(Skill::Flash);
    }
    if difficulty >= Difficulty::Unsafe {
        weapons.push(Skill::Sentry);
    }
    weapons
}

/// Allowed ranged weapons on this difficulty
pub fn ranged_weapons(difficulty: Difficulty) -> SmallVec<[Skill; 6]> {
    let mut weapons = smallvec![Skill::Bow, Skill::Spear];
    if difficulty >= Difficulty::Gorlek {
        weapons.push(Skill::Grenade);
        weapons.push(Skill::Shuriken);
        if difficulty >= Difficulty::Unsafe {
            weapons.push(Skill::Flash);
            weapons.push(Skill::Blaze);
        }
    }
    weapons
}

/// Allowed shield weapons
pub fn shield_weapons() -> SmallVec<[Skill; 4]> {
    smallvec![Skill::Hammer, Skill::Launch, Skill::Grenade, Skill::Spear]
}
