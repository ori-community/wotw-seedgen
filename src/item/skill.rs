use std::fmt;

use crate::util::{Difficulty, Icon, auto_display};

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Skill {
    Bash,
    WallJump,
    DoubleJump,
    Launch,
    Glide,
    WaterBreath,
    Grenade,
    Grapple,
    Flash,
    Spear,
    Regenerate,
    Bow,
    Hammer,
    Sword,
    Burrow,
    Dash,
    WaterDash,
    Shuriken,
    Seir,
    Blaze,
    Sentry,
    Flap,
    AncestralLight,
}
impl fmt::Display for Skill {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", auto_display(self))
    }
}
impl Skill {
    pub fn from_id(id: u8) -> Option<Skill> {
        match id {
            0 => Some(Skill::Bash),
            3 => Some(Skill::WallJump),
            5 => Some(Skill::DoubleJump),
            8 => Some(Skill::Launch),
            14 => Some(Skill::Glide),
            23 => Some(Skill::WaterBreath),
            51 => Some(Skill::Grenade),
            57 => Some(Skill::Grapple),
            62 => Some(Skill::Flash),
            74 => Some(Skill::Spear),
            77 => Some(Skill::Regenerate),
            97 => Some(Skill::Bow),
            98 => Some(Skill::Hammer),
            100 => Some(Skill::Sword),
            101 => Some(Skill::Burrow),
            102 => Some(Skill::Dash),
            104 => Some(Skill::WaterDash),
            106 => Some(Skill::Shuriken),
            108 => Some(Skill::Seir),
            115 => Some(Skill::Blaze),
            116 => Some(Skill::Sentry),
            118 => Some(Skill::Flap),
            120 | 121 => Some(Skill::AncestralLight),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            Skill::Bash => 0,
            Skill::WallJump => 3,
            Skill::DoubleJump => 5,
            Skill::Launch => 8,
            Skill::Glide => 14,
            Skill::WaterBreath => 23,
            Skill::Grenade => 51,
            Skill::Grapple => 57,
            Skill::Flash => 62,
            Skill::Spear => 74,
            Skill::Regenerate => 77,
            Skill::Bow => 97,
            Skill::Hammer => 98,
            Skill::Sword => 100,
            Skill::Burrow => 101,
            Skill::Dash => 102,
            Skill::WaterDash => 104,
            Skill::Shuriken => 106,
            Skill::Seir => 108,
            Skill::Blaze => 115,
            Skill::Sentry => 116,
            Skill::Flap => 118,
            Skill::AncestralLight => 120,
        }
    }

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
