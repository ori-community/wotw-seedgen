mod resource;
mod skill;
mod shard;
mod command;
mod teleporter;
mod uber_state;
mod bonus_item;
mod bonus_upgrade;
mod hint;
mod sysmessage;
mod wheel_command;
mod shop_command;

use std::fmt;

use crate::headers;
use crate::util::{Difficulty, Zone, Icon};

pub use self::{
    resource::Resource,
    skill::Skill,
    shard::Shard,
    command::{Command, ToggleCommand},
    teleporter::Teleporter,
    uber_state::{UberStateItem, UberStateOperator, UberStateRange, UberStateRangeBoundary},
    bonus_item::BonusItem,
    bonus_upgrade::BonusUpgrade,
    hint::{Hint, ZoneHintType},
    sysmessage::SysMessage,
    wheel_command::{WheelCommand, WheelBind},
    shop_command::ShopCommand,
};

#[allow(clippy::pub_enum_variant_names)]
#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub enum Item {
    SpiritLight(u16),
    RemoveSpiritLight(u16),
    Resource(Resource),
    Skill(Skill),
    RemoveSkill(Skill),
    Shard(Shard),
    RemoveShard(Shard),
    Command(Command),
    Teleporter(Teleporter),
    RemoveTeleporter(Teleporter),
    Message(String),
    UberState(UberStateItem),
    Water,
    RemoveWater,
    BonusItem(BonusItem),
    BonusUpgrade(BonusUpgrade),
    Hint(Hint),
    CheckableHint(u16, u16, Vec<Item>),
    Relic(Zone),
    SysMessage(SysMessage),
    WheelCommand(WheelCommand),
    ShopCommand(ShopCommand),
}
impl fmt::Display for Item {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Item::SpiritLight(1) => write!(f, "Spirit Light"),
            Item::SpiritLight(amount) => write!(f, "{} Spirit Light", amount),
            Item::RemoveSpiritLight(amount) => write!(f, "Remove {} Spirit Light", amount),
            Item::Resource(resource) => write!(f, "{}", resource),
            Item::Skill(skill) => write!(f, "{}", skill),
            Item::RemoveSkill(skill) => write!(f, "Remove {}", skill),
            Item::Shard(shard) => write!(f, "{}", shard),
            Item::RemoveShard(shard) => write!(f, "Remove {}", shard),
            Item::Command(command) => write!(f, "4|{}", command),
            Item::Teleporter(teleporter) => write!(f, "{}", teleporter),
            Item::RemoveTeleporter(teleporter) => write!(f, "Remove {}", teleporter),
            Item::Message(message) => {
                let mut message = message.clone();
                let mut last_index = 0;

                while let Some(mut start_index) = message[last_index..].find("$[") {
                    start_index += last_index;
                    last_index = start_index;
                    let after_bracket = start_index + 2;
                    let mut end_index = 0;

                    let mut depth = 1;
                    for (index, byte) in message[after_bracket..].bytes().enumerate() {
                        if byte == b'[' { depth += 1; }
                        if byte == b']' { depth -= 1; }
                        if depth == 0 {
                            end_index = after_bracket + index;
                            break;
                        }
                    }
                    if end_index == 0 { break; }

                    let pickup = &message[after_bracket..end_index];
                    if let Ok(pickup) = headers::parser::parse_pickup(pickup) {
                        message.replace_range(start_index..=end_index, &pickup.to_string());
                    } else { last_index = end_index; } // if nothing ends up getting replaced, move on
                }

                write!(f, "{}", message)
            },
            Item::UberState(command) => write!(f, "8|{}", command),
            Item::Water => write!(f, "Clean Water"),
            Item::RemoveWater => write!(f, "Remove Clean Water"),
            Item::BonusItem(bonus_item) => write!(f, "{}", bonus_item),
            Item::BonusUpgrade(bonus_upgrade) => write!(f, "{}", bonus_upgrade),
            Item::Hint(hint) => write!(f, "{}", hint),
            Item::CheckableHint(_, _, hints) => {
                let hints = hints.iter().map(|hint| hint.to_string()).collect::<Vec<_>>();
                write!(f, "{}", hints.join(", "))
            },
            Item::Relic(zone) => write!(f, "{} Relic", zone),
            Item::SysMessage(message) => write!(f, "{}", message),
            Item::WheelCommand(command) => write!(f, "16|{}", command),
            Item::ShopCommand(command) => write!(f, "17|{}", command),
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
                Skill::AncestralLight => difficulty >= Difficulty::Unsafe,
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
            Item::Skill(Skill::AncestralLight) |
            Item::BonusItem(_) | Item::BonusUpgrade(_) |
            Item::UberState(_) | Item::Command(_) | Item::Message(_)
        )
    }
    #[inline]
    pub fn is_checkable(&self) -> bool {
        matches!(self,
            Item::Skill(_) |
            Item::Shard(_) |
            Item::Teleporter(_) |
            Item::Water
        )
    }

    #[inline]
    pub fn cost(&self) -> u16 {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::SpiritLight(amount) => *amount,
            Item::Resource(Resource::Ore) => 20,
            Item::Resource(Resource::Health) => 240,
            Item::Resource(Resource::Keystone) => 320,
            Item::Resource(Resource::Energy) => 320,
            Item::Resource(Resource::ShardSlot) => 480,
            Item::Skill(Skill::Regenerate) | Item::Skill(Skill::WaterBreath) => 200,  // Quality-of-Life Skills
            Item::Skill(Skill::Sword) | Item::Skill(Skill::Hammer) => 600,  // Essential Weapons
            Item::Skill(Skill::WallJump) | Item::Skill(Skill::DoubleJump) | Item::Skill(Skill::Dash) => 1200,  // Essential Movement
            Item::Skill(Skill::Glide) | Item::Skill(Skill::Grapple) => 1400,  // Feel-Good Finds
            Item::Skill(Skill::Bow) | Item::Skill(Skill::Shuriken) => 1600,  // Secondary Weapons
            Item::Skill(Skill::Burrow) | Item::Skill(Skill::Bash) | Item::Skill(Skill::Flap) | Item::Skill(Skill::WaterDash) |
            Item::Skill(Skill::Grenade) | Item::Skill(Skill::Flash) | Item::Skill(Skill::Seir) | Item::Water => 1800,  // Key Skills
            Item::Skill(Skill::Blaze) | Item::Skill(Skill::Sentry) | Item::Skill(Skill::Spear) => 2800,  // Tedious Weapons
            Item::Skill(Skill::AncestralLight) => 3000,  // Unhinted Skill
            Item::Skill(Skill::Launch) => 40000,  // Absolutely Broken
            Item::Shard(_) => 1000,
            Item::Teleporter(Teleporter::Marsh) => 30000,
            Item::Teleporter(_) => 25000,
            _ => 400,
        }
    }

    #[inline]
    pub fn shop_price(&self) -> u16 {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::Resource(Resource::Health) => 200,
            Item::Resource(Resource::Energy) => 150,
            Item::Resource(Resource::Ore) |
            Item::Resource(Resource::Keystone) => 100,
            Item::Resource(Resource::ShardSlot) => 400,
            Item::Skill(Skill::Blaze) => 420,
            Item::Skill(_) => 500,
            Item::Water => 400,
            Item::Teleporter(_) => 250,
            Item::Shard(_) |
            Item::BonusItem(_) => 300,
            Item::BonusUpgrade(BonusUpgrade::SentryEfficiency) |
            Item::BonusUpgrade(BonusUpgrade::RapidHammer) => 600,
            Item::BonusUpgrade(_) => 300,
            Item::Hint(hint) => match hint.zone {
                Zone::Burrows | Zone::Willow => 50,
                Zone::Hollow | Zone::Wellspring | Zone::Woods | Zone::Reach | Zone::Depths | Zone::Pools | Zone::Wastes | Zone::Ruins => 150,
                Zone::Marsh | Zone::Glades => 200,
                _ => 150,
            },
            _ => 200,
        }
    }
    #[inline]
    pub fn random_shop_price(&self) -> bool {
        #[allow(clippy::match_same_arms)]
        match self {
            Item::Resource(_) => true,
            Item::Skill(Skill::Blaze) => false,
            Item::Skill(_) |
            Item::Water |
            Item::Teleporter(_) |
            Item::Shard(_) |
            Item::BonusItem(_) |
            Item::Hint(_) => true,
            _ => false,
        }
    }

    pub fn code(&self) -> String {
        match self {
            Item::SpiritLight(amount) => format!("0|{}", amount),
            Item::RemoveSpiritLight(amount) => format!("0|-{}", amount),
            Item::Resource(resource) => format!("1|{}", resource.to_id()),
            Item::Skill(skill) => format!("2|{}", skill.to_id()),
            Item::RemoveSkill(skill) => format!("2|-{}", skill.to_id()),
            Item::Shard(shard) => format!("3|{}", shard.to_id()),
            Item::RemoveShard(shard) => format!("3|-{}", shard.to_id()),
            Item::Command(command) => format!("4|{}", command),
            Item::Teleporter(teleporter) => format!("5|{}", teleporter.to_id()),
            Item::RemoveTeleporter(teleporter) => format!("5|-{}", teleporter.to_id()),
            Item::Message(message) => format!("6|{}", message),
            Item::UberState(command) => format!("8|{}", command),
            Item::Water => String::from("9|0"),
            Item::RemoveWater => String::from("9|-0"),
            Item::BonusItem(bonus) => format!("10|{}", bonus.to_id()),
            Item::BonusUpgrade(bonus) => format!("11|{}", bonus.to_id()),
            Item::Hint(hint) => format!("12|{}|{}", hint.zone.to_id(), hint.hint_type.to_id()),
            Item::CheckableHint(base_price, price_modifier, hint) => {
                let hint = hint.iter().map(|item| str::replace(&item.code(), '-', "|")).collect::<Vec<_>>();
                format!("13|{}|{}|{}", base_price, price_modifier, hint.join(","))
            },
            Item::Relic(zone) => format!("14|{}", zone.to_id()),
            Item::SysMessage(message) => match message {
                    SysMessage::MapRelicList(zone) => format!("15|{}|{}", zone.to_id(), message.to_id()),
                    _ => format!("15|{}", message.to_id()),
                },
            Item::WheelCommand(command) => format!("16|{}", command),
            Item::ShopCommand(command) => format!("17|{}", command),
        }
    }

    pub fn icon(&self) -> Option<Icon> {
        match self {
            Item::SpiritLight(_) => Some(Icon::Map(17)),
            Item::Resource(resource) => resource.icon(),
            Item::Skill(skill) => skill.icon(),
            Item::Shard(shard) => shard.icon(),
            Item::Teleporter(_) => Some(Icon::Map(13)),
            Item::Message(_) => Some(Icon::Map(25)),
            Item::Water => Some(Icon::File(String::from("icons/water.png"))),
            Item::BonusItem(bonus_item) => bonus_item.icon(),
            Item::BonusUpgrade(bonus_upgrade) => bonus_upgrade.icon(),
            Item::Relic(_) => Some(Icon::File(String::from("icons/relic.png"))),
            _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use hint::ZoneHintType;
    use crate::util::Zone;

    #[test]
    fn item_display() {
        assert_eq!(Item::SpiritLight(45).code(), "0|45");
        assert_eq!(Item::Resource(Resource::Keystone).code(), "1|3");
        assert_eq!(Item::Skill(Skill::Launch).code(), "2|8");
        assert_eq!(Item::Skill(Skill::AncestralLight).code(), "2|120");
        assert_eq!(Item::Shard(Shard::Magnet).code(), "3|8");
        assert_eq!(Item::Teleporter(Teleporter::Marsh).code(), "5|16");
        assert_eq!(Item::Water.code(), "9|0");
        assert_eq!(Item::BonusItem(BonusItem::Relic).code(), "10|20");
        assert_eq!(Item::BonusUpgrade(BonusUpgrade::ShurikenEfficiency).code(), "11|4");
        assert_eq!(Item::Hint(Hint { zone: Zone::Void, hint_type: ZoneHintType::Skills }).code(), "12|12|1");
        assert_eq!(Item::Message(String::from("8|0|9|7")).code(), "6|8|0|9|7");
    }
}
