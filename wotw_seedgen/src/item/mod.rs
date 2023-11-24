mod bonus_item;
mod bonus_upgrade;
mod command;
mod message;
mod resource;
mod shard;
mod shop_command;
mod skill;
mod sysmessage;
mod teleporter;
mod uber_state;
mod wheel_command;

use std::fmt;
use std::str::FromStr;

use rustc_hash::FxHashMap;
use serde::{Deserialize, Serialize};
use wotw_seedgen_derive::VVariant;

use crate::header::{parser, CodeDisplay};
use crate::header::{vdisplay, VResolve, VString};
use crate::settings::{logical_difficulty, Difficulty};
use crate::uber_state::UberIdentifier;
use crate::util::{Icon, MapIcon, Zone};

pub use self::{
    bonus_item::BonusItem,
    bonus_upgrade::BonusUpgrade,
    command::{Command, EquipSlot, ToggleCommand, VCommand},
    message::{Message, VMessage},
    resource::Resource,
    shard::Shard,
    shop_command::{ShopCommand, VShopCommand},
    skill::Skill,
    sysmessage::SysMessage,
    teleporter::Teleporter,
    uber_state::{
        UberStateItem, UberStateOperator, UberStateRange, UberStateRangeBoundary, UberStateValue,
        VUberStateItem, VUberStateOperator, VUberStateRange, VUberStateRangeBoundary,
    },
    wheel_command::{VWheelCommand, WheelBind, WheelCommand, WheelItemPosition},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant, Serialize, Deserialize)]
#[serde(into = "String", try_from = "&str")]
pub enum Item {
    Relic(Zone),
    Water,
    RemoveWater,
    Skill(Skill),
    RemoveSkill(Skill),
    Teleporter(Teleporter),
    RemoveTeleporter(Teleporter),
    Resource(Resource),
    Shard(Shard),
    RemoveShard(Shard),
    BonusItem(BonusItem),
    BonusUpgrade(BonusUpgrade),
    SpiritLight(#[VWrap] u32),
    RemoveSpiritLight(#[VWrap] u32),
    Message(#[VType] Message),
    UberState(#[VType] UberStateItem),
    Command(#[VType] Command),
    WheelCommand(#[VType] WheelCommand),
    ShopCommand(#[VType] ShopCommand),
    SysMessage(SysMessage),
    SetMapMessage(#[VType] String),
}
vdisplay! {
    VItem,
    impl fmt::Display for Item {
        fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
            match self {
                Self::SpiritLight(amount) => write!(f, "{amount} Spirit Light"),
                Self::RemoveSpiritLight(amount) => write!(f, "Remove {amount} Spirit Light"),
                Self::Resource(resource) => write!(f, "{resource}"),
                Self::Skill(skill) => write!(f, "{skill}"),
                Self::RemoveSkill(skill) => write!(f, "Remove {skill}"),
                Self::Shard(shard) => write!(f, "{shard}"),
                Self::RemoveShard(shard) => write!(f, "Remove {shard}"),
                Self::Command(command) => write!(f, "{command}"),
                Self::Teleporter(teleporter) => write!(f, "{teleporter} TP"),
                Self::RemoveTeleporter(teleporter) => write!(f, "Remove {teleporter} TP"),
                Self::Message(message) => write!(f, "Display \"{message}\""),
                Self::UberState(command) => write!(f, "{command}"),
                Self::Water => write!(f, "Clean Water"),
                Self::RemoveWater => write!(f, "Remove Clean Water"),
                Self::BonusItem(bonus_item) => write!(f, "{bonus_item}"),
                Self::BonusUpgrade(bonus_upgrade) => write!(f, "{bonus_upgrade}"),
                Self::Relic(zone) => write!(f, "{zone} Relic"),
                Self::SysMessage(message) => write!(f, "{message}"),
                Self::WheelCommand(command) => write!(f, "{command}"),
                Self::ShopCommand(command) => write!(f, "{command}"),
                Self::SetMapMessage(message) => write!(f, "Set Map Message to \"{message}\""),
            }
        }
    }
}
impl FromStr for Item {
    type Err = String;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = parser::new(input);
        let item = VItem::parse(&mut parser)
            .map_err(|err| err.verbose_display())?
            .resolve(&FxHashMap::default())?;
        parser.expect_end().map_err(|err| err.verbose_display())?;
        Ok(item)
    }
}
impl Item {
    // TODO read from logic file instead
    // TODO is this worth it?
    #[inline]
    pub fn is_progression(&self, difficulty: Difficulty) -> bool {
        match self {
            Item::Resource(resource) => match resource {
                Resource::ShardSlot => difficulty >= Difficulty::Unsafe, // lower difficulties have no issue with the default shards
                Resource::HealthFragment
                | Resource::EnergyFragment
                | Resource::GorlekOre
                | Resource::Keystone => true,
            },
            Item::Skill(skill) => match skill {
                Skill::GladesAncestralLight | Skill::InkwaterAncestralLight => {
                    difficulty >= logical_difficulty::DAMAGE_BUFFS
                }
                Skill::Seir | Skill::WallJump => false,
                Skill::Bash
                | Skill::DoubleJump
                | Skill::Launch
                | Skill::Glide
                | Skill::WaterBreath
                | Skill::Grenade
                | Skill::Grapple
                | Skill::Flash
                | Skill::Spear
                | Skill::Regenerate
                | Skill::Bow
                | Skill::Hammer
                | Skill::Sword
                | Skill::Burrow
                | Skill::Dash
                | Skill::WaterDash
                | Skill::Shuriken
                | Skill::Blaze
                | Skill::Sentry
                | Skill::Flap => true,
            },
            Item::Shard(shard) => match shard {
                Shard::UltraGrapple
                | Shard::Deflector
                | Shard::Magnet
                | Shard::Splinter
                | Shard::Sticky
                | Shard::Fracture => true, // Here logic has to decide the difficulties
                Shard::TripleJump => difficulty >= logical_difficulty::TRIPLE_JUMP,
                Shard::Resilience => difficulty >= logical_difficulty::RESILIENCE,
                Shard::Vitality => difficulty >= logical_difficulty::VITALITY,
                Shard::Energy => difficulty >= logical_difficulty::ENERGY_SHARD,
                Shard::Lifeforce
                | Shard::Finesse
                | Shard::LastStand
                | Shard::Reckless
                | Shard::Wingclip
                | Shard::SpiritSurge => difficulty >= logical_difficulty::DAMAGE_BUFFS,
                Shard::LifePact => difficulty >= logical_difficulty::LIFE_PACT,
                Shard::Overcharge => difficulty >= logical_difficulty::OVERCHARGE,
                Shard::UltraBash => difficulty >= logical_difficulty::ULTRA_BASH,
                Shard::Overflow => difficulty >= logical_difficulty::OVERFLOW,
                Shard::Thorn => difficulty >= logical_difficulty::THORN,
                Shard::Catalyst => difficulty >= logical_difficulty::CATALYST,
                _ => false,
            },
            Item::SpiritLight(_) | Item::Teleporter(_) | Item::Water | Item::UberState(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_multiworld_spread(&self) -> bool {
        // Note that requirement::solutions has logic based on spirit light not being multiworld spread (check_slot_limits)
        !matches!(self, Item::SpiritLight(_))
    }

    #[inline]
    pub fn is_single_instance(&self) -> bool {
        !matches!(
            self,
            Item::SpiritLight(_)
                | Item::RemoveSpiritLight(_)
                | Item::Resource(_)
                | Item::BonusItem(_)
                | Item::BonusUpgrade(_)
                | Item::UberState(_)
                | Item::Command(_)
                | Item::Message(_)
        )
    }

    #[inline]
    pub fn cost(&self) -> u32 {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::SpiritLight(amount) => *amount,
            Item::Resource(Resource::GorlekOre) => 20,
            Item::Resource(Resource::EnergyFragment | Resource::HealthFragment) => 120,
            Item::Resource(Resource::Keystone) => 320,
            Item::Resource(Resource::ShardSlot) => 480,
            Item::Skill(Skill::Regenerate | Skill::WaterBreath) => 200, // Quality-of-Life Skills
            Item::Skill(
                Skill::WallJump | Skill::Dash // Essential Movement
                | Skill::Flap // Counteracting a bias because Flap unlocks rather little
            ) => 1200,
            Item::Skill(Skill::Glide | Skill::Grapple) => 1400,         // Feel-Good Finds
            Item::Skill(Skill::Sword | Skill::Hammer | Skill::Bow | Skill::Shuriken) => 1600, // Basic Weapons
            Item::Skill(Skill::Burrow | Skill::WaterDash | Skill::Grenade | Skill::Flash)
            | Item::Water => 1800, // Key Skills
            Item::Skill(Skill::DoubleJump) => 2000, // Good to find, but this is already biased for by being powerful
            Item::Skill(Skill::Blaze | Skill::Sentry) => 2800, // Tedious Weapons
            Item::Skill(Skill::Bash) => 3000, // Counteracting a bias because Bash unlocks a lot
            Item::Skill(Skill::Spear) => 4000, // No
            Item::Skill(Skill::Launch) => 40000, // Absolutely Broken
            Item::Skill(Skill::GladesAncestralLight | Skill::InkwaterAncestralLight)
            | Item::Shard(_) => 1000,
            Item::Teleporter(Teleporter::Marsh) => 30000,
            Item::Teleporter(_) => 25000,
            _ => 400,
        }
    }

    #[inline]
    pub fn shop_price(&self) -> u32 {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::Resource(Resource::HealthFragment) => 200,
            Item::Resource(Resource::EnergyFragment) => 150,
            Item::Resource(Resource::GorlekOre | Resource::Keystone) => 100,
            Item::Resource(Resource::ShardSlot) => 250,
            Item::Skill(skill) => match skill {
                Skill::WaterBreath | Skill::Regenerate | Skill::Seir => 200,
                Skill::GladesAncestralLight | Skill::InkwaterAncestralLight => 300,
                Skill::Blaze => 420,
                Skill::Launch => 800,
                _ => 500,
            },
            Item::Water => 500,
            Item::Teleporter(_) | Item::Shard(_) => 250,
            Item::BonusItem(_) => 300,
            Item::BonusUpgrade(BonusUpgrade::SentryEfficiency | BonusUpgrade::RapidHammer) => 600,
            Item::BonusUpgrade(_) => 300,
            _ => 200,
        }
    }
    #[inline]
    pub fn random_shop_price(&self) -> bool {
        !matches!(self, Item::Skill(Skill::Blaze))
    }

    /// Returns the [`UberIdentifier`] that gets set to true as a side effect when collecting this [`Item`], if applicable
    pub fn attached_state(&self) -> Option<UberIdentifier> {
        match self {
            Item::Skill(skill) => Some(1000 + *skill as u16),
            Item::Water => Some(2000),
            Item::Teleporter(teleporter) => return Some(teleporter.attached_state()),
            _ => None,
        }
        .map(|uber_id| UberIdentifier::new(6, uber_id))
    }

    pub fn code(&self) -> CodeDisplay<Item> {
        CodeDisplay::new(self, |s, f| match s {
            Item::SpiritLight(amount) => write!(f, "0|{}", amount),
            Item::RemoveSpiritLight(amount) => write!(f, "0|-{}", amount),
            Item::Resource(resource) => write!(f, "1|{}", *resource as u8),
            Item::Skill(skill) => write!(f, "2|{}", *skill as u8),
            Item::RemoveSkill(skill) => write!(f, "2|-{}", *skill as u8),
            Item::Shard(shard) => write!(f, "3|{}", *shard as u8),
            Item::RemoveShard(shard) => write!(f, "3|-{}", *shard as u8),
            Item::Command(command) => write!(f, "4|{}", command.code()),
            Item::Teleporter(teleporter) => write!(f, "5|{}", *teleporter as u8),
            Item::RemoveTeleporter(teleporter) => write!(f, "5|-{}", *teleporter as u8),
            Item::Message(message) => write!(f, "6|{}", message.code()),
            Item::UberState(command) => write!(f, "8|{}", command.code()),
            Item::Water => write!(f, "9|0"),
            Item::RemoveWater => write!(f, "9|-0"),
            Item::BonusItem(bonus) => write!(f, "10|{}", *bonus as u8),
            Item::BonusUpgrade(bonus) => write!(f, "11|{}", *bonus as u8),
            Item::Relic(zone) => write!(f, "14|{}", *zone as u8),
            Item::SysMessage(message) => {
                if let SysMessage::MapRelicList(zone) = message {
                    write!(f, "15|{}|{}", *zone as u8, message.to_id())
                } else {
                    write!(f, "15|{}", message.to_id())
                }
            }
            Item::WheelCommand(command) => write!(f, "16|{}", command.code()),
            Item::ShopCommand(command) => write!(f, "17|{}", command.code()),
            Item::SetMapMessage(message) => write!(f, "18|{}", message),
        })
    }

    pub fn description(&self) -> Option<String> {
        match self {
            Item::BonusItem(bonus_item) => bonus_item.description(),
            Item::BonusUpgrade(bonus_upgrade) => bonus_upgrade.description(),
            _ => None,
        }
    }

    pub fn icon(&self) -> Option<Icon> {
        match self {
            Item::SpiritLight(_) => {
                Some(Icon::File(String::from("assets/icons/game/experience.png")))
            }
            Item::Resource(resource) => resource.icon(),
            Item::Skill(skill) => skill.icon(),
            Item::Shard(shard) => shard.icon(),
            Item::Teleporter(_) => {
                Some(Icon::File(String::from("assets/icons/game/teleporter.png")))
            }
            Item::Message(_) => Some(Icon::File(String::from("assets/icons/game/message.png"))),
            Item::Water => Some(Icon::File(String::from("assets/icons/game/water.png"))),
            Item::BonusItem(bonus_item) => bonus_item.icon(),
            Item::BonusUpgrade(bonus_upgrade) => bonus_upgrade.icon(),
            Item::Relic(_) => Some(Icon::File(String::from("assets/icons/game/relic.png"))),
            _ => None,
        }
    }
    pub fn map_icon(&self) -> MapIcon {
        match self {
            Item::SpiritLight(_) => MapIcon::SpiritLight,
            Item::Resource(resource) => resource.map_icon(),
            Item::Skill(_) => MapIcon::Skill,
            Item::Shard(_) => MapIcon::Shard,
            Item::Teleporter(_) => MapIcon::Teleporter,
            Item::Water | Item::Relic(_) => MapIcon::QuestItem,
            _ => MapIcon::Other,
        }
    }
}

impl From<Item> for String {
    fn from(item: Item) -> String {
        item.code().to_string()
    }
}
impl TryFrom<&str> for Item {
    type Error = String;
    fn try_from<'a>(code: &str) -> Result<Self, Self::Error> {
        Item::from_str(code)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_display() {
        assert_eq!(Item::SpiritLight(45).code().to_string(), "0|45");
        assert_eq!(Item::Resource(Resource::Keystone).code().to_string(), "1|3");
        assert_eq!(Item::Skill(Skill::Launch).code().to_string(), "2|8");
        assert_eq!(
            Item::Skill(Skill::GladesAncestralLight).code().to_string(),
            "2|120"
        );
        assert_eq!(Item::Shard(Shard::Magnet).code().to_string(), "3|8");
        assert_eq!(
            Item::Teleporter(Teleporter::Marsh).code().to_string(),
            "5|16"
        );
        assert_eq!(Item::Water.code().to_string(), "9|0");
        assert_eq!(
            Item::BonusItem(BonusItem::Relic).code().to_string(),
            "10|20"
        );
        assert_eq!(
            Item::BonusUpgrade(BonusUpgrade::ShurikenEfficiency)
                .code()
                .to_string(),
            "11|4"
        );
        assert_eq!(
            Item::Message(Message::new(String::from("8|0|9|7")))
                .code()
                .to_string(),
            "6|8|0|9|7"
        );
    }
}
