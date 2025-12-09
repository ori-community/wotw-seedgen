use std::{hash::BuildHasher, sync::LazyLock};

use rustc_hash::{FxBuildHasher, FxHashMap};
use wotw_seedgen::{
    assets::{LocData, LocDataEntry},
    data::{self, Position, UberIdentifier},
    seed_language::ast::Comparator,
};

use crate::api::reach_check::{MapIcon, MapIconCondition, MapIcons};

static SPIRIT_TRIAL_ICONS: LazyLock<FxHashMap<UberIdentifier, (Position, Position)>> =
    LazyLock::new(|| {
        FxHashMap::from_iter([
            // MarshPastOpher.SpiritTrial
            (
                UberIdentifier::new(44964, 45951),
                (
                    Position::new(-614., -4319.),
                    Position::new(-423.68, -4306.3604),
                ),
            ),
            // WestHollow.SpiritTrial
            (
                UberIdentifier::new(44964, 25545),
                (
                    Position::new(-115., -4259.),
                    Position::new(-175.43, -4440.89),
                ),
            ),
            // OuterWellspring.SpiritTrial
            (
                UberIdentifier::new(44964, 11512),
                (
                    Position::new(-668., -3937.),
                    Position::new(-834.55005, -3893.5503),
                ),
            ),
            // EastPools.SpiritTrial
            (
                UberIdentifier::new(44964, 54686),
                (
                    Position::new(-1417., -4126.),
                    Position::new(-1485.9731, -4059.728),
                ),
            ),
            // WoodsMain.SpiritTrial
            (
                UberIdentifier::new(44964, 22703),
                (
                    Position::new(820., -4047.),
                    Position::new(859.62, -3938.6702),
                ),
            ),
            // LowerReach.SpiritTrial
            (
                UberIdentifier::new(44964, 23661),
                (
                    Position::new(75., -4046.),
                    Position::new(101.933716, -4046.7227),
                ),
            ),
            // LowerDepths.SpiritTrial
            (
                UberIdentifier::new(44964, 28552),
                (
                    Position::new(478., -4517.),
                    Position::new(573.47345, -4510.134),
                ),
            ),
            // LowerWastes.SpiritTrial
            (
                UberIdentifier::new(44964, 30767),
                (
                    Position::new(1527., -4009.),
                    Position::new(1580.71, -3898.5503),
                ),
            ),
        ])
    });

impl MapIcons {
    pub fn new(loc_data: &LocData) -> Self {
        const SPIRIT_TRIAL_COUNT: usize = 8;
        const SHOP_ICON_COUNT: usize = 7;
        const OPHER_ITEM_COUNT: usize = 12;
        const TWILLEN_ITEM_COUNT: usize = 8;
        const LUPO_ITEM_COUNT: usize = 3;
        const SHOP_ITEM_COUNT: usize = OPHER_ITEM_COUNT + TWILLEN_ITEM_COUNT + LUPO_ITEM_COUNT;

        let map_icon_count =
            loc_data.entries.len() + SPIRIT_TRIAL_COUNT * 3 + SHOP_ICON_COUNT - SHOP_ITEM_COUNT;

        let mut map_icons = Vec::with_capacity(map_icon_count);

        let mut opher_conditions = Vec::with_capacity(OPHER_ITEM_COUNT);
        let mut twillen_conditions = Vec::with_capacity(TWILLEN_ITEM_COUNT);
        let mut lupo_conditions = Vec::with_capacity(LUPO_ITEM_COUNT);

        for entry in &loc_data.entries {
            match entry.map_icon {
                data::MapIcon::Opher => {
                    opher_conditions.push(MapIconCondition::new(entry.uber_identifier, None));
                }
                data::MapIcon::Twillen => {
                    twillen_conditions.push(MapIconCondition::new(entry.uber_identifier, None));
                }
                data::MapIcon::Lupo => {
                    lupo_conditions.push(MapIconCondition::new(entry.uber_identifier, None));
                }
                data::MapIcon::RaceStart => {
                    let (start, end) = SPIRIT_TRIAL_ICONS[&entry.uber_identifier];

                    map_icons.extend([
                        MapIcon::spirit_trial_start(entry, start),
                        MapIcon::spirit_trial_start_finished(entry, start),
                        MapIcon::spirit_trial_end(entry, end),
                        MapIcon::spirit_trial_end_finished(entry, end),
                    ]);
                }
                _ => map_icons.extend(MapIcon::from_entry(entry)),
            }
        }

        map_icons.extend([
            MapIcon {
                label: "OpherShop".to_string(),
                kind: data::MapIcon::Opher,
                positions: vec![
                    Position::new(-597.1, -4291.3),
                    Position::new(-203.9, -4146.4),
                    Position::new(-1259.7, -3675.5),
                ],
                visible_if_any: opher_conditions,
            },
            MapIcon {
                label: "TwillenShop".to_string(),
                kind: data::MapIcon::Twillen,
                positions: vec![
                    Position::new(-281.3, -4236.4),
                    Position::new(-410.5, -4158.9),
                ],
                visible_if_any: twillen_conditions,
            },
            MapIcon {
                label: "LupoShop".to_string(),
                kind: data::MapIcon::Lupo,
                positions: vec![Position::new(-212.3, -4158.8)],
                visible_if_any: lupo_conditions,
            },
            // TODO make from loc_data if that gets grom shop entries
            MapIcon {
                label: "GromShop".to_string(),
                kind: data::MapIcon::Grom,
                positions: vec![Position::new(-319.1, -4150.1)],
                visible_if_any: vec![
                    MapIconCondition::new(UberIdentifier::new(17, 15068), None),
                    MapIconCondition::new(UberIdentifier::new(17, 16586), None),
                    MapIconCondition::new(UberIdentifier::new(17, 16825), None),
                    MapIconCondition::new(UberIdentifier::new(17, 18751), None),
                    MapIconCondition::new(UberIdentifier::new(17, 23607), None),
                    MapIconCondition::new(UberIdentifier::new(17, 40448), None),
                    MapIconCondition::new(UberIdentifier::new(17, 51230), None),
                ],
            },
        ]);

        let hash = FxBuildHasher.hash_one(&map_icons);

        Self { map_icons, hash }
    }
}

impl MapIcon {
    fn from_entry(entry: &LocDataEntry) -> Option<Self> {
        entry.map_position.map(|map_position| MapIcon {
            label: entry.identifier.clone(),
            kind: entry.map_icon,
            positions: vec![map_position],
            visible_if_any: vec![MapIconCondition::new(entry.uber_identifier, entry.value)],
        })
    }

    fn spirit_trial_start(entry: &LocDataEntry, position: Position) -> Self {
        MapIcon {
            label: entry.identifier.clone(),
            kind: data::MapIcon::RaceStart,
            positions: vec![position],
            visible_if_any: vec![MapIconCondition {
                uber_identifier: entry.uber_identifier,
                comparator: Comparator::Equal,
                value: (1.).into(),
            }],
        }
    }

    fn spirit_trial_start_finished(entry: &LocDataEntry, position: Position) -> Self {
        MapIcon {
            label: entry.identifier.clone(),
            kind: data::MapIcon::RaceStartFinished,
            positions: vec![position],
            visible_if_any: vec![MapIconCondition {
                uber_identifier: entry.uber_identifier,
                comparator: Comparator::Equal,
                value: (2.).into(),
            }],
        }
    }

    fn spirit_trial_end(entry: &LocDataEntry, position: Position) -> Self {
        MapIcon {
            label: activation_identifier(&entry.identifier),
            kind: data::MapIcon::RaceEnd,
            positions: vec![position],
            visible_if_any: vec![MapIconCondition {
                uber_identifier: entry.uber_identifier,
                comparator: Comparator::LessOrEqual,
                value: (1.).into(),
            }],
        }
    }

    fn spirit_trial_end_finished(entry: &LocDataEntry, position: Position) -> Self {
        MapIcon {
            label: activation_identifier(&entry.identifier),
            kind: data::MapIcon::RaceEndFinished,
            positions: vec![position],
            visible_if_any: vec![MapIconCondition {
                uber_identifier: entry.uber_identifier,
                comparator: Comparator::Equal,
                value: (2.).into(),
            }],
        }
    }
}

fn activation_identifier(base: &str) -> String {
    format!("{}.TrialActivation", base.split('.').next().unwrap())
}

impl MapIconCondition {
    fn new(uber_identifier: UberIdentifier, value: Option<i32>) -> Self {
        Self {
            uber_identifier,
            comparator: Comparator::Less,
            value: match value {
                None => 0.5.into(),
                Some(value) => (value as f32).into(),
            },
        }
    }
}
