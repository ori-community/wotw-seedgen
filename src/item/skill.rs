use num_enum::TryFromPrimitive;

use crate::{util::Icon, settings::Difficulty};

#[derive(Debug, seedgen_derive::Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum Skill {
    Bash = 0,
    WallJump = 3,
    DoubleJump = 5,
    Launch = 8,
    Glide = 14,
    WaterBreath = 23,
    Grenade = 51,
    Grapple = 57,
    Flash = 62,
    Spear = 74,
    Regenerate = 77,
    Bow = 97,
    Hammer = 98,
    Sword = 100,
    Burrow = 101,
    Dash = 102,
    WaterDash = 104,
    Shuriken = 106,
    Seir = 108,
    Blaze = 115,
    Sentry = 116,
    Flap = 118,
    #[num_enum(alternatives = [121])]
    AncestralLight = 120,
}
impl Skill {
    pub fn icon(self) -> Option<Icon> {
        match self {
            Skill::Bash => Some(Icon::Spell(3000)),
            Skill::WallJump => None,
            Skill::DoubleJump => Some(Icon::Spell(4001)),
            Skill::Launch => Some(Icon::Spell(2019)),
            Skill::Glide => Some(Icon::Spell(4002)),
            Skill::WaterBreath => Some(Icon::Opher(10)),
            Skill::Grenade => Some(Icon::Spell(2010)),
            Skill::Grapple => Some(Icon::Spell(3001)),
            Skill::Flash => Some(Icon::Spell(2004)),
            Skill::Spear => Some(Icon::Opher(6)),
            Skill::Regenerate => Some(Icon::Spell(2013)),
            Skill::Bow => Some(Icon::Spell(1001)),
            Skill::Hammer => Some(Icon::Opher(2)),
            Skill::Sword => Some(Icon::Spell(1002)),
            Skill::Burrow => Some(Icon::Spell(3002)),
            Skill::Dash => Some(Icon::Spell(4000)),
            Skill::WaterDash => Some(Icon::Spell(4004)),
            Skill::Shuriken => Some(Icon::Opher(4)),
            Skill::Seir => Some(Icon::Spell(2018)),
            Skill::Blaze => Some(Icon::Opher(8)),
            Skill::Sentry => Some(Icon::Opher(0)),
            Skill::Flap => Some(Icon::Spell(3005)),
            Skill::AncestralLight => Some(Icon::File(String::from("assets/icons/game/ancestrallight1.png"))),
        }
    }

    pub fn energy_cost(self) -> f32 {
        match self {
            Skill::Bow => 0.25,
            Skill::Shuriken => 0.5,
            Skill::Grenade | Skill::Flash | Skill::Regenerate | Skill::Blaze | Skill::Sentry => 1.0,
            Skill::Spear => 2.0,
            _ => 0.0,
        }
    }

    pub fn damage(self, difficulty: Difficulty) -> f32 {
        match self {
            Skill::Bow | Skill::Sword => 4.0,
            Skill::Launch => 5.0,
            Skill::Hammer | Skill::Flash => 12.0,
            Skill::Shuriken => 7.0,
            Skill::Grenade => if difficulty >= Difficulty::Unsafe { 8.0 } else { 4.0 },
            Skill::Spear => 20.0,
            Skill::Blaze => 3.0,
            Skill::Sentry => 8.8,
            _ => 0.0,
        }
    }
    pub fn burn_damage(self) -> f32 {
        match self {
            Skill::Grenade => 9.0,
            Skill::Blaze => 10.8,
            _ => 0.0,
        }
    }

    pub fn damage_per_energy(self, difficulty: Difficulty) -> f32 {
        // (self.damage(unsafe_paths) + self.burn_damage()) / self.energy_cost()
        (10.0 / (self.damage(difficulty) + self.burn_damage())).ceil() * self.energy_cost()  // "how much energy do you need to deal 10 damage" leads to a more realistic ordering than pure damage per energy
    }
}
