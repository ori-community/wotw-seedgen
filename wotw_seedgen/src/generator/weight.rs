use wotw_seedgen_data::{CommonUberIdentifier, Skill, Teleporter, UberIdentifier, WeaponUpgrade};

use super::placement::{PREFERRED_SPAWN_SLOTS, SPAWN_SLOTS};

const SECONDARY_SPAWN_SLOTS: usize = SPAWN_SLOTS - PREFERRED_SPAWN_SLOTS;

pub fn weight(
    new_reached: usize,
    uber_identifier: UberIdentifier,
    amount: usize,
    used_slots: usize,
    slots: usize,
) -> f32 {
    // TODO default cost?
    let cost = CommonUberIdentifier::from_uber_identifier(uber_identifier).map_or(0, cost);
    let mut weight = (new_reached + 1) as f32 / (cost * amount) as f32;

    debug_assert!(
        weight.is_finite(),
        "infinite weight: ({new_reached} + 1) / ({cost} * {amount}) = {weight}"
    );

    // TODO make it less likely to use spawn slots for later progressions?
    let begrudgingly_used_slots = (used_slots + SECONDARY_SPAWN_SLOTS).saturating_sub(slots);
    weight *= (0.3_f32).powf(begrudgingly_used_slots as f32);

    weight
}

pub fn cost(uber_identifier: CommonUberIdentifier) -> usize {
    match uber_identifier {
        CommonUberIdentifier::SpiritLight => 1,
        CommonUberIdentifier::GorlekOre => 20, // TODO I wonder why there's so much gorlek ore progression oriLol
        CommonUberIdentifier::MaxHealth | CommonUberIdentifier::MaxEnergy => 120, // adjust once weighting accounts for the modification amount
        CommonUberIdentifier::Skill(Skill::Regenerate)
        | CommonUberIdentifier::Skill(Skill::WaterBreath) => 200, // Quality-of-Life Skills
        CommonUberIdentifier::Keystones => 320,
        CommonUberIdentifier::WeaponUpgrade(WeaponUpgrade::ExplodingSpear)
        | CommonUberIdentifier::WeaponUpgrade(WeaponUpgrade::HammerShockwave)
        | CommonUberIdentifier::WeaponUpgrade(WeaponUpgrade::StaticShuriken)
        | CommonUberIdentifier::WeaponUpgrade(WeaponUpgrade::ChargeBlaze)
        | CommonUberIdentifier::WeaponUpgrade(WeaponUpgrade::RapidSentry) => 400,
        CommonUberIdentifier::ShardSlots => 480,
        CommonUberIdentifier::Skill(Skill::GladesAncestralLight)
        | CommonUberIdentifier::Skill(Skill::MarshAncestralLight)
        | CommonUberIdentifier::Shard(_) => 1000,
        CommonUberIdentifier::Skill(Skill::Dash) | CommonUberIdentifier::Skill(Skill::Flap) => 1200, // Counteracting bias because these unlock rather little
        CommonUberIdentifier::Skill(Skill::Glide) | CommonUberIdentifier::Skill(Skill::Grapple) => {
            1400
        } // Feel-Good Finds
        CommonUberIdentifier::Skill(Skill::Sword)
        | CommonUberIdentifier::Skill(Skill::Hammer)
        | CommonUberIdentifier::Skill(Skill::Bow)
        | CommonUberIdentifier::Skill(Skill::Shuriken) => 1600, // Basic Weapons
        CommonUberIdentifier::Skill(Skill::Burrow)
        | CommonUberIdentifier::Skill(Skill::WaterDash)
        | CommonUberIdentifier::Skill(Skill::Grenade)
        | CommonUberIdentifier::Skill(Skill::Flash)
        | CommonUberIdentifier::CleanWater => 1800, // Key Skills
        CommonUberIdentifier::Skill(Skill::DoubleJump) => 2000, // Good to find, but this is already biased for by being powerful
        CommonUberIdentifier::Skill(Skill::Blaze) | CommonUberIdentifier::Skill(Skill::Sentry) => {
            2800
        } // Tedious Weapons
        CommonUberIdentifier::Skill(Skill::Bash) => 3000, // Counteracting bias because Bash unlocks a lot
        CommonUberIdentifier::Skill(Skill::Spear) => 4000, // Lowering the frequency of slow Spear starts
        CommonUberIdentifier::Teleporter(Teleporter::Den)
        | CommonUberIdentifier::Teleporter(Teleporter::Hollow)
        | CommonUberIdentifier::Teleporter(Teleporter::Glades)
        | CommonUberIdentifier::Teleporter(Teleporter::Wellspring)
        | CommonUberIdentifier::Teleporter(Teleporter::Burrows)
        | CommonUberIdentifier::Teleporter(Teleporter::WoodsEntrance)
        | CommonUberIdentifier::Teleporter(Teleporter::WoodsExit)
        | CommonUberIdentifier::Teleporter(Teleporter::Reach)
        | CommonUberIdentifier::Teleporter(Teleporter::Depths)
        | CommonUberIdentifier::Teleporter(Teleporter::CentralPools)
        | CommonUberIdentifier::Teleporter(Teleporter::PoolsBoss)
        | CommonUberIdentifier::Teleporter(Teleporter::FeedingGrounds)
        | CommonUberIdentifier::Teleporter(Teleporter::CentralWastes)
        | CommonUberIdentifier::Teleporter(Teleporter::OuterRuins)
        | CommonUberIdentifier::Teleporter(Teleporter::InnerRuins)
        | CommonUberIdentifier::Teleporter(Teleporter::Willow)
        | CommonUberIdentifier::Teleporter(Teleporter::Shriek) => 25000,
        CommonUberIdentifier::Teleporter(Teleporter::Marsh) => 30000,
        CommonUberIdentifier::Skill(Skill::Launch) => 40000, // Absolutely Broken
        _ => 0,
    }
}
