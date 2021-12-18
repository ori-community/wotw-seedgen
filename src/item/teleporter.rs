use std::fmt;

use num_enum::TryFromPrimitive;

use crate::util::auto_display;

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy, TryFromPrimitive)]
#[repr(u8)]
pub enum Teleporter {
    Marsh = 16,
    Den = 1,
    Hollow = 5,
    Glades = 17,
    Wellspring = 3,
    Burrows = 0,
    WestWoods = 7,
    EastWoods = 8,
    Reach = 4,
    Depths = 6,
    EastLuma = 2,
    WestLuma = 13,
    WestWastes = 9,
    EastWastes = 10,
    OuterRuins = 11,
    InnerRuins = 14,
    Willow = 12,
    Shriek = 15,
}
impl fmt::Display for Teleporter {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} TP", auto_display(self))
    }
}
