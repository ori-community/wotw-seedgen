use std::collections::HashSet;
use wotw_seedgen_data::{Shard, Skill, Teleporter, WeaponUpgrade};
use wotw_seedgen_seed_language::output::CommandVoid;

use crate::{common_item::CommonItem, inventory::Inventory};

pub trait Cost {
    fn cost(&self) -> usize;
}
impl Cost for Skill {
    fn cost(&self) -> usize {
        match self {
            Skill::Regenerate | Skill::WaterBreath => 200, // Quality-of-Life Skills
            Skill::WallJump | Skill::Dash // Essential Movement
            | Skill::Flap // Counteracting a bias because Flap unlocks rather little
             => 1200,
            Skill::Glide | Skill::Grapple => 1400,         // Feel-Good Finds
            Skill::Sword | Skill::Hammer | Skill::Bow | Skill::Shuriken => 1600, // Basic Weapons
            Skill::Burrow | Skill::WaterDash | Skill::Grenade | Skill::Flash => 1800, // Key Skills
            Skill::DoubleJump => 2000, // Good to find, but this is already biased for by being powerful
            Skill::Blaze | Skill::Sentry => 2800, // Tedious Weapons
            Skill::Bash => 3000, // Counteracting a bias because Bash unlocks a lot
            Skill::Spear => 4000, // No
            Skill::Launch => 40000, // Absolutely Broken
            Skill::GladesAncestralLight | Skill::InkwaterAncestralLight => 1000,
            Skill::SpiritFlame | Skill::Seir | Skill::BowCharge | Skill::Magnet | Skill::WeaponCharge => 0 // ?
        }
    }
}
impl Cost for Shard {
    fn cost(&self) -> usize {
        1000
    }
}
impl Cost for Teleporter {
    fn cost(&self) -> usize {
        match self {
            Teleporter::Inkwater => 30000,
            _ => 25000,
        }
    }
}
impl Cost for WeaponUpgrade {
    fn cost(&self) -> usize {
        400
    }
}
impl Cost for Inventory {
    fn cost(&self) -> usize {
        self.spirit_light.max(0)
            + self.health_fragments().max(0) * CommonItem::HealthFragment.cost()
            + self.energy_fragments().max(0) * CommonItem::EnergyFragment.cost()
            + self.gorlek_ore.max(0) * CommonItem::GorlekOre.cost()
            + self.keystones.max(0) * CommonItem::Keystone.cost()
            + self.shard_slots.max(0) * CommonItem::ShardSlot.cost()
            + self.weapon_upgrades.cost()
            + self.shards.cost()
            + self.teleporters.cost()
            + self.skills.cost()
            + self.clean_water as usize * CommonItem::CleanWater.cost()
    }
}
impl Cost for CommandVoid {
    fn cost(&self) -> usize {
        CommonItem::from_command(self).cost()
    }
}
impl Cost for CommonItem {
    fn cost(&self) -> usize {
        match self {
            CommonItem::SpiritLight(amount) => usize::max(*amount, 0),
            CommonItem::HealthFragment | CommonItem::EnergyFragment => 120,
            CommonItem::GorlekOre => 20,
            CommonItem::Keystone => 320,
            CommonItem::ShardSlot => 480,
            CommonItem::WeaponUpgrade(weapon_upgrade) => weapon_upgrade.cost(),
            CommonItem::Shard(shard) => shard.cost(),
            CommonItem::Teleporter(teleporter) => teleporter.cost(),
            CommonItem::Skill(skill) => skill.cost(),
            CommonItem::CleanWater => 1800, // Key Item
        }
    }
}
impl<C: Cost> Cost for Vec<C> {
    fn cost(&self) -> usize {
        self.iter().map(|c| c.cost()).sum()
    }
}
impl<C: Cost, S> Cost for HashSet<C, S> {
    fn cost(&self) -> usize {
        self.iter().map(|c| c.cost()).sum()
    }
}
