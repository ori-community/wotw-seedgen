use crate::inventory::Inventory;
use wotw_seedgen_data::{uber_identifier, MapIcon, Shard, Skill, Teleporter, WeaponUpgrade};
use wotw_seedgen_seed_language::output::{
    ArithmeticOperator, CommandBoolean, CommandFloat, CommandInteger, CommandVoid, Operation,
};

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum CommonItem {
    SpiritLight(usize),
    HealthFragment,
    EnergyFragment,
    GorlekOre,
    Keystone,
    ShardSlot,
    WeaponUpgrade(WeaponUpgrade),
    Shard(Shard),
    Teleporter(Teleporter),
    Skill(Skill),
    CleanWater,
}

impl CommonItem {
    // TODO could do an iterator here and it would probably be a performance advantage
    pub fn from_command(command: &CommandVoid) -> Vec<Self> {
        match command {
            CommandVoid::Multi { commands } => {
                commands.iter().flat_map(Self::from_command).collect()
            }
            CommandVoid::StoreBoolean {
                uber_identifier,
                value: CommandBoolean::Constant { value: true },
                ..
            } => {
                if let Some(weapon_upgrade) = WeaponUpgrade::from_uber_identifier(*uber_identifier)
                {
                    vec![CommonItem::WeaponUpgrade(weapon_upgrade)]
                } else if let Some(shard) = Shard::from_uber_identifier(*uber_identifier) {
                    vec![CommonItem::Shard(shard)]
                } else if let Some(teleporter) = Teleporter::from_uber_identifier(*uber_identifier)
                {
                    vec![CommonItem::Teleporter(teleporter)]
                } else if let Some(skill) = Skill::from_uber_identifier(*uber_identifier) {
                    vec![CommonItem::Skill(skill)]
                } else if *uber_identifier == uber_identifier::CLEAN_WATER {
                    vec![CommonItem::CleanWater]
                } else {
                    vec![]
                }
            }
            CommandVoid::StoreInteger {
                uber_identifier,
                value: CommandInteger::Arithmetic { operation },
                ..
            } => match &**operation {
                Operation {
                    left:
                        CommandInteger::FetchInteger {
                            uber_identifier: fetch_identifier,
                        },
                    operator: ArithmeticOperator::Add,
                    right: CommandInteger::Constant { value: amount },
                } if fetch_identifier == uber_identifier && *amount >= 0 => {
                    let amount = *amount as usize;
                    match *uber_identifier {
                        uber_identifier::SPIRIT_LIGHT => {
                            vec![CommonItem::SpiritLight(amount)]
                        }
                        uber_identifier::MAX_HEALTH => {
                            vec![CommonItem::HealthFragment; amount / 5]
                        }
                        uber_identifier::GORLEK_ORE => {
                            vec![CommonItem::GorlekOre; amount]
                        }
                        uber_identifier::KEYSTONES => vec![CommonItem::Keystone; amount],
                        uber_identifier::SHARD_SLOTS => {
                            vec![CommonItem::ShardSlot; amount]
                        }

                        _ => vec![],
                    }
                }
                _ => vec![],
            },
            CommandVoid::StoreFloat {
                uber_identifier: uber_identifier::MAX_ENERGY,
                value: CommandFloat::Arithmetic { operation },
                ..
            } => match &**operation {
                Operation {
                    left:
                        CommandFloat::FetchFloat {
                            uber_identifier: uber_identifier::MAX_ENERGY,
                        },
                    operator: ArithmeticOperator::Add,
                    right: CommandFloat::Constant { value },
                } => vec![CommonItem::EnergyFragment; (**value * 2.) as usize],
                _ => vec![],
            },
            _ => vec![],
        }
    }

    pub fn grant(self, inventory: &mut Inventory) {
        match self {
            CommonItem::SpiritLight(amount) => {
                inventory.spirit_light += amount;
            }
            CommonItem::HealthFragment => {
                inventory.health += 5;
            }
            CommonItem::EnergyFragment => {
                inventory.energy += 0.5;
            }
            CommonItem::GorlekOre => {
                inventory.gorlek_ore += 1;
            }
            CommonItem::Keystone => {
                inventory.keystones += 1;
            }
            CommonItem::ShardSlot => {
                inventory.shard_slots += 1;
            }
            CommonItem::WeaponUpgrade(weapon_upgrade) => {
                inventory.weapon_upgrades.insert(weapon_upgrade);
            }
            CommonItem::Shard(shard) => {
                inventory.shards.insert(shard);
            }
            CommonItem::Teleporter(teleporter) => {
                inventory.teleporters.insert(teleporter);
            }
            CommonItem::Skill(skill) => {
                inventory.skills.insert(skill);
            }
            CommonItem::CleanWater => {
                inventory.clean_water = true;
            }
        }
    }
    pub fn remove(self, inventory: &mut Inventory) {
        match self {
            CommonItem::SpiritLight(amount) => {
                inventory.spirit_light -= amount;
            }
            CommonItem::HealthFragment => {
                inventory.health -= 5;
            }
            CommonItem::EnergyFragment => {
                inventory.energy -= 0.5;
            }
            CommonItem::GorlekOre => {
                inventory.gorlek_ore -= 1;
            }
            CommonItem::Keystone => {
                inventory.keystones -= 1;
            }
            CommonItem::ShardSlot => {
                inventory.shard_slots -= 1;
            }
            CommonItem::WeaponUpgrade(weapon_upgrade) => {
                inventory.weapon_upgrades.remove(&weapon_upgrade);
            }
            CommonItem::Shard(shard) => {
                inventory.shards.remove(&shard);
            }
            CommonItem::Teleporter(teleporter) => {
                inventory.teleporters.remove(&teleporter);
            }
            CommonItem::Skill(skill) => {
                inventory.skills.remove(&skill);
            }
            CommonItem::CleanWater => {
                inventory.clean_water = false;
            }
        }
    }

    pub const fn map_icon(&self) -> MapIcon {
        match self {
            CommonItem::SpiritLight(_) => MapIcon::Experience,
            CommonItem::HealthFragment => MapIcon::HealthFragment,
            CommonItem::EnergyFragment => MapIcon::EnergyFragment,
            CommonItem::GorlekOre => MapIcon::Ore,
            CommonItem::Keystone => MapIcon::Keystone,
            CommonItem::ShardSlot => MapIcon::ShardSlotUpgrade,
            CommonItem::WeaponUpgrade(_) => MapIcon::BonusItem, // TODO is this good?
            CommonItem::Shard(_) => MapIcon::SpiritShard,
            CommonItem::Teleporter(_) => MapIcon::Teleporter,
            CommonItem::Skill(_) => MapIcon::AbilityPedestal,
            CommonItem::CleanWater => MapIcon::CleanWater,
        }
    }

    pub const fn shop_price(&self) -> f32 {
        match self {
            CommonItem::GorlekOre | CommonItem::Keystone => 100.,
            CommonItem::ShardSlot => 250.,
            CommonItem::HealthFragment => 200.,
            CommonItem::EnergyFragment => 150.,
            CommonItem::Skill(skill) => match skill {
                Skill::WaterBreath | Skill::Regenerate | Skill::Seir => 200.,
                Skill::GladesAncestralLight | Skill::MarshAncestralLight => 300.,
                Skill::Blaze => 420.,
                Skill::Launch => 800.,
                _ => 500.,
            },
            CommonItem::CleanWater => 500.,
            CommonItem::Teleporter(_) | CommonItem::Shard(_) => 250.,
            _ => 200.,
        }
    }
}
