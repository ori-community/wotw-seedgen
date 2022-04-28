pub mod orbs;
pub mod uber_state;
pub mod constants;

use decorum::R32;
use num_enum::FromPrimitive;
pub use uber_state::{UberState, UberIdentifier, UberType};

use std::{
    fmt,
    fs,
    io::{self, Write},
    path::{Path, PathBuf},
};

use serde::{Serialize, Deserialize};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Difficulty {
    Moki,
    Gorlek,
    Kii,
    Unsafe,
}
impl Default for Difficulty {
    fn default() -> Difficulty { Difficulty::Moki }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, Serialize, Deserialize)]
pub enum Glitch {
    SwordSentryJump,    // Grounded Sentry Jumps with Sword
    HammerSentryJump,   // Grounded Sentry Jump with Hammer
    ShurikenBreak,      // Breaking Walls from behind with Shuriken
    SentryBreak,        // Breaking Walls from behind with Sentry
    HammerBreak,        // Breaking Walls from behind with Hammer
    SpearBreak,         // Breaking Walls from behind with Spear
    SentryBurn,         // Melting Ice using Sentries
    RemoveKillPlane,    // Removing Shriek's Killplane at Feeding Grounds
    LaunchSwap,         // Using the weapon wheel to cancel Launch
    SentrySwap,         // Using the weapon wheel to cancel Sentry
    FlashSwap,          // Using the weapon wheel to cancel Flash
    BlazeSwap,          // Using the weapon wheel to cancel Blaze
    WaveDash,           // Gaining speed off a wall with Regenerate and Dash
    GrenadeJump,        // Preserving jump momentum with Grenade
    HammerJump,         // Preserving Double Jump momentum with Hammer
    SwordJump,          // Preserving Double Jump momentum with Sword
    GrenadeRedirect,    // Redirecting projectiles with Grenade
    SentryRedirect,     // Redirecting projectiles with Sentry
    PauseHover,         // Cancelling falling momentum through the pause menu
    GlideJump,          // Storing a grounded jump into the air with Glide
    GlideHammerJump,    // Preserving Glide Jump momentum with Hammer
    SpearJump,          // Storing a grounded jump into the air with Spear
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub enum GoalMode {
    Wisps,
    Trees,
    Quests,
    Relics(usize),
    RelicChance(f64),
}
impl fmt::Display for GoalMode {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            GoalMode::Wisps => write!(f, "ForceWisps"),
            GoalMode::Trees => write!(f, "ForceTrees"),
            GoalMode::Quests => write!(f, "ForceQuests"),
            GoalMode::Relics(_) | GoalMode::RelicChance(_) => write!(f, "WorldTour"),
        }
    }
}

#[inline]
pub fn auto_display<D: fmt::Debug>(debug: D) -> String {
    let mut debug = format!("{:?}", debug);

    let mut indices = Vec::new();

    for (index, _) in debug.match_indices(char::is_uppercase) {
        if index > 0 {
            indices.push(index);
        }
    }
    indices.reverse();
    for index in indices {
        debug.insert(index, ' ');
    }

    debug
}

#[macro_export]
macro_rules! auto_display {
    ($type:ty) => {
        impl std::fmt::Display for $type {
            fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
                write!(f, "{}", $crate::util::auto_display(self))
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, FromPrimitive)]
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
auto_display!(Zone);

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Icon {
    Shard(u16),
    Spell(u16),
    Opher(u16),
    Lupo(u16),
    Grom(u16),
    Tuley(u16),
    File(String),
}
impl fmt::Display for Icon {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Icon::Shard(id) => write!(f, "shard:{}", id),
            Icon::Spell(id) => write!(f, "spell:{}", id),
            Icon::Opher(id) => write!(f, "opher:{}", id),
            Icon::Lupo(id) => write!(f, "lupo:{}", id),
            Icon::Grom(id) => write!(f, "grom:{}", id),
            Icon::Tuley(id) => write!(f, "tuley:{}", id),
            Icon::File(path) => write!(f, "file:{}", path),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
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
pub enum RefillType {
    Full,
    Checkpoint,
    Health(f32),
    Energy(f32),
}
#[derive(Debug, PartialEq)]
pub enum NodeType {
    Anchor,
    Pickup,
    State,
    Quest,
}

#[derive(Debug, Default, PartialEq, Eq, Hash, Clone)]
pub struct Position {
    pub x: R32,
    pub y: R32,
}
impl Position {
    pub fn code(&self) -> String {
        format!("{}|{}", self.x, self.y)
    }
}

impl fmt::Display for Position {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}, {}", self.x, self.y)
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

pub fn float_to_int(float: f32) -> Result<u16, String> {
    if float < u16::MIN.into() || float > u16::MAX.into() {
        return Err(format!("Failed to convert float to int: {}", float));
    }
    Ok(float as u16)
}
