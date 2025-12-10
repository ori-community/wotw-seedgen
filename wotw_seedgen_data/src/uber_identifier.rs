use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::{Icon, MapIcon, OpherIcon, Shard, Skill, Teleporter, WeaponUpgrade};

/// Identifier for an UberState
///
/// UberStates make up most of the save file format; every world state is associated with an UberState which may hold data, usually a single boolean or number.
/// The `UberIdentifier` is the unique identifier for a given UberState
#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct UberIdentifier {
    pub group: i32,
    pub member: i32,
}

impl UberIdentifier {
    /// Creates a new `UberIdentifier` from its parts
    ///
    /// See the [`uber_identifier`] module for constants on major `UberIdentifier`s that might make your code more readable
    pub const fn new(group: i32, member: i32) -> Self {
        Self { group, member }
    }

    // /// Returns `true` if this `UberIdentifier` corresponds to a "shop item bought" state
    pub const fn is_shop(self) -> bool {
        !matches!(self.shop_kind(), ShopKind::None)
    }

    /// Returns what kind of shop, if any, this `UberIdentifier` corresponds to
    pub const fn shop_kind(self) -> ShopKind {
        match self {
            Self {
                group: 1 | 2 | 15, ..
            } => ShopKind::Opherlike,
            Self { group: 17, .. } => ShopKind::Grom,
            Self {
                group: 48248,
                member: 18767 | 45538 | 3638 | 1590 | 1557 | 29604 | 48423 | 61146 | 4045,
            } => ShopKind::Map,
            _ => ShopKind::None,
        }
    }

    /// Returns `true` if this `UberIdentifier` corresponds to a door connection state
    pub const fn is_door(self) -> bool {
        self.group == 27
    }

    pub const SPIRIT_LIGHT: UberIdentifier = UberIdentifier::new(5, 0);
    pub const GORLEK_ORE: UberIdentifier = UberIdentifier::new(5, 1);
    pub const KEYSTONES: UberIdentifier = UberIdentifier::new(5, 2);
    pub const SHARD_SLOTS: UberIdentifier = UberIdentifier::new(5, 3); // TODO client needs to add this
    pub const CLEAN_WATER: UberIdentifier = UberIdentifier::new(6, 2000);
    pub const MAX_HEALTH: UberIdentifier = UberIdentifier::new(5, 10);
    pub const HEALTH: UberIdentifier = UberIdentifier::new(5, 11);
    pub const MAX_ENERGY: UberIdentifier = UberIdentifier::new(5, 12);
    pub const ENERGY: UberIdentifier = UberIdentifier::new(5, 13);
}

impl Display for UberIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}|{}", self.group, self.member)
    }
}

impl Debug for UberIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{self}")
    }
}

impl Serialize for UberIdentifier {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        (self.group, self.member).serialize(serializer)
    }
}

impl<'de> Deserialize<'de> for UberIdentifier {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        <(i32, i32)>::deserialize(deserializer).map(|(group, member)| Self { group, member })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShopKind {
    /// Not a shop
    None,
    /// A shop with multiple items that cost Spirit Light.
    ///
    /// This includes the Opher, Twillen and Glades Lupo shops.
    Opherlike,
    /// A purchasable map from Lupo
    Map,
    /// Grom's Gorlek Ore shop
    Grom,
}

/// A helper type to represent common [`UberIdentifier`]s
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonUberIdentifier {
    SpiritLight,
    GorlekOre,
    Keystones,
    ShardSlots,
    CleanWater,
    MaxHealth,
    Health,
    MaxEnergy,
    Energy,
    Skill(Skill),
    Shard(Shard),
    Teleporter(Teleporter),
    WeaponUpgrade(WeaponUpgrade),
}

impl CommonUberIdentifier {
    pub const fn map_icon(self) -> MapIcon {
        match self {
            Self::SpiritLight => MapIcon::SpiritLight,
            Self::GorlekOre => MapIcon::GorlekOre,
            Self::Keystones => MapIcon::Keystone,
            Self::ShardSlots => MapIcon::ShardSlot,
            Self::CleanWater => MapIcon::CleanWater,
            Self::MaxHealth | Self::Health => MapIcon::HealthFragment,
            Self::MaxEnergy | Self::Energy => MapIcon::EnergyFragment,
            Self::WeaponUpgrade(_) => MapIcon::BonusItem, // TODO is this good?
            Self::Shard(_) => MapIcon::Shard,
            Self::Teleporter(_) => MapIcon::SpiritWell,
            Self::Skill(_) => MapIcon::Skill,
        }
    }

    pub const fn shop_price(self) -> f32 {
        match self {
            Self::SpiritLight => 200.,
            Self::GorlekOre | Self::Keystones => 100.,
            Self::ShardSlots => 250.,
            Self::CleanWater => 500.,
            Self::MaxHealth | Self::Health => 200.,
            Self::MaxEnergy | Self::Energy => 150.,
            Self::Skill(skill) => match skill {
                Skill::WaterBreath | Skill::Regenerate | Skill::Seir => 200.,
                Skill::GladesAncestralLight | Skill::MarshAncestralLight => 300.,
                Skill::Blaze => 420.,
                Skill::Launch => 800.,
                _ => 500.,
            },
            Self::Shard(_) | Self::Teleporter(_) => 250.,
            Self::WeaponUpgrade(_) => 200.,
        }
    }

    pub const fn icon(self) -> Option<Icon> {
        let icon = match self {
            Self::SpiritLight => Icon::File(Cow::Borrowed("assets/icons/game/experience.png")),
            Self::GorlekOre => Icon::File(Cow::Borrowed("assets/icons/game/gorlekore.png")),
            Self::Keystones => Icon::File(Cow::Borrowed("assets/icons/game/keystone.png")),
            Self::ShardSlots => Icon::File(Cow::Borrowed("assets/icons/game/shardslot.png")),
            Self::CleanWater => Icon::File(Cow::Borrowed("assets/icons/game/water.png")),
            Self::MaxHealth | Self::Health => {
                Icon::File(Cow::Borrowed("assets/icons/game/healthfragment.png"))
            }
            Self::MaxEnergy | Self::Energy => {
                Icon::File(Cow::Borrowed("assets/icons/game/energyfragment.png"))
            }
            Self::WeaponUpgrade(weapon_upgrade) => match weapon_upgrade {
                WeaponUpgrade::ExplodingSpear => Icon::Opher(OpherIcon::ExplodingSpear),
                WeaponUpgrade::HammerShockwave => Icon::Opher(OpherIcon::HammerShockwave),
                WeaponUpgrade::StaticShuriken => Icon::Opher(OpherIcon::StaticShuriken),
                WeaponUpgrade::ChargeBlaze => Icon::Opher(OpherIcon::ChargeBlaze),
                WeaponUpgrade::RapidSentry => Icon::Opher(OpherIcon::RapidSentry),
            },
            Self::Shard(shard) => Icon::Shard(shard),
            Self::Teleporter(_) => Icon::File(Cow::Borrowed("assets/icons/game/teleporter.png")),
            Self::Skill(skill) => match skill {
                // TODO does the equipment not work for these?
                Skill::WaterBreath => Icon::Opher(OpherIcon::WaterBreath),
                Skill::Spear => Icon::Opher(OpherIcon::Spear),
                Skill::Hammer => Icon::Opher(OpherIcon::Hammer),
                Skill::Shuriken => Icon::Opher(OpherIcon::Shuriken),
                Skill::Blaze => Icon::Opher(OpherIcon::Blaze),
                Skill::Sentry => Icon::Opher(OpherIcon::Sentry),
                Skill::GladesAncestralLight => {
                    Icon::File(Cow::Borrowed("assets/icons/game/ancestrallight1.png"))
                }
                Skill::MarshAncestralLight => {
                    Icon::File(Cow::Borrowed("assets/icons/game/ancestrallight2.png"))
                }
                skill => match skill.equipment() {
                    None => return None,
                    Some(equipment) => Icon::Equipment(equipment),
                },
            },
        };

        Some(icon)
    }

    /// Returns the [`UberIdentifier`] corresponding this `CommonUberIdentifier`
    pub const fn uber_identifier(self) -> UberIdentifier {
        match self {
            Self::SpiritLight => UberIdentifier::SPIRIT_LIGHT,
            Self::GorlekOre => UberIdentifier::GORLEK_ORE,
            Self::Keystones => UberIdentifier::KEYSTONES,
            Self::ShardSlots => UberIdentifier::SHARD_SLOTS,
            Self::CleanWater => UberIdentifier::CLEAN_WATER,
            Self::MaxHealth => UberIdentifier::MAX_HEALTH,
            Self::Health => UberIdentifier::HEALTH,
            Self::MaxEnergy => UberIdentifier::MAX_ENERGY,
            Self::Energy => UberIdentifier::ENERGY,
            Self::Skill(skill) => skill.uber_identifier(),
            Self::Shard(shard) => shard.uber_identifier(),
            Self::Teleporter(teleporter) => teleporter.uber_identifier(),
            Self::WeaponUpgrade(weapon_upgrade) => weapon_upgrade.uber_identifier(),
        }
    }

    /// Returns the `CommonUberIdentifier` corresponsing to the [`UberIdentifier`], if one exists
    pub const fn from_uber_identifier(uber_identifier: UberIdentifier) -> Option<Self> {
        match uber_identifier {
            UberIdentifier::SPIRIT_LIGHT => Some(Self::SpiritLight),
            UberIdentifier::GORLEK_ORE => Some(Self::GorlekOre),
            UberIdentifier::KEYSTONES => Some(Self::Keystones),
            UberIdentifier::SHARD_SLOTS => Some(Self::ShardSlots),
            UberIdentifier::CLEAN_WATER => Some(Self::CleanWater),
            UberIdentifier::MAX_HEALTH => Some(Self::MaxHealth),
            UberIdentifier::HEALTH => Some(Self::Health),
            UberIdentifier::MAX_ENERGY => Some(Self::MaxEnergy),
            UberIdentifier::ENERGY => Some(Self::Energy),
            uber_identifier => {
                if let Some(skill) = Skill::from_uber_identifier(uber_identifier) {
                    Some(Self::Skill(skill))
                } else if let Some(shard) = Shard::from_uber_identifier(uber_identifier) {
                    Some(Self::Shard(shard))
                } else if let Some(teleporter) = Teleporter::from_uber_identifier(uber_identifier) {
                    Some(Self::Teleporter(teleporter))
                } else if let Some(weapon_upgrade) =
                    WeaponUpgrade::from_uber_identifier(uber_identifier)
                {
                    Some(Self::WeaponUpgrade(weapon_upgrade))
                } else {
                    None
                }
            }
        }
    }
}
