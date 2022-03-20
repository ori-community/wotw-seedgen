use std::{str::FromStr, convert::TryFrom};

use decorum::R32;

use crate::{
    VItem,
    item::{Resource, Skill, Shard, VCommand, Teleporter, BonusItem, BonusUpgrade, ToggleCommand, SysMessage, VWheelCommand, WheelBind, VShopCommand, VUberStateItem, VUberStateOperator, VUberStateRange, VUberStateRangeBoundary},
    util::{Zone, Icon, VUberState, UberType, UberIdentifier, VPosition, NumericBool},
    header::{V, VString},
};

use super::{parse_uber_identifier, parse_uber_state};

fn parse_v_uber_state_condition<'a, P>(parts: &mut P) -> Result<VUberState, String>
where P: Iterator<Item=&'a str>
{
    let uber_group = parts.next().ok_or_else(|| String::from("missing uber group"))?;
    let uber_id = parts.next().ok_or_else(|| String::from("missing uber id"))?;
    let identifier = UberIdentifier::from_parts(uber_group, uber_id)?;

    let value = parts.next().ok_or_else(|| String::from("missing uber value"))?;
    let value = V::wrap(value);

    Ok(VUberState { identifier, value })
}
fn parse_v_position<'a, P>(parts: &mut P) -> Result<VPosition, String>
where P: Iterator<Item=&'a str>
{
    let x = parts.next().ok_or_else(|| String::from("missing x coordinate"))?;
    let x = V::try_wrap(x).map_err(|_| format!("invalid x coordinate {x}"))?;
    let y = parts.next().ok_or_else(|| String::from("missing y coordinate"))?;
    let y = V::try_wrap(y).map_err(|_| format!("invalid y coordinate {y}"))?;

    Ok(VPosition { x, y })
}

fn end_of_item<'a, I>(mut parts: I) -> Result<(), String>
where
    I: Iterator<Item = &'a str>,
{
    if parts.next().is_some() { return Err(String::from("too many parts")); }
    Ok(())
}

fn parse_spirit_light<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let spirit_light = parts.next().ok_or_else(|| String::from("missing spirit light amount"))?;
    end_of_item(parts)?;
    if let Some(spirit_light) = spirit_light.strip_prefix('-') {
        let spirit_light = V::try_wrap(spirit_light).map_err(|_| format!("invalid spirit light amount {spirit_light}"))?;
        Ok(VItem::RemoveSpiritLight(spirit_light))
    } else {
        let spirit_light = V::try_wrap(spirit_light).map_err(|_| format!("invalid spirit light amount {spirit_light}"))?;
        Ok(VItem::SpiritLight(spirit_light))
    }
}
fn parse_resource<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let resource_type = parts.next().ok_or_else(|| String::from("missing resource type"))?;
    end_of_item(parts)?;
    let resource_type: u8 = resource_type.parse().map_err(|_| format!("invalid resource type {resource_type}"))?;
    let resource = Resource::try_from(resource_type).map_err(|_| format!("invalid resource type {resource_type}"))?;
    Ok(VItem::Resource(resource))
}
fn parse_skill<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let skill_type = parts.next().ok_or_else(|| String::from("missing skill type"))?;
    end_of_item(parts)?;
    if let Some(skill_type) = skill_type.strip_prefix('-') {
        let skill_type: u8 = skill_type.parse().map_err(|_| format!("invalid skill type {skill_type}"))?;
        let skill = Skill::try_from(skill_type).map_err(|_| format!("invalid skill type {skill_type}"))?;
        Ok(VItem::RemoveSkill(skill))
    } else {
        let skill_type: u8 = skill_type.parse().map_err(|_| format!("invalid skill type {skill_type}"))?;
        let skill = Skill::try_from(skill_type).map_err(|_| format!("invalid skill type {skill_type}"))?;
        Ok(VItem::Skill(skill))
    }
}
fn parse_shard<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let shard_type = parts.next().ok_or_else(|| String::from("missing shard type"))?;
    end_of_item(parts)?;
    if let Some(shard_type) = shard_type.strip_prefix('-') {
        let shard_type: u8 = shard_type.parse().map_err(|_| format!("invalid shard type {shard_type}"))?;
        let shard = Shard::try_from(shard_type).map_err(|_| format!("invalid shard type {shard_type}"))?;
        Ok(VItem::RemoveShard(shard))
    } else {
        let shard_type: u8 = shard_type.parse().map_err(|_| format!("invalid shard type {shard_type}"))?;
        let shard = Shard::try_from(shard_type).map_err(|_| format!("invalid shard type {shard_type}"))?;
        Ok(VItem::Shard(shard))
    }
}
fn parse_autosave<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    end_of_item(parts)?;
    Ok(VItem::Command(VCommand::Autosave))
}
fn parse_set_resource<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let resource = parts.next().ok_or_else(|| String::from("missing resource type"))?;
    let resource: u8 = resource.parse().map_err(|_| format!("invalid resource type {resource}"))?;
    let resource = Resource::try_from(resource).map_err(|_| format!("invalid resource type {resource}"))?;
    let amount = parts.next().ok_or_else(|| String::from("missing resource amount"))?;
    let amount = V::try_wrap(amount).map_err(|_| format!("invalid resource amount {amount}"))?;
    end_of_item(parts)?;
    Ok(VItem::Command(VCommand::Resource { resource, amount }))
}
fn parse_checkpoint<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    end_of_item(parts)?;
    Ok(VItem::Command(VCommand::Checkpoint))
}
fn parse_magic<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    end_of_item(parts)?;
    Ok(VItem::Command(VCommand::Magic))
}
fn parse_stop<'a, P>(mut parts: P) -> Result<VUberState, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_v_uber_state_condition(&mut parts)?;
    end_of_item(parts)?;

    Ok(uber_state)
}
fn parse_stop_equal<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_stop(parts)?;
    Ok(VItem::Command(VCommand::StopEqual { uber_state }))
}
fn parse_stop_greater<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_stop(parts)?;
    Ok(VItem::Command(VCommand::StopGreater { uber_state }))
}
fn parse_stop_less<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_stop(parts)?;
    Ok(VItem::Command(VCommand::StopLess { uber_state }))
}
fn parse_toggle<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let toggle_type = parts.next().ok_or_else(|| String::from("missing toggle command type"))?;
    let toggle_type: u8 = toggle_type.parse().map_err(|_| format!("invalid toggle command type {toggle_type}"))?;
    let toggle_type = ToggleCommand::try_from(toggle_type).map_err(|_| format!("invalid toggle command type {toggle_type}"))?;
    let on = parts.next().ok_or_else(|| String::from("missing toggle command value"))?;
    let on = V::<NumericBool>::try_wrap(on).map_err(|_| format!("invalid toggle command value {on}"))?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::Toggle { target: toggle_type, on }))
}
fn parse_warp<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let position = parse_v_position(&mut parts)?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::Warp { position }))
}
fn parse_timer<'a, P>(mut parts: P) -> Result<UberIdentifier, String>
where P: Iterator<Item=&'a str>
{
    let uber_identifier = parse_uber_identifier(&mut parts)?;
    end_of_item(parts)?;

    Ok(uber_identifier)
}
fn parse_start_timer<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let identifier = parse_timer(parts)?;
    Ok(VItem::Command(VCommand::StartTimer { identifier }))
}
fn parse_stop_timer<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let identifier = parse_timer(parts)?;
    Ok(VItem::Command(VCommand::StopTimer { identifier }))
}
fn parse_intercept<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let intercept = parts.next().ok_or_else(|| String::from("missing intercept"))?;
    let intercept = intercept.parse().map_err(|_| format!("invalid intercept {intercept}"))?;
    let set = parts.next().ok_or_else(|| String::from("missing set"))?;
    let set = set.parse().map_err(|_| format!("invalid set {set}"))?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::StateRedirect { intercept, set }))
}
fn parse_set_player<'a, P>(mut parts: P) -> Result<V<i16>, String>
where P: Iterator<Item=&'a str>
{
    let amount = parts.next().ok_or_else(|| String::from("missing amount"))?;
    let amount = V::try_wrap(amount).map_err(|_| format!("invalid amount {amount}"))?;
    end_of_item(parts)?;

    Ok(amount)
}
fn parse_set_health<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let amount = parse_set_player(parts)?;
    Ok(VItem::Command(VCommand::SetHealth { amount }))
}
fn parse_set_energy<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let amount = parse_set_player(parts)?;
    Ok(VItem::Command(VCommand::SetEnergy { amount }))
}
fn parse_set_spirit_light<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let amount = parse_set_player(parts)?;
    Ok(VItem::Command(VCommand::SetSpiritLight { amount }))
}
fn parse_equip<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let slot = parts.next().ok_or_else(|| String::from("missing equip slot"))?;
    let slot = V::try_wrap(slot).map_err(|_| format!("invalid equip slot {slot}"))?;
    let ability = parts.next().ok_or_else(|| String::from("missing ability to equip"))?;
    let ability: u16 = ability.parse().map_err(|_| format!("invalid ability to equip {ability}"))?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::Equip { slot, ability }))
}
fn parse_ahk_signal<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let signal = parts.next().ok_or_else(|| String::from("missing ahk signal specifier"))?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::AhkSignal { signal: signal.to_string() }))
}
fn parse_if<'a, P>(mut parts: P) -> Result<(VUberState, Box<VItem>), String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_v_uber_state_condition(&mut parts)?;

    let item = Box::new(parse_item_parts(parts)?);

    Ok((uber_state, item))
}
fn parse_if_equal<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (uber_state, item) = parse_if(parts)?;
    Ok(VItem::Command(VCommand::IfEqual { uber_state, item }))
}
fn parse_if_greater<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (uber_state, item) = parse_if(parts)?;
    Ok(VItem::Command(VCommand::IfGreater { uber_state, item }))
}
fn parse_if_less<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (uber_state, item) = parse_if(parts)?;
    Ok(VItem::Command(VCommand::IfLess { uber_state, item }))
}
fn parse_disable_sync<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_uber_state(&mut parts)?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::DisableSync { uber_state }))
}
fn parse_enable_sync<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_uber_state(&mut parts)?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::DisableSync { uber_state }))
}
fn parse_create_warp<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let id = parts.next().ok_or_else(|| String::from("missing warp id"))?;
    let id = id.parse().map_err(|_| format!("invalid warp id {id}"))?;
    let position = parse_v_position(&mut parts)?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::CreateWarp { id, position }))
}
fn parse_destroy_warp<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let id = parts.next().ok_or_else(|| String::from("missing warp id"))?;
    let id = id.parse().map_err(|_| format!("invalid warp id {id}"))?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::DestroyWarp { id }))
}
fn parse_if_box<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let position1 = parse_v_position(&mut parts)?;
    let position2 = parse_v_position(&mut parts)?;

    let item = Box::new(parse_item_parts(parts)?);

    Ok(VItem::Command(VCommand::IfBox { position1, position2, item }))
}
fn parse_if_self<'a, P>(mut parts: P) -> Result<(V<String>, Box<VItem>), String>
where P: Iterator<Item=&'a str>
{
    let value = parts.next().ok_or_else(|| String::from("missing uber value"))?;
    let value = V::wrap(value);
    let item = Box::new(parse_item_parts(parts)?);

    Ok((value, item))
}
fn parse_if_self_equal<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (value, item) = parse_if_self(parts)?;
    Ok(VItem::Command(VCommand::IfSelfEqual { value, item }))
}
fn parse_if_self_greater<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (value, item) = parse_if_self(parts)?;
    Ok(VItem::Command(VCommand::IfSelfGreater { value, item }))
}
fn parse_if_self_less<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (value, item) = parse_if_self(parts)?;
    Ok(VItem::Command(VCommand::IfSelfLess { value, item }))
}
fn parse_unequip<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let ability = parts.next().ok_or_else(|| String::from("missing ability to unequip"))?;
    let ability: u16 = ability.parse().map_err(|_| format!("invalid ability to unequip {ability}"))?;
    end_of_item(parts)?;

    Ok(VItem::Command(VCommand::UnEquip { ability }))
}
fn parse_command<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let command_type = parts.next().ok_or_else(|| String::from("missing command item type"))?;
    match command_type {
        "0" => parse_autosave(parts),
        "1" => parse_set_resource(parts),
        "2" => parse_checkpoint(parts),
        "3" => parse_magic(parts),
        "4" => parse_stop_equal(parts),
        "5" => parse_stop_greater(parts),
        "6" => parse_stop_less(parts),
        "7" => parse_toggle(parts),
        "8" => parse_warp(parts),
        "9" => parse_start_timer(parts),
        "10" => parse_stop_timer(parts),
        "11" => parse_intercept(parts),
        "12" => parse_set_health(parts),
        "13" => parse_set_energy(parts),
        "14" => parse_set_spirit_light(parts),
        "15" => parse_equip(parts),
        "16" => parse_ahk_signal(parts),
        "17" => parse_if_equal(parts),
        "18" => parse_if_greater(parts),
        "19" => parse_if_less(parts),
        "20" => parse_disable_sync(parts),
        "21" => parse_enable_sync(parts),
        "22" => parse_create_warp(parts),
        "23" => parse_destroy_warp(parts),
        "24" => parse_if_box(parts),
        "25" => parse_if_self_equal(parts),
        "26" => parse_if_self_greater(parts),
        "27" => parse_if_self_less(parts),
        "28" => parse_unequip(parts),
        _ => Err(format!("invalid command type {command_type}")),
    }
}
fn parse_teleporter<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let teleporter_type = parts.next().ok_or_else(|| String::from("missing teleporter type"))?;
    end_of_item(parts)?;
    if let Some(teleporter_type) = teleporter_type.strip_prefix('-') {
        let teleporter_type: u8 = teleporter_type.parse().map_err(|_| format!("invalid teleporter type {teleporter_type}"))?;
        let teleporter = Teleporter::try_from(teleporter_type).map_err(|_| format!("invalid teleporter type {teleporter_type}"))?;
        Ok(VItem::RemoveTeleporter(teleporter))
    } else {
        let teleporter_type: u8 = teleporter_type.parse().map_err(|_| format!("invalid teleporter type {teleporter_type}"))?;
        let teleporter = Teleporter::try_from(teleporter_type).map_err(|_| format!("invalid teleporter type {teleporter_type}"))?;
        Ok(VItem::Teleporter(teleporter))
    }
}
fn parse_message<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let parts = parts.collect::<Vec<&str>>();
    if parts.is_empty() {
        return Err(String::from("missing message"));
    }

    let message = VString(parts.join("|"));
    Ok(VItem::Message(message))
}
fn parse_pointer(str: &str) -> Option<Result<UberIdentifier, String>> {
    if let Some(str) = str.strip_prefix("$(") {
        if let Some(pointer) = str.strip_suffix(')') {
            let mut parts = pointer.splitn(2, '|');
            let uber_group = parts.next().unwrap();
            if let Some(uber_id) = parts.next() {
                return Some(UberIdentifier::from_parts(uber_group, uber_id));
            } else {
                return Some(Err(format!("Invalid uber identifier in pointer {pointer}")));
            }
        } else {
            return Some(Err(String::from("unmatched brackets")))
        }
    }

    None
}
fn parse_set_uber_state<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_group = parts.next().ok_or_else(|| String::from("missing uber group"))?;
    let uber_id = parts.next().ok_or_else(|| String::from("missing uber id"))?;
    let uber_identifier = UberIdentifier::from_parts(uber_group, uber_id)?;

    let uber_type = parts.next().ok_or_else(|| String::from("missing uber state type"))?;
    let uber_type = UberType::from_str(uber_type)?;

    let mut remaining = &parts.into_iter().collect::<Vec<_>>().join("|")[..];

    let mut signed = false;
    let mut sign = false;
    if remaining.starts_with('+') {
        signed = true;
        sign = true;
    } else if remaining.starts_with('-') {
        signed = true;
    }
    if signed {
        if matches!(uber_type, UberType::Bool) { return Err(String::from("can't math with bools")); }
        remaining = &remaining[1..];
    }

    let mut skip = false;
    if let Some(last) = remaining.rfind('|') {
        let mut last_part = &remaining[last + 1..];
        if let Some(skip) = last_part.strip_prefix("skip=") {
            last_part = skip;
        }
        if let Ok(skip_amount) = last_part.parse::<u8>() {
            if skip_amount > 0 {
                if skip_amount > 1 {
                    log::warn!("An UberState pickup is skipping the next {last_part} triggers, note that this will not be correctly simulated during seed generation.");
                }
                skip = true;
            }
            remaining = &remaining[..last];
        }
    }

    let parse_by_value = |value: &str| -> Result<(), String> {
        match uber_type {
            UberType::Bool | UberType::Teleporter => { value.parse::<bool>().map_err(|_| format!("invalid value {value} as boolean"))?; },
            UberType::Byte => { value.parse::<u8>().map_err(|_| format!("invalid value {value} as byte"))?; },
            UberType::Int => { value.parse::<i32>().map_err(|_| format!("invalid value {value} as integer"))?; },
            UberType::Float => { value.parse::<R32>().map_err(|_| format!("invalid value {value} as floating point"))?; },
        }
        Ok(())
    };

    let operator = if let Some(range) = remaining.strip_prefix('[') {
        if let Some(range) = range.strip_suffix(']') {
            let mut parts = range.splitn(2, ',');
            let start = parts.next().unwrap().trim();
            let end = parts.next().ok_or("missing range end")?.trim();

            let parse_boundary = |value| -> Result<_, String> {
                if let Some(uber_identifier) = parse_pointer(value) {
                    Ok(VUberStateRangeBoundary::Pointer(uber_identifier?))
                } else {
                    parse_by_value(value)?;
                    Ok(VUberStateRangeBoundary::Value(V::wrap(value)))
                }
            };

            let start = parse_boundary(start)?;
            let end = parse_boundary(end)?;
            Ok(VUberStateOperator::Range(VUberStateRange {
                start,
                end,
            }))
        } else {
            Err(String::from("unmatched brackets"))
        }
    } else if let Some(pointer) = parse_pointer(remaining) {
        Ok(VUberStateOperator::Pointer(pointer?))
    } else {
        parse_by_value(remaining)?;
        Ok(VUberStateOperator::Value(V::wrap(remaining)))
    }?;

    Ok(VItem::UberState(VUberStateItem {
        uber_identifier,
        uber_type,
        signed,
        sign,
        operator,
        skip,
    }))
}
fn parse_world_event<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let world_event_type = parts.next().ok_or_else(|| String::from("missing world event type"))?;
    end_of_item(parts)?;
    if let Some(world_event_type) = world_event_type.strip_prefix('-') {
        let world_event_type: u8 = world_event_type.parse().map_err(|_| format!("invalid world event type {world_event_type}"))?;
        if world_event_type != 0 { return Err(format!("invalid world event type {world_event_type}")); }
        Ok(VItem::RemoveWater)
    } else {
        let world_event_type: u8 = world_event_type.parse().map_err(|_| format!("invalid world event type {world_event_type}"))?;
        if world_event_type != 0 { return Err(format!("invalid world event type {world_event_type}")); }
        Ok(VItem::Water)
    }
}
fn parse_bonus_item<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let bonus_type = parts.next().ok_or_else(|| String::from("missing bonus item type"))?;
    end_of_item(parts)?;
    let bonus_type: u8 = bonus_type.parse().map_err(|_| format!("invalid bonus item type {bonus_type}"))?;
    let bonus = BonusItem::try_from(bonus_type).map_err(|_| format!("invalid bonus item type {bonus_type}"))?;
    Ok(VItem::BonusItem(bonus))
}
fn parse_bonus_upgrade<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let bonus_type = parts.next().ok_or_else(|| String::from("missing bonus upgrade type"))?;
    end_of_item(parts)?;
    let bonus_type: u8 = bonus_type.parse().map_err(|_| format!("invalid bonus upgrade type {bonus_type}"))?;
    let bonus = BonusUpgrade::try_from(bonus_type).map_err(|_| format!("invalid bonus upgrade type {bonus_type}"))?;
    Ok(VItem::BonusUpgrade(bonus))
}
fn parse_zone_hint() -> Result<VItem, String> {
    Err(String::from("Hint Items are deprecated"))
}
fn parse_checkable_hint() -> Result<VItem, String> {
    Err(String::from("Hint Items are deprecated"))
}
fn parse_relic<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let zone = parts.next().ok_or_else(|| String::from("missing relic zone"))?;
    end_of_item(parts)?;

    let zone: u8 = zone.parse().map_err(|_| format!("invalid relic zone {zone}"))?;
    let zone = Zone::try_from(zone).map_err(|_| format!("invalid relic zone {zone}"))?;

    Ok(VItem::Relic(zone))
}
fn parse_sysmessage<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let message = parts.next().ok_or_else(|| String::from("missing sysmessage type"))?;
    end_of_item(parts)?;

    let message: u8 = message.parse().map_err(|_| format!("invalid sysmessage type {message}"))?;
    let message = SysMessage::from_id(message).ok_or_else(|| format!("invalid sysmessage type {message}"))?;

    Ok(VItem::SysMessage(message))
}
fn parse_wheel_item_position<'a, P>(parts: &mut P) -> Result<(u16, u8), String>
where P: Iterator<Item=&'a str>
{
    let wheel = parts.next().ok_or_else(|| String::from("missing wheel id"))?;
    let wheel = wheel.parse().map_err(|_| format!("invalid wheel id {wheel}"))?;
    let position = parts.next().ok_or_else(|| String::from("missing wheel item position"))?;
    let position = position.parse().map_err(|_| format!("invalid wheel item position {position}"))?;

    Ok((wheel, position))
}
fn parse_wheel_set_name<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (wheel, position) = parse_wheel_item_position(&mut parts)?;

    let parts = parts.collect::<Vec<&str>>();
    if parts.is_empty() {
        return Err(String::from("missing name"));
    }
    let name = VString(parts.join("|"));

    Ok(VItem::WheelCommand(VWheelCommand::SetName { wheel, position, name }))
}
fn parse_wheel_set_description<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (wheel, position) = parse_wheel_item_position(&mut parts)?;

    let parts = parts.collect::<Vec<&str>>();
    if parts.is_empty() {
        return Err(String::from("missing description"));
    }
    let description = VString(parts.join("|"));

    Ok(VItem::WheelCommand(VWheelCommand::SetDescription { wheel, position, description }))
}
fn parse_wheel_set_icon<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (wheel, position) = parse_wheel_item_position(&mut parts)?;
    let icon = parts.next().ok_or_else(|| String::from("missing icon"))?;
    let icon = Icon::parse(icon)?;
    end_of_item(parts)?;

    Ok(VItem::WheelCommand(VWheelCommand::SetIcon { wheel, position, icon }))
}
fn parse_wheel_set_color<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (wheel, position) = parse_wheel_item_position(&mut parts)?;
    let r = parts.next().ok_or_else(|| String::from("missing red channel"))?;
    let r = V::try_wrap(r).map_err(|_| format!("invalid red channel {r}"))?;
    let g = parts.next().ok_or_else(|| String::from("missing green channel"))?;
    let g = V::try_wrap(g).map_err(|_| format!("invalid green channel {g}"))?;
    let b = parts.next().ok_or_else(|| String::from("missing blue channel"))?;
    let b = V::try_wrap(b).map_err(|_| format!("invalid blue channel {b}"))?;
    let a = parts.next().ok_or_else(|| String::from("missing alpha channel"))?;
    let a = V::try_wrap(a).map_err(|_| format!("invalid alpha channel {a}"))?;
    end_of_item(parts)?;

    Ok(VItem::WheelCommand(VWheelCommand::SetColor { wheel, position, r, g, b, a }))

}
fn parse_wheel_set_item<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (wheel, position) = parse_wheel_item_position(&mut parts)?;
    let bind = parts.next().ok_or_else(|| String::from("missing bind"))?;
    let bind = match bind {
        "0" => WheelBind::All,
        "1" => WheelBind::Ability1,
        "2" => WheelBind::Ability2,
        "3" => WheelBind::Ability3,
        _ => return Err(format!("invalid bind {bind}")),
    };

    let item = Box::new(parse_item_parts(parts)?);

    Ok(VItem::WheelCommand(VWheelCommand::SetItem { wheel, position, bind, item }))
}
fn parse_wheel_set_sticky<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let wheel = parts.next().ok_or_else(|| String::from("missing wheel id"))?;
    let wheel = wheel.parse().map_err(|_| format!("invalid wheel id {wheel}"))?;
    let sticky = parts.next().ok_or_else(|| String::from("missing sticky boolean"))?;
    let sticky = V::try_wrap(sticky).map_err(|_| format!("invalid sticky boolean {sticky}"))?;
    end_of_item(parts)?;

    Ok(VItem::WheelCommand(VWheelCommand::SetSticky { wheel, sticky }))
}
fn parse_wheel_switch_wheel<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let wheel = parts.next().ok_or_else(|| String::from("missing wheel id"))?;
    let wheel = wheel.parse().map_err(|_| format!("invalid wheel id {wheel}"))?;
    end_of_item(parts)?;

    Ok(VItem::WheelCommand(VWheelCommand::SwitchWheel { wheel }))
}
fn parse_wheel_remove_item<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let (wheel, position) = parse_wheel_item_position(&mut parts)?;
    end_of_item(parts)?;

    Ok(VItem::WheelCommand(VWheelCommand::RemoveItem { wheel, position }))
}
fn parse_wheel_clear_all<'a, P>(parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    end_of_item(parts)?;

    Ok(VItem::WheelCommand(VWheelCommand::ClearAll))
}
fn parse_wheelcommand<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let command_type = parts.next().ok_or_else(|| String::from("missing wheel command type"))?;
    match command_type {
        "0" => parse_wheel_set_name(parts),
        "1" => parse_wheel_set_description(parts),
        "2" => parse_wheel_set_icon(parts),
        "3" => parse_wheel_set_color(parts),
        "4" => parse_wheel_set_item(parts),
        "5" => parse_wheel_set_sticky(parts),
        "6" => parse_wheel_switch_wheel(parts),
        "7" => parse_wheel_remove_item(parts),
        "8" => parse_wheel_clear_all(parts),
        _ => Err(format!("invalid wheel command type {command_type}")),
    }
}
fn parse_shop_set_icon<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_uber_identifier(&mut parts)?;

    let icon = parts.next().ok_or_else(|| String::from("missing icon"))?;
    let icon = Icon::parse(icon)?;
    end_of_item(parts)?;

    Ok(VItem::ShopCommand(VShopCommand::SetIcon { uber_state, icon }))
}
fn parse_shop_set_title<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_uber_identifier(&mut parts)?;

    let title = parts.next().map(str::to_owned).map(VString);
    end_of_item(parts)?;

    Ok(VItem::ShopCommand(VShopCommand::SetTitle { uber_state, title }))
}
fn parse_shop_set_description<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_uber_identifier(&mut parts)?;

    let description = parts.next().map(str::to_owned).map(VString);
    end_of_item(parts)?;

    Ok(VItem::ShopCommand(VShopCommand::SetDescription { uber_state, description }))
}
fn parse_shop_set_locked<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_uber_identifier(&mut parts)?;

    let locked = parts.next().ok_or_else(|| String::from("missing locked"))?;
    let locked = V::try_wrap(locked).map_err(|_| format!("Invalid value {locked} for boolean locked"))?;
    end_of_item(parts)?;

    Ok(VItem::ShopCommand(VShopCommand::SetLocked { uber_state, locked }))
}
fn parse_shop_set_visible<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let uber_state = parse_uber_identifier(&mut parts)?;

    let visible = parts.next().ok_or_else(|| String::from("missing visible"))?;
    let visible = V::try_wrap(visible).map_err(|_| format!("Invalid value {visible} for boolean visible"))?;
    end_of_item(parts)?;

    Ok(VItem::ShopCommand(VShopCommand::SetVisible { uber_state, visible }))
}
fn parse_shopcommand<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let command_type = parts.next().ok_or_else(|| String::from("missing shop command type"))?;
    match command_type {
        "0" => parse_shop_set_icon(parts),
        "1" => parse_shop_set_title(parts),
        "2" => parse_shop_set_description(parts),
        "3" => parse_shop_set_locked(parts),
        "4" => parse_shop_set_visible(parts),
        _ => Err(format!("invalid shop command type {command_type}")),
    }
}

fn parse_item_parts<'a, P>(mut parts: P) -> Result<VItem, String>
where P: Iterator<Item=&'a str>
{
    let item_type = parts.next().unwrap_or("tried to parse empty item");
    match item_type {
        "0" => parse_spirit_light(parts),
        "1" => parse_resource(parts),
        "2" => parse_skill(parts),
        "3" => parse_shard(parts),
        "4" => parse_command(parts),
        "5" => parse_teleporter(parts),
        "6" => parse_message(parts),
        "8" => parse_set_uber_state(parts),
        "9" => parse_world_event(parts),
        "10" => parse_bonus_item(parts),
        "11" => parse_bonus_upgrade(parts),
        "12" => parse_zone_hint(),
        "13" => parse_checkable_hint(),
        "14" => parse_relic(parts),
        "15" => parse_sysmessage(parts),
        "16" => parse_wheelcommand(parts),
        "17" => parse_shopcommand(parts),
        _ => Err(format!("invalid item type {item_type}")),
    }
}

impl VItem {
    /// Parse item syntax
    pub fn parse(item: &str) -> Result<VItem, String> {
        let parts = item.trim().split('|');

        parse_item_parts(parts).map_err(|err| format!("{err} in item {item}"))
    }
}
