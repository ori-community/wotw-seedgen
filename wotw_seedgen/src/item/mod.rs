mod resource;
mod skill;
mod shard;
mod command;
mod teleporter;
mod message;
mod uber_state;
mod bonus_item;
mod bonus_upgrade;
mod sysmessage;
mod wheel_command;
mod shop_command;

use std::fmt;
use std::str::FromStr;

use rustc_hash::FxHashMap;
use serde::{Serialize, Serializer};
use wotw_seedgen_derive::VVariant;

use crate::header::parser;
use crate::header::{VResolve, vdisplay};
use crate::settings::Difficulty;
use crate::util::{Zone, Icon, MapIcon, UberState, UberIdentifier};

pub use self::{
    resource::Resource,
    skill::Skill,
    shard::Shard,
    command::{Command, VCommand, ToggleCommand, EquipSlot},
    teleporter::Teleporter,
    message::{Message, VMessage},
    uber_state::{UberStateItem, VUberStateItem, UberStateOperator, VUberStateOperator, UberStateRange, VUberStateRange, UberStateRangeBoundary, VUberStateRangeBoundary},
    bonus_item::BonusItem,
    bonus_upgrade::BonusUpgrade,
    sysmessage::SysMessage,
    wheel_command::{WheelCommand, VWheelCommand, WheelItemPosition, WheelBind},
    shop_command::{ShopCommand, VShopCommand},
};

#[derive(Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, VVariant)]
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
        let remaining = parser.remaining();
        if remaining.is_empty() {
            Ok(item)
        } else {
            Err(format!("Input left after parsing item: \"{remaining}\""))
        }
    }
}
impl Item {
    // TODO read from logic file instead
    #[inline]
    pub fn is_progression(&self, difficulty: Difficulty) -> bool {
        match self {
            Item::Resource(resource) => match resource {
                Resource::ShardSlot => difficulty >= Difficulty::Unsafe,
                Resource::Health | Resource::Energy | Resource::Ore | Resource::Keystone => true,
            },
            Item::Skill(skill) => match skill {
                Skill::AncestralLight1 | Skill::AncestralLight2 => difficulty >= Difficulty::Unsafe,
                Skill::Shuriken | Skill::Blaze | Skill::Sentry => difficulty >= Difficulty::Gorlek,
                Skill::Seir | Skill::WallJump => false,
                Skill::Bash |
                Skill::DoubleJump |
                Skill::Launch |
                Skill::Glide |
                Skill::WaterBreath |
                Skill::Grenade |
                Skill::Grapple |
                Skill::Flash |
                Skill::Spear |
                Skill::Regenerate |
                Skill::Bow |
                Skill::Hammer |
                Skill::Sword |
                Skill::Burrow |
                Skill::Dash |
                Skill::WaterDash |
                Skill::Flap => true,
            },
            Item::Shard(shard) => match shard {
                Shard::Overcharge |
                Shard::Wingclip |
                Shard::Magnet |
                Shard::Splinter |
                Shard::Reckless |
                Shard::LifePact |
                Shard::LastStand |
                Shard::UltraBash |
                Shard::UltraGrapple |
                Shard::Overflow |
                Shard::Thorn |
                Shard::Catalyst |
                Shard::Sticky |
                Shard::Finesse |
                Shard::SpiritSurge |
                Shard::Lifeforce |
                Shard::Deflector |
                Shard::Fracture => difficulty >= Difficulty::Unsafe,
                Shard::TripleJump |
                Shard::Resilience |
                Shard::Vitality |
                Shard::Energy => difficulty >= Difficulty::Gorlek,
                Shard::Bounty |
                Shard::Swap |
                Shard::Quickshot |
                Shard::SpiritLightHarvest |
                Shard::LifeHarvest |
                Shard::EnergyHarvest |
                Shard::Sense |
                Shard::Turmoil |
                Shard::Arcing => false,
            },
            Item::SpiritLight(_) | Item::Teleporter(_) | Item::Water | Item::UberState(_) => true,
            _ => false,
        }
    }
    #[inline]
    pub fn is_multiworld_spread(&self) -> bool {
        !matches!(self, Item::SpiritLight(_))
    }

    #[inline]
    pub fn is_single_instance(&self) -> bool {
        !matches!(self,
            Item::SpiritLight(_) | Item::RemoveSpiritLight(_) |
            Item::Resource(_) |
            Item::BonusItem(_) | Item::BonusUpgrade(_) |
            Item::UberState(_) | Item::Command(_) | Item::Message(_)
        )
    }

    #[inline]
    pub fn cost(&self) -> u32 {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::SpiritLight(amount) => *amount,
            Item::Resource(Resource::Ore) => 20,
            Item::Resource(Resource::Energy | Resource::Health) => 120,
            Item::Resource(Resource::Keystone) => 320,
            Item::Resource(Resource::ShardSlot) => 480,
            Item::Skill(Skill::Regenerate | Skill::WaterBreath) => 200,  // Quality-of-Life Skills
            Item::Skill(Skill::WallJump | Skill::DoubleJump | Skill::Dash) => 1200,  // Essential Movement
            Item::Skill(Skill::Glide | Skill::Grapple) => 1400,  // Feel-Good Finds
            Item::Skill(Skill::Sword | Skill::Hammer | Skill::Bow | Skill::Shuriken) => 1600,  // Basic Weapons
            Item::Skill(Skill::Burrow | Skill::Bash | Skill::Flap | Skill::WaterDash | Skill::Grenade | Skill::Flash | Skill::Seir) | Item::Water => 1800,  // Key Skills
            Item::Skill(Skill::Blaze | Skill::Sentry) => 2800,  // Tedious Weapons
            Item::Skill(Skill::AncestralLight1 | Skill::AncestralLight2) => 3000,  // Unhinted Skill
            Item::Skill(Skill::Spear) => 4000,  // No
            Item::Skill(Skill::Launch) => 40000,  // Absolutely Broken
            Item::Shard(_) => 1000,
            Item::Teleporter(Teleporter::Marsh) => 30000,
            Item::Teleporter(_) => 25000,
            _ => 400,
        }
    }

    #[inline]
    pub fn shop_price(&self) -> u32 {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::Resource(Resource::Health) => 200,
            Item::Resource(Resource::Energy) => 150,
            Item::Resource(Resource::Ore | Resource::Keystone) => 100,
            Item::Resource(Resource::ShardSlot) => 250,
            Item::Skill(skill) => match skill {
                Skill::WaterBreath | Skill::Regenerate | Skill::Seir => 200,
                Skill::AncestralLight1 | Skill::AncestralLight2 => 300,
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

    /// Returns the UberState that gets set when collecting this item, if applicable
    pub fn triggered_state(&self) -> Option<UberState> {
        match self {
            Item::Skill(skill) => Some(1000 + *skill as u16),
            Item::Water => Some(2000),
            Item::Teleporter(teleporter) => return Some(teleporter.triggered_state()),
            _ => None,
        }.map(|uber_id|
            UberState {
                identifier: UberIdentifier {
                    uber_group: 6,
                    uber_id,
                },
                value: String::new(),
            }
        )
    }

    pub fn code(&self) -> String {
        match self {
            Item::SpiritLight(amount) => format!("0|{}", amount),
            Item::RemoveSpiritLight(amount) => format!("0|-{}", amount),
            Item::Resource(resource) => format!("1|{}", *resource as u8),
            Item::Skill(skill) => format!("2|{}", *skill as u8),
            Item::RemoveSkill(skill) => format!("2|-{}", *skill as u8),
            Item::Shard(shard) => format!("3|{}", *shard as u8),
            Item::RemoveShard(shard) => format!("3|-{}", *shard as u8),
            Item::Command(command) => format!("4|{}", command.code()),
            Item::Teleporter(teleporter) => format!("5|{}", *teleporter as u8),
            Item::RemoveTeleporter(teleporter) => format!("5|-{}", *teleporter as u8),
            Item::Message(message) => format!("6|{}", message.code()),
            Item::UberState(command) => format!("8|{}", command.code()),
            Item::Water => String::from("9|0"),
            Item::RemoveWater => String::from("9|-0"),
            Item::BonusItem(bonus) => format!("10|{}", *bonus as u8),
            Item::BonusUpgrade(bonus) => format!("11|{}", *bonus as u8),
            Item::Relic(zone) => format!("14|{}", *zone as u8),
            Item::SysMessage(message) => match message {
                    SysMessage::MapRelicList(zone) => format!("15|{}|{}", *zone as u8, message.to_id()),
                    _ => format!("15|{}", message.to_id()),
                },
            Item::WheelCommand(command) => format!("16|{}", command.code()),
            Item::ShopCommand(command) => format!("17|{}", command.code()),
        }
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
            Item::SpiritLight(_) => Some(Icon::File(String::from("assets/icons/game/experience.png"))),
            Item::Resource(resource) => resource.icon(),
            Item::Skill(skill) => skill.icon(),
            Item::Shard(shard) => shard.icon(),
            Item::Teleporter(_) => Some(Icon::File(String::from("assets/icons/game/teleporter.png"))),
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

impl Serialize for Item {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(&self.code())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn item_display() {
        assert_eq!(Item::SpiritLight(45).code(), "0|45");
        assert_eq!(Item::Resource(Resource::Keystone).code(), "1|3");
        assert_eq!(Item::Skill(Skill::Launch).code(), "2|8");
        assert_eq!(Item::Skill(Skill::AncestralLight1).code(), "2|120");
        assert_eq!(Item::Shard(Shard::Magnet).code(), "3|8");
        assert_eq!(Item::Teleporter(Teleporter::Marsh).code(), "5|16");
        assert_eq!(Item::Water.code(), "9|0");
        assert_eq!(Item::BonusItem(BonusItem::Relic).code(), "10|20");
        assert_eq!(Item::BonusUpgrade(BonusUpgrade::ShurikenEfficiency).code(), "11|4");
        assert_eq!(Item::Message(Message::new(String::from("8|0|9|7"))).code(), "6|8|0|9|7");
    }
}
