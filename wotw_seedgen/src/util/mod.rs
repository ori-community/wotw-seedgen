pub mod orbs;
pub mod constants;
pub mod icon;
pub(crate) mod extensions;

pub use orbs::{Orbs, OrbVariants};
pub use icon::{Icon, MapIcon};
use serde::{Serialize, Deserialize};

use decorum::R32;
use num_enum::{FromPrimitive, TryFromPrimitive};
use wotw_seedgen_derive::{VVariant, FromStr, Display};

use std::fmt;

use crate::header::{vdisplay, CodeDisplay};

use self::constants::DEFAULT_SPAWN;

#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u8)]
pub enum NumericBool {
    False = 0,
    True = 1,
}

#[derive(Debug, Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
#[repr(u16)]
pub enum Spell {
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

#[derive(Debug, wotw_seedgen_derive::Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize, FromPrimitive, FromStr)]
#[repr(u8)]
pub enum Zone {
    Marsh = 0,
    Hollow = 1,
    Glades = 2,
    Wellspring = 3,
    Woods = 7,
    Reach = 6,
    Depths = 8,
    Pools = 4,
    Wastes = 9,
    Ruins = 10,
    Willow = 11,
    Burrows = 5,
    Spawn = 14,
    Shop = 12,
    #[num_enum(default)]
    Void = 13,
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, FromStr)]
#[ParseFromIdentifier]
pub enum Enemy {
    Mantis,
    Slug,
    WeakSlug,
    BombSlug,
    CorruptSlug,
    SneezeSlug,
    ShieldSlug,
    Lizard,
    Bat,
    Hornbug,
    Skeeto,
    SmallSkeeto,
    Bee,
    Nest,
    Crab,
    SpinCrab,
    Tentacle,
    Balloon,
    Miner,
    MaceMiner,
    ShieldMiner,
    CrystalMiner,
    ShieldCrystalMiner,
    Sandworm,
    Spiderling,
    EnergyRefill,
}
impl Enemy {
    pub fn health(self) -> f32 {
        match self {
            Enemy::BombSlug | Enemy::CorruptSlug | Enemy::Balloon => 1.0,
            Enemy::SmallSkeeto => 8.0,
            Enemy::WeakSlug | Enemy::Spiderling => 12.0,
            Enemy::Slug => 13.0,
            Enemy::Skeeto | Enemy::Sandworm | Enemy::Tentacle => 20.0,
            Enemy::ShieldSlug | Enemy::Lizard | Enemy::Bee => 24.0,
            Enemy::Nest => 25.0,
            Enemy::Mantis | Enemy::SneezeSlug | Enemy::Bat | Enemy::Crab | Enemy::SpinCrab => 32.0,
            Enemy::Hornbug | Enemy::Miner => 40.0,
            Enemy::ShieldCrystalMiner => 50.0,
            Enemy::MaceMiner | Enemy::ShieldMiner => 60.0,
            Enemy::CrystalMiner => 80.0,
            Enemy::EnergyRefill => 0.0,
        }
    }
    pub fn shielded(self) -> bool {
        matches!(self, Enemy::Hornbug | Enemy::ShieldSlug | Enemy::ShieldMiner | Enemy::ShieldCrystalMiner)
    }
    pub fn armored(self) -> bool {
        matches!(self, Enemy::Tentacle)
    }
    pub fn aerial(self) -> bool {  // whether we consider the enemy flying for movement restriction purposes
        matches!(self, Enemy::Bat | Enemy::Skeeto | Enemy::SmallSkeeto | Enemy::Bee | Enemy::Nest | Enemy::Tentacle)
    }
    pub fn flying(self) -> bool {  // whether the game considers the enemy flying for wingclip
        matches!(self, Enemy::Skeeto | Enemy::SmallSkeeto | Enemy::Bee)
    }
    pub fn ranged(self) -> bool {  // whether you need a ranged weapon
        matches!(self, Enemy::BombSlug | Enemy::CorruptSlug | Enemy::Balloon | Enemy::Bat)
    }
    pub fn dangerous(self) -> bool {
        matches!(self, Enemy::SneezeSlug | Enemy::Hornbug | Enemy::Crab | Enemy::SpinCrab | Enemy::Miner | Enemy::MaceMiner | Enemy::ShieldMiner | Enemy::CrystalMiner | Enemy::ShieldCrystalMiner)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum RefillValue {
    Full,
    Checkpoint,
    Health(f32),
    Energy(f32),
}
#[derive(Debug, Clone, PartialEq, Display)]
pub enum NodeKind {
    Anchor,
    Pickup,
    State,
    Quest,
}

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant, Serialize, Deserialize)]
#[serde(into = "SerdePosition", from = "SerdePosition")]
pub struct Position {
    #[VWrap]
    pub x: R32,
    #[VWrap]
    pub y: R32,
}
#[derive(Serialize, Deserialize)]
struct SerdePosition {
    x: f32,
    y: f32,
}
impl From<Position> for SerdePosition {
    fn from(position: Position) -> SerdePosition {
        SerdePosition {
            x: position.x.into(),
            y: position.y.into(),
        }
    }
}
impl From<SerdePosition> for Position {
    fn from(position: SerdePosition) -> Position {
        Position {
            x: position.x.into(),
            y: position.y.into(),
        }
    }
}
impl Position {
    /// Returns a new [`Position`] with the given coordinates
    /// 
    /// # Panics
    /// 
    /// Panics if either coordinate is not a real number
    pub fn new(x: f32, y: f32) -> Position {
        Position { x: x.into(), y: y.into() }
    }
    pub fn code(&self) -> CodeDisplay<Position> {
        CodeDisplay::new(self, |s, f| { write!(f, "{}|{}", s.x, s.y)})
    }
}
vdisplay! {
    VPosition,
    impl fmt::Display for Position {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            write!(f, "{}, {}", self.x, self.y)
        }
    }
}

pub(crate) fn add_trailing_spaces(string: &mut String, target_length: usize) {
    let mut length = string.len();
    while target_length > length {
        string.push(' ');
        length += 1;
    }
}

#[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
pub(crate) fn float_to_int(float: f32) -> Result<u32, String> {
    const MIN: f32 = u32::MIN as f32;
    const MAX: f32 = u32::MAX as f32;

    if (MIN..=MAX).contains(&float) {
        return Ok(float as u32);
    }
    Err(format!("Failed to convert float to int: {float}"))
}

pub(crate) fn float_to_real(float: f32) -> Result<R32, String> {
    match float.is_finite() {
        true => Ok(float.into()),
        false => Err(format!("Expected finite number, found {float}")),
    }
}

/// Read the spawn location from a generated seed
/// 
/// This reads the final spawn location, e.g. if the settings declared a random spawn, this will read the spawn that was chosen
/// Returns an error if the seed contains a Spawn but doesn't annotate its identifier
pub fn spawn_from_seed(input: &str) -> Result<String, String> {
    input.lines()
        .find_map(|line| line.strip_prefix("Spawn: ")
        .map(|spawn| spawn.split_once("//")
        .ok_or_else(|| "Failed to read spawn location from seed".to_string())
        .map(|(_, identifier)| identifier.trim().to_string())))
        .unwrap_or_else(|| Ok(DEFAULT_SPAWN.to_string()))
}
