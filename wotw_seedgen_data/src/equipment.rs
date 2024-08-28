use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, EnumString};

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
    EnumString,
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
    EnumString,
)]
#[repr(u16)]
pub enum Equipment {
    Hammer = 1000,
    Bow = 1001,
    Sword = 1002,
    Torch = 1003,
    Swordstaff = 1004,
    Chainsword = 1005,
    Shot = 2000,
    HomingMissiles = 2001,
    Wave = 2002,
    Whirl = 2003,
    Glow = 2004,
    LockOn = 2005,
    Shield = 2006,
    Invisibility = 2007,
    LifeAbsorb = 2008,
    Shards = 2009,
    Grenade = 2010,
    Sentry = 2011,
    Spear = 2012,
    Regenerate = 2013,
    Teleport = 2014,
    Shuriken = 2015,
    Blaze = 2016,
    Turret = 2017,
    Sein = 2018,
    Launch = 2019,
    Bash = 3000,
    Grapple = 3001,
    Burrow = 3002,
    Drill = 3003,
    DoubleJump = 3004,
    Flap = 3005,
    Dash = 4000,
    Bounce = 4001,
    Glide = 4002,
    ChargeJump = 4003,
    WaterDash = 4004,
    Climb = 4005,
    WeaponCharge = 4006,
    DamageUpgradeA = 4007,
    DamageUpgradeB = 4008,
    WaterBreath = 4009,
}
