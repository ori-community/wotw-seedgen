pub mod orbs;
pub mod uber_state;
pub mod constants;
pub mod icon;
pub(crate) mod extensions;

pub use orbs::Orbs;
pub use icon::{Icon, MapIcon};
use serde::{Serialize, Serializer, Deserialize};
use serde::ser::SerializeStruct;
pub use uber_state::{UberState, VUberState, UberIdentifier, UberType};

use decorum::R32;
use num_enum::{FromPrimitive, TryFromPrimitive};
use wotw_seedgen_derive::{VVariant, FromStr, Display};

use std::{
    fmt,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use crate::header::vdisplay;

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

#[derive(Debug, wotw_seedgen_derive::Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, FromPrimitive, FromStr)]
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
            _ => 0.0,
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
    pub fn code(&self) -> String {
        format!("{}|{}", self.x, self.y)
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

fn in_folder<P1: AsRef<Path>, P2: AsRef<Path>>(file: P1, folder: P2) -> Result<Option<PathBuf>, String> {
    let file = file.as_ref();
    let file_name = file.file_name().ok_or_else(|| format!("Invalid file path {}", file.display()))?;
    let file_location = file.parent().ok_or_else(|| format!("Invalid file path {}", file.display()))?;

    if let Some(file_folder) = file_location.file_name() {
        if folder.as_ref() == file_folder {
            return Ok(None);
        }
    }

    let mut in_folder = PathBuf::from(file_location);
    in_folder.push(folder);
    in_folder.push(file_name);

    Ok(Some(in_folder))
}

pub fn read_file<P1: AsRef<Path>, P2: AsRef<Path>>(file: P1, default_folder: P2) -> Result<String, String> {
    in_folder(&file, default_folder)?.map_or_else(
        || fs::read_to_string(&file).map_err(|err| format!("Failed to read file {}: {}", file.as_ref().display(), err)),
        |in_folder| fs::read_to_string(in_folder).or_else(|_| {
            fs::read_to_string(&file).map_err(|err| format!("Failed to read file {}: {}", file.as_ref().display(), err))
        })
    )
}

fn create_in_folder<P: AsRef<Path>>(file: P, contents: &str, create_new: bool) -> Result<PathBuf, io::Error> {
    let file = file.as_ref();
    let mut index = 0;
    loop {
        let mut filename = file.file_stem().unwrap().to_os_string();
        if index > 0 {
            filename.push(format!("_{}", index));
        }
        filename.push(format!(".{}", file.extension().unwrap().to_string_lossy()));
        let path = file.with_file_name(filename);

        match fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .create(true)
            .create_new(create_new)
            .open(&path) {
                Ok(mut file) => {
                    file.write_all(contents.as_bytes())?;
                    return Ok(path);
                },
                Err(err) if err.kind() == io::ErrorKind::AlreadyExists => index += 1,
                Err(err) if err.kind() == io::ErrorKind::NotFound => fs::create_dir_all(path.parent().unwrap())?,
                Err(err) => return Err(err),
            }
    }
}
pub fn create_file<P1: AsRef<Path>, P2: AsRef<Path>>(file: P1, contents: &str, default_folder: P2, create_new: bool) -> Result<PathBuf, String> {
    in_folder(&file, default_folder)?.map_or_else(
        || create_in_folder(&file, contents, create_new).map_err(|err| format!("Failed to create {}: {}", file.as_ref().display(), err)),
        |in_folder| create_in_folder(&in_folder, contents, create_new).or_else(|_| {
            create_in_folder(&file, contents, create_new).map_err(|err| format!("Failed to create {}: {}", file.as_ref().display(), err))
        })
    )
}
pub fn create_folder<P: AsRef<Path>>(file: P) -> Result<PathBuf, io::Error> {
    let file = file.as_ref();
    let mut index = 0;
    loop {
        let mut filename = file.file_stem().unwrap().to_os_string();
        if index > 0 {
            filename.push(format!("_{}", index));
        }
        let path = file.with_file_name(filename);

        match fs::create_dir(&path) {
            Ok(_) => return Ok(path),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => index += 1,
            Err(err) if err.kind() == io::ErrorKind::NotFound => fs::create_dir_all(path.parent().unwrap())?,
            Err(err) => return Err(err),
        }
    }
}

pub fn add_trailing_spaces(string: &mut String, target_length: usize) {
    let mut length = string.len();
    while target_length > length {
        string.push(' ');
        length += 1;
    }
}
pub fn with_leading_spaces(string: &str, target_length: usize) -> String {
    let mut length = string.len();
    let mut out = String::with_capacity(length);
    while target_length > length {
        out.push(' ');
        length += 1;
    }
    out += string;
    out
}

pub fn float_to_int(float: f32) -> Result<u32, String> {
    if float < u32::MIN as f32  || float > u32::MAX as f32 {
        return Err(format!("Failed to convert float to int: {float}"));
    }
    Ok(float as u32)
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
/// Returns [`None`] if the seed contains no information about the spawn location
pub fn spawn_from_seed(input: &str) -> Option<String> {
    input.lines()
        .find_map(|line| line.strip_prefix("Spawn: ")
        .and_then(|spawn| spawn.split_once("//")
        .map(|(_, identifier)| identifier.trim().to_string())))
}
