use crate::{Equipment, UberIdentifier};
use serde_repr::{Deserialize_repr, Serialize_repr};
use strum::{Display, FromRepr, VariantArray};
use wotw_seedgen_derive::FromStr;

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
    FromStr,
    FromRepr,
    VariantArray,
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
    // Magnet = 112,
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
    pub const BASH_ID: UberIdentifier = Self::Bash.uber_identifier();
    // pub const CHARGE_FLAME_ID: UberIdentifier = Self::ChargeFlame.uber_identifier();
    pub const WALL_JUMP_ID: UberIdentifier = Self::WallJump.uber_identifier();
    // pub const STOMP_ID: UberIdentifier = Self::Stomp.uber_identifier();
    pub const DOUBLE_JUMP_ID: UberIdentifier = Self::DoubleJump.uber_identifier();
    pub const LAUNCH_ID: UberIdentifier = Self::Launch.uber_identifier();
    // pub const MAGNET_ID: UberIdentifier = Self::Magnet.uber_identifier();
    // pub const ULTRA_MAGNET_ID: UberIdentifier = Self::UltraMagnet.uber_identifier();
    // pub const CLIMB_ID: UberIdentifier = Self::Climb.uber_identifier();
    pub const GLIDE_ID: UberIdentifier = Self::Glide.uber_identifier();
    pub const SPIRIT_FLAME_ID: UberIdentifier = Self::SpiritFlame.uber_identifier();
    // pub const RAPID_FLAME_ID: UberIdentifier = Self::RapidFlame.uber_identifier();
    // pub const SPLIT_FLAME_UPGRADE_ID: UberIdentifier = Self::SplitFlameUpgrade.uber_identifier();
    // pub const SOUL_EFFICIENCY_ID: UberIdentifier = Self::SoulEfficiency.uber_identifier();
    pub const WATER_BREATH_ID: UberIdentifier = Self::WaterBreath.uber_identifier();
    // pub const CHARGE_FLAME_BLAST_ID: UberIdentifier = Self::ChargeFlameBlast.uber_identifier();
    // pub const CHARGE_FLAME_BURN_ID: UberIdentifier = Self::ChargeFlameBurn.uber_identifier();
    // pub const DOUBLE_JUMP_UPGRADE_ID: UberIdentifier = Self::DoubleJumpUpgrade.uber_identifier();
    // pub const BASH_BUFF_ID: UberIdentifier = Self::BashBuff.uber_identifier();
    // pub const ULTRA_DEFENSE_ID: UberIdentifier = Self::UltraDefense.uber_identifier();
    // pub const HEALTH_EFFICIENCY_ID: UberIdentifier = Self::HealthEfficiency.uber_identifier();
    // pub const SENSE_ID: UberIdentifier = Self::Sense.uber_identifier();
    // pub const ULTRA_STOMP_ID: UberIdentifier = Self::UltraStomp.uber_identifier();
    // pub const SPARK_FLAME_ID: UberIdentifier = Self::SparkFlame.uber_identifier();
    // pub const QUICK_FLAME_ID: UberIdentifier = Self::QuickFlame.uber_identifier();
    // pub const MAP_MARKERS_ID: UberIdentifier = Self::MapMarkers.uber_identifier();
    // pub const ENERGY_EFFICIENCY_ID: UberIdentifier = Self::EnergyEfficiency.uber_identifier();
    // pub const HEALTH_MARKERS_ID: UberIdentifier = Self::HealthMarkers.uber_identifier();
    // pub const ENERGY_MARKERS_ID: UberIdentifier = Self::EnergyMarkers.uber_identifier();
    // pub const ABILITY_MARKERS_ID: UberIdentifier = Self::AbilityMarkers.uber_identifier();
    // pub const REKINDLE_ID: UberIdentifier = Self::Rekindle.uber_identifier();
    // pub const REGROUP_ID: UberIdentifier = Self::Regroup.uber_identifier();
    // pub const CHARGE_FLAME_EFFICIENCY_ID: UberIdentifier = Self::ChargeFlameEfficiency.uber_identifier();
    // pub const ULTRA_SOUL_FLAME_ID: UberIdentifier = Self::UltraSoulFlame.uber_identifier();
    // pub const SOUL_FLAME_EFFICIENCY_ID: UberIdentifier = Self::SoulFlameEfficiency.uber_identifier();
    // pub const CINDER_FLAME_ID: UberIdentifier = Self::CinderFlame.uber_identifier();
    // pub const ULTRA_SPLIT_FLAME_ID: UberIdentifier = Self::UltraSplitFlame.uber_identifier();
    // pub const DASH_ID: UberIdentifier = Self::Dash.uber_identifier();
    pub const GRENADE_ID: UberIdentifier = Self::Grenade.uber_identifier();
    // pub const GRENADE_UPGRADE_ID: UberIdentifier = Self::GrenadeUpgrade.uber_identifier();
    // pub const CHARGE_DASH_ID: UberIdentifier = Self::ChargeDash.uber_identifier();
    // pub const AIR_DASH_ID: UberIdentifier = Self::AirDash.uber_identifier();
    // pub const GRENADE_EFFICIENCY_ID: UberIdentifier = Self::GrenadeEfficiency.uber_identifier();
    // pub const BOUNCE_ID: UberIdentifier = Self::Bounce.uber_identifier();
    pub const GRAPPLE_ID: UberIdentifier = Self::Grapple.uber_identifier();
    // pub const SPIRIT_SLASH_ID: UberIdentifier = Self::SpiritSlash.uber_identifier();
    // pub const HEAVY_SPIRIT_SLASH_ID: UberIdentifier = Self::HeavySpiritSlash.uber_identifier();
    // pub const FIRE_BURST_SPELL_ID: UberIdentifier = Self::FireBurstSpell.uber_identifier();
    // pub const FIRE_WHIRL_SPELL_ID: UberIdentifier = Self::FireWhirlSpell.uber_identifier();
    pub const FLASH_ID: UberIdentifier = Self::Flash.uber_identifier();
    // pub const LOCK_ON_SPELL_ID: UberIdentifier = Self::LockOnSpell.uber_identifier();
    // pub const TIME_WARP_SPELL_ID: UberIdentifier = Self::TimeWarpSpell.uber_identifier();
    // pub const SHIELD_SPELL_ID: UberIdentifier = Self::ShieldSpell.uber_identifier();
    // pub const ENERGY_WALL_SPELL_ID: UberIdentifier = Self::EnergyWallSpell.uber_identifier();
    // pub const INVISIBILITY_SPELL_ID: UberIdentifier = Self::InvisibilitySpell.uber_identifier();
    // pub const TRAP_SPELL_ID: UberIdentifier = Self::TrapSpell.uber_identifier();
    // pub const WARP_SPELL_ID: UberIdentifier = Self::WarpSpell.uber_identifier();
    // pub const LIGHT_SPELL_ID: UberIdentifier = Self::LightSpell.uber_identifier();
    // pub const MIND_CONTROL_SPELL_ID: UberIdentifier = Self::MindControlSpell.uber_identifier();
    // pub const MIRAGE_SPELL_ID: UberIdentifier = Self::MirageSpell.uber_identifier();
    // pub const STICKY_MINE_SPELL_ID: UberIdentifier = Self::StickyMineSpell.uber_identifier();
    pub const SPEAR_ID: UberIdentifier = Self::Spear.uber_identifier();
    // pub const LIGHT_SPEAR_SPELL_ID: UberIdentifier = Self::LightSpearSpell.uber_identifier();
    // pub const LIFE_ABSORB_SPELL_ID: UberIdentifier = Self::LifeAbsorbSpell.uber_identifier();
    pub const REGENERATE_ID: UberIdentifier = Self::Regenerate.uber_identifier();
    // pub const CHARGE_SHOT_SPELL_ID: UberIdentifier = Self::ChargeShotSpell.uber_identifier();
    // pub const SPIRIT_SHARDS_SPELL_ID: UberIdentifier = Self::SpiritShardsSpell.uber_identifier();
    // pub const SPIRIT_SENTRY_SPELL_ID: UberIdentifier = Self::SpiritSentrySpell.uber_identifier();
    // pub const POWERSLIDE_SPELL_ID: UberIdentifier = Self::PowerslideSpell.uber_identifier();
    // pub const COUNTERSTRIKE_SPELL_ID: UberIdentifier = Self::CounterstrikeSpell.uber_identifier();
    // pub const EARTH_SHATTER_SPELL_ID: UberIdentifier = Self::EarthShatterSpell.uber_identifier();
    // pub const JUMP_SHOT_SPELL_ID: UberIdentifier = Self::JumpShotSpell.uber_identifier();
    // pub const ROUNDUP_LEASH_SPELL_ID: UberIdentifier = Self::RoundupLeashSpell.uber_identifier();
    // pub const BURROW_SPELL_ID: UberIdentifier = Self::BurrowSpell.uber_identifier();
    // pub const POWER_OF_FRIENDSHIP_SPELL_ID: UberIdentifier = Self::PowerOfFriendshipSpell.uber_identifier();
    // pub const LIGHTNING_SPELL_ID: UberIdentifier = Self::LightningSpell.uber_identifier();
    // pub const SPIRIT_FLARE_SPELL_ID: UberIdentifier = Self::SpiritFlareSpell.uber_identifier();
    // pub const ENTANGLING_ROOTS_SPELL_ID: UberIdentifier = Self::EntanglingRootsSpell.uber_identifier();
    // pub const MARK_OF_THE_WILDS_SPELL_ID: UberIdentifier = Self::MarkOfTheWildsSpell.uber_identifier();
    // pub const HOMING_MISSILE_SPELL_ID: UberIdentifier = Self::HomingMissileSpell.uber_identifier();
    // pub const SPIRIT_CRESCENT_SPELL_ID: UberIdentifier = Self::SpiritCrescentSpell.uber_identifier();
    // pub const MINE_SPELL_ID: UberIdentifier = Self::MineSpell.uber_identifier();
    // pub const PINNED_ID: UberIdentifier = Self::Pinned.uber_identifier();
    // pub const LEACHED_ID: UberIdentifier = Self::Leached.uber_identifier();
    pub const BOW_ID: UberIdentifier = Self::Bow.uber_identifier();
    pub const HAMMER_ID: UberIdentifier = Self::Hammer.uber_identifier();
    // pub const TORCH_ID: UberIdentifier = Self::Torch.uber_identifier();
    pub const SWORD_ID: UberIdentifier = Self::Sword.uber_identifier();
    pub const BURROW_ID: UberIdentifier = Self::Burrow.uber_identifier();
    pub const DASH_ID: UberIdentifier = Self::Dash.uber_identifier();
    // pub const LAUNCH_ID: UberIdentifier = Self::Launch.uber_identifier();
    pub const WATER_DASH_ID: UberIdentifier = Self::WaterDash.uber_identifier();
    // pub const TELEPORT_SPELL_ID: UberIdentifier = Self::TeleportSpell.uber_identifier();
    pub const SHURIKEN_ID: UberIdentifier = Self::Shuriken.uber_identifier();
    // pub const DRILL_ID: UberIdentifier = Self::Drill.uber_identifier();
    pub const SEIR_ID: UberIdentifier = Self::Seir.uber_identifier();
    pub const BOW_CHARGE_ID: UberIdentifier = Self::BowCharge.uber_identifier();
    // pub const SWORDSTAFF_ID: UberIdentifier = Self::Swordstaff.uber_identifier();
    // pub const CHAINSWORD_ID: UberIdentifier = Self::Chainsword.uber_identifier();
    // pub const MAGNET_ID: UberIdentifier = Self::Magnet.uber_identifier();
    // pub const SWORD_CHARGE_ID: UberIdentifier = Self::SwordCharge.uber_identifier();
    // pub const HAMMER_CHARGE_ID: UberIdentifier = Self::HammerCharge.uber_identifier();
    pub const BLAZE_ID: UberIdentifier = Self::Blaze.uber_identifier();
    pub const SENTRY_ID: UberIdentifier = Self::Sentry.uber_identifier();
    // pub const REGENERATE_ID: UberIdentifier = Self::Regenerate.uber_identifier();
    pub const FLAP_ID: UberIdentifier = Self::Flap.uber_identifier();
    pub const WEAPON_CHARGE_ID: UberIdentifier = Self::WeaponCharge.uber_identifier();
    pub const GLADES_ANCESTRAL_LIGHT_ID: UberIdentifier =
        Self::GladesAncestralLight.uber_identifier();
    pub const MARSH_ANCESTRAL_LIGHT_ID: UberIdentifier =
        Self::MarshAncestralLight.uber_identifier();

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
    // TODO unmatched equipments: Torch, Shot, Wave, Whirl, SentryDeprecated, DoubleJump, Launch
    /// Returns the [`Equipment`] corresponding to this `Skill`
    pub const fn equipment(self) -> Option<Equipment> {
        match self {
            Skill::Bash => Some(Equipment::Bash),
            // Skill::ChargeFlame => todo!(),
            Skill::WallJump => None,
            // Skill::Stomp => todo!(),
            Skill::DoubleJump => Some(Equipment::Bounce),
            Skill::Launch => Some(Equipment::ChargeJump),
            // Skill::Magnet => todo!(),
            // Skill::UltraMagnet => todo!(),
            // Skill::Climb => Some(Equipment::Climb),
            Skill::Glide => Some(Equipment::Glide),
            Skill::SpiritFlame => None,
            // Skill::RapidFlame => todo!(),
            // Skill::SplitFlameUpgrade => todo!(),
            // Skill::SoulEfficiency => todo!(),
            Skill::WaterBreath => Some(Equipment::WaterBreath),
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
            Skill::Grenade => Some(Equipment::Grenade),
            // Skill::GrenadeUpgrade => todo!(),
            // Skill::ChargeDash => todo!(),
            // Skill::AirDash => todo!(),
            // Skill::GrenadeEfficiency => todo!(),
            // Skill::Bounce => todo!(),
            Skill::Grapple => Some(Equipment::Grapple),
            // Skill::SpiritSlash => todo!(),
            // Skill::HeavySpiritSlash => todo!(),
            // Skill::FireBurstSpell => todo!(),
            // Skill::FireWhirlSpell => todo!(),
            Skill::Flash => Some(Equipment::Flash),
            // Skill::LockOnSpell => Some(Equipment::LockOn),
            // Skill::TimeWarpSpell => todo!(),
            // Skill::ShieldSpell => Some(Equipment::Shield),
            // Skill::EnergyWallSpell => todo!(),
            // Skill::InvisibilitySpell => Some(Equipment::Invisibility),
            // Skill::TrapSpell => todo!(),
            // Skill::WarpSpell => todo!(),
            // Skill::LightSpell => todo!(),
            // Skill::MindControlSpell => todo!(),
            // Skill::MirageSpell => todo!(),
            // Skill::StickyMineSpell => todo!(),
            Skill::Spear => Some(Equipment::Spear),
            // Skill::LightSpearSpell => todo!(),
            // Skill::LifeAbsorbSpell => Some(Equipment::LifeAbsorb),
            Skill::Regenerate => Some(Equipment::Regenerate),
            // Skill::ChargeShotSpell => todo!(),
            // Skill::SpiritShardsSpell => Some(Equipment::Shards),
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
            // Skill::HomingMissileSpell => Some(Equipment::HomingMissiles),
            // Skill::SpiritCrescentSpell => todo!(),
            // Skill::MineSpell => todo!(),
            // Skill::Pinned => todo!(),
            // Skill::Leached => todo!(),
            Skill::Bow => Some(Equipment::Bow),
            Skill::Hammer => Some(Equipment::Hammer),
            // Skill::Torch => todo!(),
            Skill::Sword => Some(Equipment::Sword),
            Skill::Burrow => Some(Equipment::Burrow),
            Skill::Dash => Some(Equipment::Dash),
            // Skill::Launch => todo!(),
            Skill::WaterDash => Some(Equipment::WaterDash),
            // Skill::TeleportSpell => Some(Equipment::Teleport),
            Skill::Shuriken => Some(Equipment::Shuriken),
            // Skill::Drill => Some(Equipment::Drill),
            Skill::Seir => Some(Equipment::Sein),
            Skill::BowCharge => None,
            // Skill::Swordstaff => Some(Equipment::Swordstaff),
            // Skill::Chainsword => Some(Equipment::Chainsword),
            // Skill::Magnet => None,
            // Skill::SwordCharge => todo!(),
            // Skill::HammerCharge => todo!(),
            Skill::Blaze => Some(Equipment::Blaze),
            Skill::Sentry => Some(Equipment::Sentry),
            // Skill::Regenerate => todo!(),
            Skill::Flap => Some(Equipment::Flap),
            Skill::WeaponCharge => Some(Equipment::WeaponCharge),
            Skill::GladesAncestralLight => Some(Equipment::DamageUpgradeA),
            Skill::MarshAncestralLight => Some(Equipment::DamageUpgradeB),
        }
    }
}
