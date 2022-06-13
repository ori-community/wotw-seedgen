use num_enum::TryFromPrimitive;
use wotw_seedgen_derive::FromStr;

use crate::util::{UberIdentifier, UberState};

#[derive(Debug, wotw_seedgen_derive::Display, PartialEq, Eq, PartialOrd, Ord, Hash, Clone, Copy, TryFromPrimitive, FromStr)]
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
impl Teleporter {
    pub(crate) fn triggered_state(self) -> UberState {
        let identifier = match self {
            Teleporter::Marsh => UberIdentifier { uber_group: 21786, uber_id: 10185 },
            Teleporter::Den => UberIdentifier { uber_group: 11666, uber_id: 61594 },
            Teleporter::Hollow => UberIdentifier { uber_group: 937, uber_id: 26601 },
            Teleporter::Glades => UberIdentifier { uber_group: 42178, uber_id: 42096 },
            Teleporter::Wellspring => UberIdentifier { uber_group: 53632, uber_id: 18181 },
            Teleporter::Burrows => UberIdentifier { uber_group: 24922, uber_id: 42531 },
            Teleporter::WestWoods => UberIdentifier { uber_group: 58674, uber_id: 7071 },
            Teleporter::EastWoods => UberIdentifier { uber_group: 58674, uber_id: 1965 },
            Teleporter::Reach => UberIdentifier { uber_group: 28895, uber_id: 54235 },
            Teleporter::Depths => UberIdentifier { uber_group: 18793, uber_id: 38871 },
            Teleporter::EastLuma => UberIdentifier { uber_group: 945, uber_id: 58183 },
            Teleporter::WestLuma => UberIdentifier { uber_group: 945, uber_id: 1370 },
            Teleporter::WestWastes => UberIdentifier { uber_group: 58674, uber_id: 10029 },
            Teleporter::EastWastes => UberIdentifier { uber_group: 20120, uber_id: 49994 },
            Teleporter::OuterRuins => UberIdentifier { uber_group: 20120, uber_id: 41398 },
            Teleporter::InnerRuins => UberIdentifier { uber_group: 10289, uber_id: 4928 },
            Teleporter::Willow => UberIdentifier { uber_group: 16155, uber_id: 41465 },
            Teleporter::Shriek => UberIdentifier { uber_group: 16155, uber_id: 50867 },
        };
        UberState {
            identifier,
            value: String::new(),
        }
    }
}
