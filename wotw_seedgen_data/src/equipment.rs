use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, VariantArray};
use wotw_seedgen_derive::FromStr;

/// Available slots for [`Equipment`] (the stuff displayed at the bottom between your energy and health)
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize_repr,
    Serialize_repr,
    Display,
    FromStr,
    VariantArray,
)]
#[repr(u8)]
pub enum EquipSlot {
    /// Bottom Left equipment slot
    Ability1 = 0,
    /// Top equipment slot
    Ability2 = 1,
    /// Bottom right equipment slot
    Ability3 = 2,
}
// TODO try for better error messages than the strum default
// TODO test equipments and document variants, some don't actually need to be equipped? how does that work
// Source: https://github.com/ori-community/wotw-rando-client/blob/main/projects/Modloader/app/structs/EquipmentType__Enum.h
/// Abilities which have to be equipped before use
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Deserialize_repr,
    Serialize_repr,
    Display,
    FromStr,
    VariantArray,
)]
#[repr(u16)]
pub enum Equipment {
    Hammer = 1000,           // Weapon_Hammer
    Bow = 1001,              // Weapon_Bow
    Sword = 1002,            // Weapon_Sword
    Torch = 1003,            // Weapon_Torch
    Swordstaff = 1004,       // Weapon_Swordstaff
    Chainsword = 1005,       // Weapon_Chainsword
    Shot = 2000,             // Spell_Shot
    HomingMissiles = 2001,   // Spell_HomingMissiles
    Wave = 2002,             // Spell_Wave
    Whirl = 2003,            // Spell_Whirl
    Flash = 2004,            // Spell_Glow
    LockOn = 2005,           // Spell_LockOn
    Shield = 2006,           // Spell_Shield
    Invisibility = 2007,     // Spell_Invisibility
    LifeAbsorb = 2008,       // Spell_LifeAbsorb
    Shards = 2009,           // Spell_Shards
    Grenade = 2010,          // Spell_StickyMine
    SentryDeprecated = 2011, // Spell_Sentry
    Spear = 2012,            // Spell_Spear
    Regenerate = 2013,       // Spell_Meditate
    Teleport = 2014,         // Spell_Teleport
    Shuriken = 2015,         // Spell_Chakram
    Blaze = 2016,            // Spell_Blaze
    Sentry = 2017,           // Spell_Turret
    Sein = 2018,             // Spell_GoldenSein
    Launch = 2019,           // Spell_ChargeJump
    Bash = 3000,             // Ability_Bash
    Grapple = 3001,          // Ability_Leash
    Burrow = 3002,           // Ability_Digging
    Drill = 3003,            // Ability_Drill
    DoubleJump = 3004,       // Ability_DoubleJump
    Flap = 3005,             // Ability_FeatherFlap
    Dash = 4000,             // AutoAbility_Dash
    Bounce = 4001,           // AutoAbility_Bounce
    Glide = 4002,            // AutoAbility_Glide
    ChargeJump = 4003,       // AutoAbility_ChargeJump
    WaterDash = 4004,        // AutoAbility_WaterDash
    Climb = 4005,            // AutoAbility_Climb
    WeaponCharge = 4006,     // AutoAbility_WeaponCharge
    DamageUpgradeA = 4007,   // AutoAbility_DamageUpgradeA
    DamageUpgradeB = 4008,   // AutoAbility_DamageUpgradeB
    WaterBreath = 4009,      // AutoAbility_WaterBreath
}
