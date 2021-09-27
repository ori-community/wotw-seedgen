use std::fmt;

use crate::util::auto_display;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum Teleporter {
    Marsh,
    Den,
    Hollow,
    Glades,
    Wellspring,
    Burrows,
    WestWoods,
    EastWoods,
    Reach,
    Depths,
    EastLuma,
    WestLuma,
    WestWastes,
    EastWastes,
    OuterRuins,
    InnerRuins,
    Willow,
    Shriek,
}
impl fmt::Display for Teleporter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} TP", auto_display(self))
    }
}
impl Teleporter {
    pub fn from_id(id: u8) -> Option<Teleporter> {
        match id {
            0 => Some(Teleporter::Burrows),
            1 => Some(Teleporter::Den),
            2 => Some(Teleporter::EastLuma),
            3 => Some(Teleporter::Wellspring),
            4 => Some(Teleporter::Reach),
            5 => Some(Teleporter::Hollow),
            6 => Some(Teleporter::Depths),
            7 => Some(Teleporter::WestWoods),
            8 => Some(Teleporter::EastWoods),
            9 => Some(Teleporter::WestWastes),
            10 => Some(Teleporter::EastWastes),
            11 => Some(Teleporter::OuterRuins),
            12 => Some(Teleporter::Willow),
            13 => Some(Teleporter::WestLuma),
            14 => Some(Teleporter::InnerRuins),
            15 => Some(Teleporter::Shriek),
            16 => Some(Teleporter::Marsh),
            17 => Some(Teleporter::Glades),
            _ => None,
        }
    }
    pub fn to_id(self) -> u16 {
        match self {
            Teleporter::Burrows => 0,
            Teleporter::Den => 1,
            Teleporter::EastLuma => 2,
            Teleporter::Wellspring => 3,
            Teleporter::Reach => 4,
            Teleporter::Hollow => 5,
            Teleporter::Depths => 6,
            Teleporter::WestWoods => 7,
            Teleporter::EastWoods => 8,
            Teleporter::WestWastes => 9,
            Teleporter::EastWastes => 10,
            Teleporter::OuterRuins => 11,
            Teleporter::Willow => 12,
            Teleporter::WestLuma => 13,
            Teleporter::InnerRuins => 14,
            Teleporter::Shriek => 15,
            Teleporter::Marsh => 16,
            Teleporter::Glades => 17,
        }
    }
}
