use std::hash::BuildHasher;

use rustc_hash::FxBuildHasher;
use wotw_seedgen::data::{
    Shard, Skill, Teleporter, UberIdentifier, WeaponUpgrade,
    assets::{Pickups, States},
};

use crate::api::{Hash, logic::RelevantUberStates};

impl RelevantUberStates {
    pub fn new(pickups: &Pickups, states: &States) -> Self {
        const INVENTORY: [UberIdentifier; 81] = [
            UberIdentifier::SPIRIT_LIGHT,
            UberIdentifier::GORLEK_ORE,
            UberIdentifier::KEYSTONES,
            UberIdentifier::SHARD_SLOTS,
            UberIdentifier::CLEAN_WATER,
            UberIdentifier::BASE_MAX_HEALTH,
            UberIdentifier::BASE_MAX_ENERGY,
            Skill::BASH_ID,
            Skill::DOUBLE_JUMP_ID,
            Skill::LAUNCH_ID,
            Skill::GLIDE_ID,
            Skill::WATER_BREATH_ID,
            Skill::GRENADE_ID,
            Skill::GRAPPLE_ID,
            Skill::FLASH_ID,
            Skill::SPEAR_ID,
            Skill::REGENERATE_ID,
            Skill::BOW_ID,
            Skill::HAMMER_ID,
            Skill::SWORD_ID,
            Skill::BURROW_ID,
            Skill::DASH_ID,
            Skill::WATER_DASH_ID,
            Skill::SHURIKEN_ID,
            Skill::BLAZE_ID,
            Skill::SENTRY_ID,
            Skill::FLAP_ID,
            Skill::GLADES_ANCESTRAL_LIGHT_ID,
            Skill::MARSH_ANCESTRAL_LIGHT_ID,
            Shard::OVERCHARGE_ID,
            Shard::TRIPLE_JUMP_ID,
            Shard::WINGCLIP_ID,
            Shard::BOUNTY_ID,
            Shard::SWAP_ID,
            Shard::MAGNET_ID,
            Shard::SPLINTER_ID,
            Shard::RECKLESS_ID,
            Shard::QUICKSHOT_ID,
            Shard::RESILIENCE_ID,
            Shard::VITALITY_ID,
            Shard::LIFE_HARVEST_ID,
            Shard::ENERGY_HARVEST_ID,
            Shard::ENERGY_ID,
            Shard::LIFE_PACT_ID,
            Shard::LAST_STAND_ID,
            Shard::ULTRA_BASH_ID,
            Shard::ULTRA_GRAPPLE_ID,
            Shard::OVERFLOW_ID,
            Shard::THORN_ID,
            Shard::CATALYST_ID,
            Shard::TURMOIL_ID,
            Shard::STICKY_ID,
            Shard::FINESSE_ID,
            Shard::SPIRIT_SURGE_ID,
            Shard::LIFEFORCE_ID,
            Shard::DEFLECTOR_ID,
            Shard::FRACTURE_ID,
            Shard::ARCING_ID,
            Teleporter::MARSH_ID,
            Teleporter::DEN_ID,
            Teleporter::HOLLOW_ID,
            Teleporter::GLADES_ID,
            Teleporter::WELLSPRING_ID,
            Teleporter::BURROWS_ID,
            Teleporter::WOODS_ENTRANCE_ID,
            Teleporter::WOODS_EXIT_ID,
            Teleporter::REACH_ID,
            Teleporter::DEPTHS_ID,
            Teleporter::CENTRAL_POOLS_ID,
            Teleporter::POOLS_BOSS_ID,
            Teleporter::FEEDING_GROUNDS_ID,
            Teleporter::CENTRAL_WASTES_ID,
            Teleporter::UPPER_WASTES_ID,
            Teleporter::RUINS_ID,
            Teleporter::WILLOW_ID,
            Teleporter::SHRIEK_ID,
            WeaponUpgrade::EXPLODING_SPEAR_ID,
            WeaponUpgrade::SHOCK_HAMMER_ID,
            WeaponUpgrade::STATIC_SHURIKEN_ID,
            WeaponUpgrade::CHARGE_BLAZE_ID,
            WeaponUpgrade::RAPID_SENTRY_ID,
        ];

        let pickups = pickups.entries.iter().map(|entry| entry.uber_identifier);
        let states = states.entries.iter().map(|entry| entry.uber_identifier);

        let entrances = (1..=32).map(UberIdentifier::entrances);
        let entrances_known = (1..=32).map(UberIdentifier::entrances_known);

        let identifiers = pickups
            .chain(states)
            .chain(entrances)
            .chain(entrances_known)
            .chain(INVENTORY)
            .collect();

        let hash = Hash(FxBuildHasher.hash_one(&identifiers));

        Self { identifiers, hash }
    }
}
