use num_enum::TryFromPrimitive;
use wotw_seedgen_derive::FromStr;

use crate::uber_state::UberIdentifier;

#[derive(
    Debug,
    wotw_seedgen_derive::Display,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    Clone,
    Copy,
    TryFromPrimitive,
    FromStr,
)]
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
    FeedingGrounds = 9,
    EastWastes = 10,
    OuterRuins = 11,
    InnerRuins = 14,
    Willow = 12,
    Shriek = 15,
}
impl Teleporter {
    pub(crate) fn attached_state(self) -> UberIdentifier {
        match self {
            Teleporter::Marsh => UberIdentifier::new(21786, 10185),
            Teleporter::Den => UberIdentifier::new(11666, 61594),
            Teleporter::Hollow => UberIdentifier::new(937, 26601),
            Teleporter::Glades => UberIdentifier::new(42178, 42096),
            Teleporter::Wellspring => UberIdentifier::new(53632, 18181),
            Teleporter::Burrows => UberIdentifier::new(24922, 42531),
            Teleporter::WestWoods => UberIdentifier::new(58674, 7071),
            Teleporter::EastWoods => UberIdentifier::new(58674, 1965),
            Teleporter::Reach => UberIdentifier::new(28895, 54235),
            Teleporter::Depths => UberIdentifier::new(18793, 38871),
            Teleporter::EastLuma => UberIdentifier::new(945, 58183),
            Teleporter::WestLuma => UberIdentifier::new(945, 1370),
            Teleporter::FeedingGrounds => UberIdentifier::new(58674, 10029),
            Teleporter::EastWastes => UberIdentifier::new(20120, 49994),
            Teleporter::OuterRuins => UberIdentifier::new(20120, 41398),
            Teleporter::InnerRuins => UberIdentifier::new(10289, 4928),
            Teleporter::Willow => UberIdentifier::new(16155, 41465),
            Teleporter::Shriek => UberIdentifier::new(16155, 50867),
        }
    }
}
