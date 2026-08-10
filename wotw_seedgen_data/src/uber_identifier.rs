use std::{
    borrow::Cow,
    fmt::{self, Debug, Display},
    num::ParseIntError,
    str::FromStr,
};

use serde::{Deserialize, Deserializer, Serialize, Serializer};
use thiserror::Error;
use utoipa::{
    openapi::{schema::ArrayItems, ArrayBuilder, RefOr, Schema},
    PartialSchema, ToSchema,
};

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

const PLAYER_GROUP: i32 = 5;
const RANDO_STATE_GROUP: i32 = 6;
const RANDO_CONFIG_GROUP: i32 = 7;
const MULTIWORLD_GROUP: i32 = 12;
const MAP_SEGMENTS_GROUP: i32 = 22;
const ITEM_TRACKER_GROUP: i32 = 23;
const SKILLS_GROUP: i32 = 24;
const SHARDS_GROUP: i32 = 25;
const ENTRANCES_GROUP: i32 = 27;
const KNOWN_ENTRANCE_CONNECTIONS_GROUP: i32 = 28;
const SETTINGS_GROUP: i32 = 29;

impl UberIdentifier {
    /// Creates a new `UberIdentifier` from its parts
    ///
    /// See the [`uber_identifier`] module for constants on major `UberIdentifier`s that might make your code more readable
    pub const fn new(group: i32, member: i32) -> Self {
        Self { group, member }
    }

    pub const fn player(member: i32) -> Self {
        Self::new(PLAYER_GROUP, member)
    }

    pub const fn rando_state(member: i32) -> Self {
        Self::new(RANDO_STATE_GROUP, member)
    }

    pub const fn rando_config(member: i32) -> Self {
        Self::new(RANDO_CONFIG_GROUP, member)
    }

    pub const fn item_tracker(member: i32) -> Self {
        Self::new(ITEM_TRACKER_GROUP, member)
    }

    pub const fn multiworld(member: i32) -> Self {
        Self::new(MULTIWORLD_GROUP, member)
    }

    pub const fn map_segment(member: i32) -> Self {
        Self::new(MAP_SEGMENTS_GROUP, member)
    }

    pub const fn skills(member: i32) -> Self {
        Self::new(SKILLS_GROUP, member)
    }

    pub const fn shards(member: i32) -> Self {
        Self::new(SHARDS_GROUP, member)
    }

    pub const fn entrances(member: i32) -> Self {
        Self::new(ENTRANCES_GROUP, member)
    }

    pub const fn known_entrance_connections(member: i32) -> Self {
        Self::new(KNOWN_ENTRANCE_CONNECTIONS_GROUP, member)
    }

    pub const fn settings(member: i32) -> Self {
        Self::new(SETTINGS_GROUP, member)
    }

    pub const fn as_skills(self) -> Option<i32> {
        match self {
            UberIdentifier {
                group: SKILLS_GROUP,
                member,
            } => Some(member),
            _ => None,
        }
    }

    pub const fn as_shards(self) -> Option<i32> {
        match self {
            UberIdentifier {
                group: SHARDS_GROUP,
                member,
            } => Some(member),
            _ => None,
        }
    }

    pub const fn as_multiworld(self) -> Option<i32> {
        match self {
            UberIdentifier {
                group: MULTIWORLD_GROUP,
                member,
            } => Some(member),
            _ => None,
        }
    }

    /// Returns `true` if this `UberIdentifier` corresponds to a "shop item bought" state
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
            Self { group: 20, .. } => ShopKind::Tuley,
            Self {
                group: 48248,
                member: 18767 | 45538 | 3638 | 1590 | 1557 | 29604 | 48423 | 61146 | 4045,
            } => ShopKind::Map,
            _ => ShopKind::None,
        }
    }

    /// Returns `true` if this `UberIdentifier` corresponds to a spirit trial state
    pub const fn is_spirit_trial(self) -> bool {
        matches!(
            self,
            Self {
                group: 44964,
                member: 45951 | 25545 | 11512 | 54686 | 22703 | 23661 | 28552 | 30767
            }
        )
    }

    /// Returns `true` if this `UberIdentifier` corresponds to an entrance connection state
    pub const fn is_entrance(self) -> bool {
        self.group == 27
    }

    pub const SPIRIT_LIGHT: UberIdentifier = UberIdentifier::player(0);
    pub const GORLEK_ORE: UberIdentifier = UberIdentifier::player(1);
    pub const KEYSTONES: UberIdentifier = UberIdentifier::player(2);
    pub const SHARD_SLOTS: UberIdentifier = UberIdentifier::player(3);
    pub const SHRIEK_BARRIER: UberIdentifier = UberIdentifier::rando_state(0);
    pub const CLEAN_WATER: UberIdentifier = UberIdentifier::rando_state(2000);
    pub const BASE_MAX_HEALTH: UberIdentifier = UberIdentifier::player(10);
    pub const MAX_HEALTH: UberIdentifier = UberIdentifier::player(11);
    pub const HEALTH: UberIdentifier = UberIdentifier::player(12);
    pub const BASE_MAX_ENERGY: UberIdentifier = UberIdentifier::player(20);
    pub const MAX_ENERGY: UberIdentifier = UberIdentifier::player(21);
    pub const ENERGY: UberIdentifier = UberIdentifier::player(22);
    pub const RANDOM_SPIRIT_LIGHT: UberIdentifier = UberIdentifier::settings(0);
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

impl FromStr for UberIdentifier {
    type Err = ParseUberIdentifierError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        fn parse_part(
            part: &str,
            error: fn(String, ParseIntError) -> ParseUberIdentifierError,
        ) -> Result<i32, ParseUberIdentifierError> {
            let part = part.trim();
            part.parse().map_err(|err| error(part.to_string(), err))
        }

        let (group, member) = s.split_once('|').ok_or(ParseUberIdentifierError::Format)?;

        Ok(Self {
            group: parse_part(group, ParseUberIdentifierError::Group)?,
            member: parse_part(member, ParseUberIdentifierError::Member)?,
        })
    }
}

#[derive(Debug, Error)]
pub enum ParseUberIdentifierError {
    #[error("invalid format")]
    Format,
    #[error("invalid group {0}: {1}")]
    Group(String, ParseIntError),
    #[error("invalid member {0}: {1}")]
    Member(String, ParseIntError),
}

impl PartialSchema for UberIdentifier {
    fn schema() -> RefOr<Schema> {
        ArrayBuilder::new()
            .items(ArrayItems::False)
            .prefix_items([utoipa::schema!(i32), utoipa::schema!(i32)])
            .into()
    }
}

impl ToSchema for UberIdentifier {}

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
    /// Tuley's shop where everything is free
    Tuley,
}

/// A helper type to represent common [`UberIdentifier`]s
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CommonUberIdentifier {
    SpiritLight,
    GorlekOre,
    Keystones,
    ShardSlots,
    CleanWater,
    BaseMaxHealth,
    BaseMaxEnergy,
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
            Self::BaseMaxHealth => MapIcon::HealthFragment,
            Self::BaseMaxEnergy => MapIcon::EnergyFragment,
            Self::WeaponUpgrade(_) => MapIcon::BonusItem, // TODO is this good?
            Self::Shard(_) => MapIcon::Shard,
            Self::Teleporter(_) => MapIcon::SavePedestalInactive,
            Self::Skill(_) => MapIcon::Skill,
        }
    }

    pub const fn shop_price(self) -> i32 {
        match self {
            Self::SpiritLight => 200,
            Self::GorlekOre | Self::Keystones => 100,
            Self::ShardSlots => 250,
            Self::CleanWater => 500,
            Self::BaseMaxHealth => 200,
            Self::BaseMaxEnergy => 150,
            Self::Skill(skill) => match skill {
                Skill::WaterBreath | Skill::Regenerate | Skill::Seir => 200,
                Skill::GladesAncestralLight | Skill::MarshAncestralLight => 300,
                Skill::Blaze => 420,
                Skill::Launch => 800,
                _ => 500,
            },
            Self::Shard(_) | Self::Teleporter(_) => 250,
            Self::WeaponUpgrade(_) => 200,
        }
    }

    pub const fn icon(self) -> Option<Icon> {
        let icon = match self {
            Self::SpiritLight => Icon::File(Cow::Borrowed("icons/game/experience.png")),
            Self::GorlekOre => Icon::File(Cow::Borrowed("icons/game/gorlekore.png")),
            Self::Keystones => Icon::File(Cow::Borrowed("icons/game/keystone.png")),
            Self::ShardSlots => Icon::File(Cow::Borrowed("icons/game/shardslot.png")),
            Self::CleanWater => Icon::File(Cow::Borrowed("icons/game/water.png")),
            Self::BaseMaxHealth => Icon::File(Cow::Borrowed("icons/game/healthfragment.png")),
            Self::BaseMaxEnergy => Icon::File(Cow::Borrowed("icons/game/energyfragment.png")),
            Self::WeaponUpgrade(weapon_upgrade) => match weapon_upgrade {
                WeaponUpgrade::ExplodingSpear => Icon::Opher(OpherIcon::ExplodingSpear),
                WeaponUpgrade::HammerShockwave => Icon::Opher(OpherIcon::HammerShockwave),
                WeaponUpgrade::StaticShuriken => Icon::Opher(OpherIcon::StaticShuriken),
                WeaponUpgrade::ChargeBlaze => Icon::Opher(OpherIcon::ChargeBlaze),
                WeaponUpgrade::RapidSentry => Icon::Opher(OpherIcon::RapidSentry),
            },
            Self::Shard(shard) => Icon::Shard(shard),
            Self::Teleporter(_) => Icon::File(Cow::Borrowed("icons/game/teleporter.png")),
            Self::Skill(skill) => match skill {
                // TODO does the equipment not work for these?
                Skill::WaterBreath => Icon::Opher(OpherIcon::WaterBreath),
                Skill::Spear => Icon::Opher(OpherIcon::Spear),
                Skill::Hammer => Icon::Opher(OpherIcon::Hammer),
                Skill::Shuriken => Icon::Opher(OpherIcon::Shuriken),
                Skill::Blaze => Icon::Opher(OpherIcon::Blaze),
                Skill::Sentry => Icon::Opher(OpherIcon::Sentry),
                Skill::GladesAncestralLight => {
                    Icon::File(Cow::Borrowed("icons/game/ancestrallight1.png"))
                }
                Skill::MarshAncestralLight => {
                    Icon::File(Cow::Borrowed("icons/game/ancestrallight2.png"))
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
            Self::BaseMaxHealth => UberIdentifier::BASE_MAX_HEALTH,
            Self::BaseMaxEnergy => UberIdentifier::BASE_MAX_ENERGY,
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
            UberIdentifier::BASE_MAX_HEALTH => Some(Self::BaseMaxHealth),
            UberIdentifier::BASE_MAX_ENERGY => Some(Self::BaseMaxEnergy),
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
