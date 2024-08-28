use crate::{Equipment, UberIdentifier};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, EnumString, FromRepr};

/// Skills, sometimes also called Abilities
///
/// Currently excludes unused skills
// TODO ^ why?
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
    FromRepr,
)]
#[repr(u8)]
pub enum Skill {
    Bash = 0,
    // ChargeFlame = 2,
    WallJump = 3,
    // Stomp = 4,
    DoubleJump = 5,
    Launch = 8,
    // Magnet = 10,
    // UltraMagnet = 11,
    // Climb = 12,
    Glide = 14,
    SpiritFlame = 15,
    // RapidFlame = 17,
    // SplitFlameUpgrade = 18,
    // SoulEfficiency = 22,
    WaterBreath = 23,
    // ChargeFlameBlast = 27,
    // ChargeFlameBurn = 28,
    // DoubleJumpUpgrade = 29,
    // BashBuff = 30,
    // UltraDefense = 31,
    // HealthEfficiency = 32,
    // Sense = 33,
    // UltraStomp = 34,
    // SparkFlame = 36,
    // QuickFlame = 37,
    // MapMarkers = 38,
    // EnergyEfficiency = 39,
    // HealthMarkers = 40,
    // EnergyMarkers = 41,
    // AbilityMarkers = 42,
    // Rekindle = 43,
    // Regroup = 44,
    // ChargeFlameEfficiency = 45,
    // UltraSoulFlame = 46,
    // SoulFlameEfficiency = 47,
    // CinderFlame = 48,
    // UltraSplitFlame = 49,
    // Dash = 50,
    Grenade = 51,
    // GrenadeUpgrade = 52,
    // ChargeDash = 53,
    // AirDash = 54,
    // GrenadeEfficiency = 55,
    // Bounce = 56,
    Grapple = 57,
    // SpiritSlash = 58,
    // HeavySpiritSlash = 59,
    // FireBurstSpell = 60,
    // FireWhirlSpell = 61,
    Flash = 62,
    // LockOnSpell = 63,
    // TimeWarpSpell = 64,
    // ShieldSpell = 65,
    // EnergyWallSpell = 66,
    // InvisibilitySpell = 67,
    // TrapSpell = 68,
    // WarpSpell = 69,
    // LightSpell = 70,
    // MindControlSpell = 71,
    // MirageSpell = 72,
    // StickyMineSpell = 73,
    Spear = 74,
    // LightSpearSpell = 75,
    // LifeAbsorbSpell = 76,
    Regenerate = 77,
    // ChargeShotSpell = 78,
    // SpiritShardsSpell = 79,
    // SpiritSentrySpell = 80,
    // PowerslideSpell = 81,
    // CounterstrikeSpell = 82,
    // EarthShatterSpell = 83,
    // JumpShotSpell = 84,
    // RoundupLeashSpell = 85,
    // BurrowSpell = 86,
    // PowerOfFriendshipSpell = 87,
    // LightningSpell = 88,
    // SpiritFlareSpell = 89,
    // EntanglingRootsSpell = 90,
    // MarkOfTheWildsSpell = 91,
    // HomingMissileSpell = 92,
    // SpiritCrescentSpell = 93,
    // MineSpell = 94,
    // Pinned = 95,
    // Leached = 96,
    Bow = 97,
    Hammer = 98,
    // Torch = 99,
    Sword = 100,
    Burrow = 101,
    Dash = 102,
    // Launch = 103,
    WaterDash = 104,
    // TeleportSpell = 105,
    Shuriken = 106,
    // Drill = 107,
    Seir = 108,
    BowCharge = 109,
    // Swordstaff = 110,
    // Chainsword = 111,
    Magnet = 112,
    // SwordCharge = 113, // TODO add an uberstate?
    // HammerCharge = 114, // TODO add an uberstate?
    Blaze = 115,
    Sentry = 116,
    // Regenerate = 117,
    Flap = 118,
    WeaponCharge = 119, // TODO what is this and why does it have an uberstate
    GladesAncestralLight = 120,
    MarshAncestralLight = 121,
}
impl Skill {
    /// Returns the [`UberIdentifier`] tracking whether the player has this `Skill`
    pub const fn uber_identifier(self) -> UberIdentifier {
        UberIdentifier::new(24, self as i32)
    }
    /// Returns the `Skill` corresponsing to the [`UberIdentifier`], if one exists
    pub const fn from_uber_identifier(uber_identifier: UberIdentifier) -> Option<Self> {
        const MIN: i32 = u8::MIN as i32;
        const MAX: i32 = u8::MAX as i32;
        match uber_identifier {
            UberIdentifier {
                group: 24,
                member: id @ MIN..=MAX,
            } => Self::from_repr(id as u8),
            _ => None,
        }
    }

    /// Returns the energy cost of using this `Skill` once
    pub const fn energy_cost(self) -> f32 {
        match self {
            Skill::Bow => 0.25,
            Skill::Shuriken => 0.5,
            Skill::Grenade | Skill::Flash | Skill::Regenerate | Skill::Blaze | Skill::Sentry => 1.0,
            Skill::Spear => 2.0,
            _ => 0.0,
        }
    }
    /// Returns the immediate damage dealt when using this `Skill` once
    ///
    /// Does not include any inflicted burn damage, use [`Skill::burn_damage`] for that
    ///
    /// `charge_grenade` determines whether to use charged or uncharged grenade damage if this is `Skill::Grenade`
    pub const fn damage(self, charge_grenade: bool) -> f32 {
        match self {
            Skill::Bow | Skill::Sword => 4.0,
            Skill::Launch => 5.0,
            Skill::Hammer | Skill::Flash => 12.0,
            Skill::Shuriken => 7.0,
            Skill::Grenade => {
                if charge_grenade {
                    8.0
                } else {
                    4.0
                }
            }
            Skill::Spear => 20.0,
            Skill::Blaze => 3.0,
            Skill::Sentry => 8.8,
            _ => 0.0,
        }
    }
    /// Returns the total damage inflicted by burn when using this `Skill` once
    pub const fn burn_damage(self) -> f32 {
        match self {
            Skill::Grenade => 9.0,
            Skill::Blaze => 10.8,
            _ => 0.0,
        }
    }
    /// Returns the [`Equipment`] corresponding to this `Skill`
    pub const fn equipment(self) -> Equipment {
        match self {
            Skill::Bash => Equipment::Bash, // TODO huh? what happens if you "unequip" bash?
            // Skill::ChargeFlame => todo!(),
            Skill::WallJump => todo!(),
            // Skill::Stomp => todo!(),
            Skill::DoubleJump => Equipment::DoubleJump,
            Skill::Launch => Equipment::Launch,
            // Skill::Magnet => todo!(),
            // Skill::UltraMagnet => todo!(),
            // Skill::Climb => todo!(),
            Skill::Glide => Equipment::Glide,
            Skill::SpiritFlame => todo!(),
            // Skill::RapidFlame => todo!(),
            // Skill::SplitFlameUpgrade => todo!(),
            // Skill::SoulEfficiency => todo!(),
            Skill::WaterBreath => Equipment::WaterBreath,
            // Skill::ChargeFlameBlast => todo!(),
            // Skill::ChargeFlameBurn => todo!(),
            // Skill::DoubleJumpUpgrade => todo!(),
            // Skill::BashBuff => todo!(),
            // Skill::UltraDefense => todo!(),
            // Skill::HealthEfficiency => todo!(),
            // Skill::Sense => todo!(),
            // Skill::UltraStomp => todo!(),
            // Skill::SparkFlame => todo!(),
            // Skill::QuickFlame => todo!(),
            // Skill::MapMarkers => todo!(),
            // Skill::EnergyEfficiency => todo!(),
            // Skill::HealthMarkers => todo!(),
            // Skill::EnergyMarkers => todo!(),
            // Skill::AbilityMarkers => todo!(),
            // Skill::Rekindle => todo!(),
            // Skill::Regroup => todo!(),
            // Skill::ChargeFlameEfficiency => todo!(),
            // Skill::UltraSoulFlame => todo!(),
            // Skill::SoulFlameEfficiency => todo!(),
            // Skill::CinderFlame => todo!(),
            // Skill::UltraSplitFlame => todo!(),
            // Skill::Dash => todo!(),
            Skill::Grenade => Equipment::Grenade,
            // Skill::GrenadeUpgrade => todo!(),
            // Skill::ChargeDash => todo!(),
            // Skill::AirDash => todo!(),
            // Skill::GrenadeEfficiency => todo!(),
            // Skill::Bounce => todo!(),
            Skill::Grapple => Equipment::Grapple,
            // Skill::SpiritSlash => todo!(),
            // Skill::HeavySpiritSlash => todo!(),
            // Skill::FireBurstSpell => todo!(),
            // Skill::FireWhirlSpell => todo!(),
            Skill::Flash => todo!(),
            // Skill::LockOnSpell => todo!(),
            // Skill::TimeWarpSpell => todo!(),
            // Skill::ShieldSpell => todo!(),
            // Skill::EnergyWallSpell => todo!(),
            // Skill::InvisibilitySpell => todo!(),
            // Skill::TrapSpell => todo!(),
            // Skill::WarpSpell => todo!(),
            // Skill::LightSpell => todo!(),
            // Skill::MindControlSpell => todo!(),
            // Skill::MirageSpell => todo!(),
            // Skill::StickyMineSpell => todo!(),
            Skill::Spear => Equipment::Spear,
            // Skill::LightSpearSpell => todo!(),
            // Skill::LifeAbsorbSpell => todo!(),
            Skill::Regenerate => Equipment::Regenerate,
            // Skill::ChargeShotSpell => todo!(),
            // Skill::SpiritShardsSpell => todo!(),
            // Skill::SpiritSentrySpell => todo!(),
            // Skill::PowerslideSpell => todo!(),
            // Skill::CounterstrikeSpell => todo!(),
            // Skill::EarthShatterSpell => todo!(),
            // Skill::JumpShotSpell => todo!(),
            // Skill::RoundupLeashSpell => todo!(),
            // Skill::BurrowSpell => todo!(),
            // Skill::PowerOfFriendshipSpell => todo!(),
            // Skill::LightningSpell => todo!(),
            // Skill::SpiritFlareSpell => todo!(),
            // Skill::EntanglingRootsSpell => todo!(),
            // Skill::MarkOfTheWildsSpell => todo!(),
            // Skill::HomingMissileSpell => todo!(),
            // Skill::SpiritCrescentSpell => todo!(),
            // Skill::MineSpell => todo!(),
            // Skill::Pinned => todo!(),
            // Skill::Leached => todo!(),
            Skill::Bow => Equipment::Bow,
            Skill::Hammer => Equipment::Hammer,
            // Skill::Torch => todo!(),
            Skill::Sword => Equipment::Sword,
            Skill::Burrow => Equipment::Burrow,
            Skill::Dash => Equipment::Dash,
            // Skill::Launch => todo!(),
            Skill::WaterDash => Equipment::WaterDash,
            // Skill::TeleportSpell => todo!(),
            Skill::Shuriken => Equipment::Shuriken,
            // Skill::Drill => todo!(),
            Skill::Seir => todo!(),
            Skill::BowCharge => todo!(),
            // Skill::Swordstaff => todo!(),
            // Skill::Chainsword => todo!(),
            Skill::Magnet => todo!(),
            // Skill::SwordCharge => todo!(),
            // Skill::HammerCharge => todo!(),
            Skill::Blaze => Equipment::Blaze,
            Skill::Sentry => Equipment::Sentry,
            // Skill::Regenerate => todo!(),
            Skill::Flap => Equipment::Flap,
            Skill::WeaponCharge => Equipment::WeaponCharge,
            Skill::GladesAncestralLight => todo!(),
            Skill::MarshAncestralLight => todo!(),
        }
    }
}
