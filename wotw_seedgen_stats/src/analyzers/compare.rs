use std::cmp::Ordering;

#[inline]
pub(super) fn order_index(s: &str, order: &[&str]) -> usize {
    order
        .iter()
        .enumerate()
        .find(|(_, zone)| s == **zone)
        .map_or(usize::MAX, |(index, _)| index)
}
pub(super) fn compare_fixed_order<T: FixedOrder>(a: &String, b: &String) -> Ordering {
    order_index(a, T::ORDER).cmp(&order_index(b, T::ORDER))
}
pub(super) fn compare_location(a: &String, b: &String) -> Ordering {
    let (a_region, a_identifier) = a.split_once('.').unwrap_or((a, ""));
    let (b_region, b_identifier) = b.split_once('.').unwrap_or((b, ""));

    match Ord::cmp(
        &order_index(a_region, RegionFixedOrder::ORDER),
        &order_index(b_region, RegionFixedOrder::ORDER),
    ) {
        Ordering::Equal => a_identifier.cmp(b_identifier),
        non_eq => non_eq,
    }
}
pub(super) trait FixedOrder {
    const ORDER: &'static [&'static str];
}
pub(super) struct RegionFixedOrder;
impl FixedOrder for RegionFixedOrder {
    const ORDER: &'static [&'static str] = &[
        "MarshSpawn",
        "MarshPastOpher",
        "HowlsDen",
        "EastHollow",
        "WestHollow",
        "GladesTown",
        "WestGlades",
        "InnerWellspring",
        "OuterWellspring",
        "WoodsEntry",
        "WoodsMain",
        "LowerReach",
        "UpperReach",
        "UpperDepths",
        "LowerDepths",
        "PoolsApproach",
        "EastPools",
        "UpperPools",
        "WestPools",
        "LowerWastes",
        "UpperWastes",
        "WindtornRuins",
        "WeepingRidge",
        "WillowsEnd",
        "MidnightBurrows",
    ];
}
pub(super) struct ZoneFixedOrder;
impl FixedOrder for ZoneFixedOrder {
    const ORDER: &'static [&'static str] = &[
        "Marsh",
        "Hollow",
        "Glades",
        "Wellspring",
        "Woods",
        "Reach",
        "Depths",
        "Pools",
        "Wastes",
        "Ruins",
        "Willow",
        "Burrows",
        "Spawn",
        "Shop",
        "Void",
    ];
}
