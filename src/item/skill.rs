use num_enum::TryFromPrimitive;
use seedgen_derive::FromStr;

use crate::util::icon::OpherIcon;
use crate::util::{Icon, Spell};
use crate::settings::Difficulty;

#[derive(Debug, seedgen_derive::Display, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
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
    AncestralLight1 = 120,
    AncestralLight2 = 121,
}
impl Skill {
    pub fn icon(self) -> Option<Icon> {
        let icon = match self {
            Skill::Bash => Icon::Spell(Spell::Bash),
            Skill::WallJump => return None,
            Skill::DoubleJump => Icon::Spell(Spell::Bounce),
            Skill::Launch => Icon::Spell(Spell::Launch),
            Skill::Glide => Icon::Spell(Spell::Glide),
            Skill::WaterBreath => Icon::Opher(OpherIcon::WaterBreath),
            Skill::Grenade => Icon::Spell(Spell::Grenade),
            Skill::Grapple => Icon::Spell(Spell::Grapple),
            Skill::Flash => Icon::Spell(Spell::Glow),
            Skill::Spear => Icon::Opher(OpherIcon::Spear),
            Skill::Regenerate => Icon::Spell(Spell::Regenerate),
            Skill::Bow => Icon::Spell(Spell::Bow),
            Skill::Hammer => Icon::Opher(OpherIcon::Hammer),
            Skill::Sword => Icon::Spell(Spell::Sword),
            Skill::Burrow => Icon::Spell(Spell::Burrow),
            Skill::Dash => Icon::Spell(Spell::Dash),
            Skill::WaterDash => Icon::Spell(Spell::WaterDash),
            Skill::Shuriken => Icon::Opher(OpherIcon::Shuriken),
            Skill::Seir => Icon::Spell(Spell::Sein),
            Skill::Blaze => Icon::Opher(OpherIcon::Blaze),
            Skill::Sentry => Icon::Opher(OpherIcon::Sentry),
            Skill::Flap => Icon::Spell(Spell::Flap),
            Skill::AncestralLight1 => Icon::File(String::from("assets/icons/game/ancestrallight1.png")),
            Skill::AncestralLight2 => Icon::File(String::from("assets/icons/game/ancestrallight2.png")),
        };
        Some(icon)
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
