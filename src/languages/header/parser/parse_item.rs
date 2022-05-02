use num_enum::TryFromPrimitive;
use seedgen_derive::FromStr;

use crate::{
    VItem,
    item::{VCommand, SysMessage, VWheelCommand, VShopCommand, VUberStateItem, VUberStateOperator, VUberStateRange, VUberStateRangeBoundary, VMessage, WheelItemPosition},
    util::{VUberState, UberType, UberIdentifier, VPosition},
    header::{V, VString, tokenizer::TokenKind},
};

use super::{Parser, ParseError, parse_string, parse_number, parse_removable_number, parse_value, parse_ident, parse_v_ident, parse_v_number, parse_v_removable_number, parse_uber_identifier, parse_icon};

impl VItem {
    /// Parse item syntax
    pub(crate) fn parse(parser: &mut Parser) -> Result<VItem, ParseError> {
        parse_item(parser)
    }
}

#[derive(TryFromPrimitive, FromStr)]
#[repr(u8)]
enum ItemKind {
    SpiritLight = 0,
    Resource = 1,
    Skill = 2,
    Shard = 3,
    Command = 4,
    Teleporter = 5,
    Message = 6,
    UberState = 8,
    Water = 9,
    BonusItem = 10,
    BonusUpgrade = 11,
    Relic = 14,
    SysMessage = 15,
    WheelCommand = 16,
    ShopCommand = 17,
}
#[derive(TryFromPrimitive, FromStr)]
#[repr(u8)]
enum CommandKind {
    Autosave = 0,
    Resource = 1,
    Checkpoint = 2,
    Magic = 3,
    StopEqual = 4,
    StopGreater = 5,
    StopLess = 6,
    Toggle = 7,
    Warp = 8,
    StartTimer = 9,
    StopTimer = 10,
    StateRedirect = 11,
    SetHealth = 12,
    SetEnergy = 13,
    SetSpiritLight = 14,
    Equip = 15,
    AhkSignal = 16,
    IfEqual = 17,
    IfGreater = 18,
    IfLess = 19,
    DisableSync = 20,
    EnableSync = 21,
    CreateWarp = 22,
    DestroyWarp = 23,
    IfBox = 24,
    IfSelfEqual = 25,
    IfSelfGreater = 26,
    IfSelfLess = 27,
    UnEquip = 28,
    SaveString = 29,
    AppendString = 30,
}
#[derive(TryFromPrimitive, FromStr)]
#[repr(u8)]
enum WorldEventKind {
    Water = 0,
}
#[derive(TryFromPrimitive, FromStr)]
#[repr(u8)]
enum SysMessageKind {
    RelicList = 0,
    MapRelicList = 1,
    PickupCount = 2,
    GoalProgress = 3,
}
#[derive(TryFromPrimitive, FromStr)]
#[repr(u8)]
enum WheelCommandKind {
    SetName = 0,
    SetDescription = 1,
    SetIcon = 2,
    SetColor = 3,
    SetItem = 4,
    SetSticky = 5,
    SwitchWheel = 6,
    RemoveItem = 7,
    ClearAll = 8,
}
#[derive(TryFromPrimitive, FromStr)]
#[repr(u8)]
enum ShopCommandKind {
    SetIcon = 0,
    SetTitle = 1,
    SetDescription = 2,
    SetLocked = 3,
    SetVisible = 4,
}

fn parse_v_uber_state_condition(parser: &mut Parser) -> Result<VUberState, ParseError> {
    let identifier = parse_uber_identifier(parser)?;
    parser.eat(TokenKind::Separator)?;
    let value = parse_v_number!(parser, "");
    Ok(VUberState { identifier, value })
}

fn parse_v_position(parser: &mut Parser) -> Result<VPosition, ParseError> {
    let x = parse_v_number!(parser, "x coordinate");
    parser.eat(TokenKind::Separator)?;
    let y = parse_v_number!(parser, "y coordinate");
    Ok(VPosition { x, y })
}

fn parse_item(parser: &mut Parser) -> Result<VItem, ParseError> {
    let kind = parse_number!(parser, "item kind");
    match kind {
        ItemKind::SpiritLight => parse_spirit_light(parser),
        ItemKind::Resource => parse_resource(parser),
        ItemKind::Skill => parse_skill(parser),
        ItemKind::Shard => parse_shard(parser),
        ItemKind::Command => parse_command(parser),
        ItemKind::Teleporter => parse_teleporter(parser),
        ItemKind::Message => parse_message(parser),
        ItemKind::UberState => parse_set_uber_state(parser),
        ItemKind::Water => parse_water(parser),
        ItemKind::BonusItem => parse_bonus_item(parser),
        ItemKind::BonusUpgrade => parse_bonus_upgrade(parser),
        ItemKind::Relic => parse_relic(parser),
        ItemKind::SysMessage => parse_sys_message(parser),
        ItemKind::WheelCommand => parse_wheel_command(parser),
        ItemKind::ShopCommand => parse_shop_command(parser),
    }
}

fn parse_spirit_light(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let (amount, remove) = parse_v_removable_number!(parser, "spirit light amount");
    let item = if remove {
        VItem::RemoveSpiritLight(amount)
    } else {
        VItem::SpiritLight(amount)
    };
    Ok(item)
}
fn parse_resource(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let resource = parse_number!(parser, "resource");
    Ok(VItem::Resource(resource))
}
fn parse_skill(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let (skill, remove) = parse_removable_number!(parser, "skill");
    let item = if remove {
        VItem::RemoveSkill(skill)
    } else {
        VItem::Skill(skill)
    };
    Ok(item)
}
fn parse_shard(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let (shard, remove) = parse_removable_number!(parser, "shard");
    let item = if remove {
        VItem::RemoveShard(shard)
    } else {
        VItem::Shard(shard)
    };
    Ok(item)
}
fn parse_command(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let kind = parse_number!(parser, "command kind");
    let command = match kind {
        CommandKind::Autosave => Ok(VCommand::Autosave),
        CommandKind::Resource => parse_set_resource(parser),
        CommandKind::Checkpoint => Ok(VCommand::Checkpoint),
        CommandKind::Magic => Ok(VCommand::Magic),
        CommandKind::StopEqual => parse_stop_equal(parser),
        CommandKind::StopGreater => parse_stop_greater(parser),
        CommandKind::StopLess => parse_stop_less(parser),
        CommandKind::Toggle => parse_toggle(parser),
        CommandKind::Warp => parse_warp(parser),
        CommandKind::StartTimer => parse_start_timer(parser),
        CommandKind::StopTimer => parse_stop_timer(parser),
        CommandKind::StateRedirect => parse_intercept(parser),
        CommandKind::SetHealth => parse_set_health(parser),
        CommandKind::SetEnergy => parse_set_energy(parser),
        CommandKind::SetSpiritLight => parse_set_spirit_light(parser),
        CommandKind::Equip => parse_equip(parser),
        CommandKind::AhkSignal => parse_ahk_signal(parser),
        CommandKind::IfEqual => parse_if_equal(parser),
        CommandKind::IfGreater => parse_if_greater(parser),
        CommandKind::IfLess => parse_if_less(parser),
        CommandKind::DisableSync => parse_disable_sync(parser),
        CommandKind::EnableSync => parse_enable_sync(parser),
        CommandKind::CreateWarp => parse_create_warp(parser),
        CommandKind::DestroyWarp => parse_destroy_warp(parser),
        CommandKind::IfBox => parse_if_box(parser),
        CommandKind::IfSelfEqual => parse_if_self_equal(parser),
        CommandKind::IfSelfGreater => parse_if_self_greater(parser),
        CommandKind::IfSelfLess => parse_if_self_less(parser),
        CommandKind::UnEquip => parse_unequip(parser),
        CommandKind::SaveString => parse_save_string(parser),
        CommandKind::AppendString => parse_append_string(parser),
    }?;
    Ok(VItem::Command(command))
}
fn parse_set_resource(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let resource = parse_number!(parser, "resource");
    parser.eat(TokenKind::Separator)?;
    let amount = parse_v_number!(parser, "resource amount");
    Ok(VCommand::Resource { resource, amount })
}
fn parse_stop_equal(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_state = parse_v_uber_state_condition(parser)?;
    Ok(VCommand::StopEqual { uber_state })
}
fn parse_stop_greater(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_state = parse_v_uber_state_condition(parser)?;
    Ok(VCommand::StopGreater { uber_state })
}
fn parse_stop_less(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_state = parse_v_uber_state_condition(parser)?;
    Ok(VCommand::StopLess { uber_state })
}
fn parse_toggle(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let target = parse_number!(parser, "toggle command kind");
    parser.eat(TokenKind::Separator)?;
    let on = parse_v_number!(parser, "toggle command value");
    Ok(VCommand::Toggle { target, on })
}
fn parse_warp(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let position = parse_v_position(parser)?;
    Ok(VCommand::Warp { position })
}
fn parse_start_timer(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let identifier = parse_uber_identifier(parser)?;
    Ok(VCommand::StartTimer { identifier })
}
fn parse_stop_timer(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let identifier = parse_uber_identifier(parser)?;
    Ok(VCommand::StopTimer { identifier })
}
fn parse_intercept(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let intercept = parse_number!(parser, "intercept");
    parser.eat(TokenKind::Separator)?;
    let set = parse_number!(parser, "set");
    Ok(VCommand::StateRedirect { intercept, set })
}
fn parse_set_health(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let amount = parse_v_number!(parser, "health amount");
    Ok(VCommand::SetHealth { amount })
}
fn parse_set_energy(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let amount = parse_v_number!(parser, "health amount");
    Ok(VCommand::SetEnergy { amount })
}
fn parse_set_spirit_light(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let amount = parse_v_number!(parser, "health amount");
    Ok(VCommand::SetSpiritLight { amount })
}
fn parse_equip(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let slot = parse_v_number!(parser, "equip slot");
    parser.eat(TokenKind::Separator)?;
    let ability = parse_number!(parser, "equipment");
    Ok(VCommand::Equip { slot, ability })
}
fn parse_ahk_signal(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let token = parser.eat(TokenKind::Ident)?;
    let signal = parser.read_token(&token).to_string();
    Ok(VCommand::AhkSignal { signal })
}
fn parse_if_equal(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_state = parse_v_uber_state_condition(parser)?;
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VCommand::IfEqual { uber_state, item })
}
fn parse_if_greater(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_state = parse_v_uber_state_condition(parser)?;
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VCommand::IfGreater { uber_state, item })
}
fn parse_if_less(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_state = parse_v_uber_state_condition(parser)?;
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VCommand::IfLess { uber_state, item })
}
fn parse_disable_sync(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_identifier = parse_uber_identifier(parser)?;
    Ok(VCommand::DisableSync { uber_identifier })
}
fn parse_enable_sync(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_identifier = parse_uber_identifier(parser)?;
    Ok(VCommand::EnableSync { uber_identifier })
}
fn parse_create_warp(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let id = parse_number!(parser, "warp id");
    parser.eat(TokenKind::Separator)?;
    let position = parse_v_position(parser)?;
    Ok(VCommand::CreateWarp { id, position })
}
fn parse_destroy_warp(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let id = parse_number!(parser, "warp id");
    Ok(VCommand::DestroyWarp { id })
}
fn parse_if_box(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let position1 = parse_v_position(parser)?;
    parser.eat(TokenKind::Separator)?;
    let position2 = parse_v_position(parser)?;
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VCommand::IfBox { position1, position2, item })
}
fn parse_if_self_equal(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let value = parse_v_number!(parser, "");
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VCommand::IfSelfEqual { value, item })
}
fn parse_if_self_greater(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let value = parse_v_number!(parser, "");
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VCommand::IfSelfGreater { value, item })
}
fn parse_if_self_less(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let value = parse_v_number!(parser, "");
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VCommand::IfSelfLess { value, item })
}
fn parse_unequip(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let ability = parse_number!(parser, "equipment");
    Ok(VCommand::UnEquip { ability })
}
fn parse_save_string(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let id = parse_number!(parser, "string id");
    parser.eat(TokenKind::Separator)?;
    let token = parser.eat(TokenKind::Number)?;
    let string = VString(parser.read_token(&token).to_string());
    Ok(VCommand::SaveString { id, string })
}
fn parse_append_string(parser: &mut Parser) -> Result<VCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let id = parse_number!(parser, "string id");
    parser.eat(TokenKind::Separator)?;
    let token = parser.eat(TokenKind::Number)?;
    let string = VString(parser.read_token(&token).to_string());
    Ok(VCommand::AppendString { id, string })
}
fn parse_teleporter(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let (teleporter, remove) = parse_removable_number!(parser, "teleporter");
    let item = if remove {
        VItem::RemoveTeleporter(teleporter)
    } else {
        VItem::Teleporter(teleporter)
    };
    Ok(item)
}
#[derive(FromStr)]
#[ParseFromIdentifier]
enum MessageFlag {
    Mute,
    F,
    Instant,
    Quiet,
    P,
    NoClear,
}
fn parse_message(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let message = VString(parse_string(parser)?.to_owned());
    let mut message = VMessage::new(message);
    while parser.current_token().kind == TokenKind::Separator {
        let peeked = parser.peek_token();
        if peeked.kind == TokenKind::Ident {
            let range = peeked.range.clone();
            if let Ok(flag) = parser.read(range).parse() {
                parser.next_token();
                parser.next_token();
                match flag {
                    MessageFlag::Mute => message.mute = true,
                    MessageFlag::F => message.frames = Some(parse_value!(parser, "frames")),
                    MessageFlag::Instant => message.instant = true,
                    MessageFlag::Quiet => message.quiet = true,
                    MessageFlag::P => message.pos = Some(parse_value!(parser, "position")),
                    MessageFlag::NoClear => message.noclear = true,
                }
            } else { break }
        } else { break }
    }
    Ok(VItem::Message(message))
}
fn parse_set_uber_state(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let uber_identifier = parse_uber_identifier(parser)?;
    parser.eat(TokenKind::Separator)?;
    let uber_type = parse_ident!(parser, "uber state type");
    parser.eat(TokenKind::Separator)?;

    let (signed, sign) = match parser.current_token().kind {
        TokenKind::Plus => {
            parser.next_token();
            (true, true)
        },
        TokenKind::Minus => {
            parser.next_token();
            (true, false)
        },
        _ => (false, false),
    };

    let token = parser.current_token().clone();
    let operator = match token.kind {
        TokenKind::OpenBracket => {
            let start = parse_boundary(parser, &uber_type)?;
            parser.eat(TokenKind::Comma)?;
            let end = parse_boundary(parser, &uber_type)?;
            parser.eat(TokenKind::CloseBracket)?;
            VUberStateOperator::Range(VUberStateRange { start, end })
        },
        TokenKind::Dollar if parser.peek_token().kind == TokenKind::OpenParen => {
            parser.next_token();
            let identifier = parse_pointer(parser)?;
            VUberStateOperator::Pointer(identifier)
        },
        TokenKind::Dollar | TokenKind::Number | TokenKind::Ident => {
            let value = match uber_type {
                UberType::Bool | UberType::Teleporter => { parse_v_ident!(parser, "boolean") },
                UberType::Byte => { parse_v_number!(parser, "byte") },
                UberType::Int => { parse_v_number!(parser, "integer") },
                UberType::Float => { parse_v_number!(parser, "float") },
            };
            VUberStateOperator::Value(value)
        },
        _ => return Err(ParseError::new("expected uber state operator", token.range)),
    };

    let mut skip = false;
    if parser.current_token().kind == TokenKind::Separator {
        let peeked = parser.peek_token();
        if peeked.kind == TokenKind::Ident {
            let range = peeked.range.clone();
            if parser.read(range) == "skip" {
                parser.next_token();
                parser.next_token();
                skip = true
            }
        }
    }

    Ok(VItem::UberState(VUberStateItem { uber_identifier, uber_type, signed, sign, operator, skip }))
}
fn parse_boundary(parser: &mut Parser, uber_type: &UberType) -> Result<VUberStateRangeBoundary, ParseError> {
    let token = parser.current_token().clone();
    let boundary = match token.kind {
        TokenKind::Dollar if parser.peek_token().kind == TokenKind::OpenParen => {
            parser.next_token();
            let identifier = parse_pointer(parser)?;
            VUberStateRangeBoundary::Pointer(identifier)
        },
        TokenKind::Dollar | TokenKind::Number | TokenKind::Ident => {
            let value = match uber_type {
                UberType::Bool | UberType::Teleporter => { parse_v_ident!(parser, "boolean") },
                UberType::Byte => { parse_v_number!(parser, "byte") },
                UberType::Int => { parse_v_number!(parser, "integer") },
                UberType::Float => { parse_v_number!(parser, "float") },
            };
            VUberStateRangeBoundary::Value(value)
        },
        _ => return Err(ParseError::new("expected value or pointer", token.range)),
    };
    Ok(boundary)
}
fn parse_pointer(parser: &mut Parser) -> Result<UberIdentifier, ParseError> {
    parser.eat(TokenKind::OpenParen)?;
    let identifier = parse_uber_identifier(parser)?;
    parser.eat(TokenKind::CloseParen)?;
    Ok(identifier)
}
fn parse_water(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let (_, remove): (WorldEventKind, bool) = parse_removable_number!(parser, "world event");
    let item = if remove {
        VItem::RemoveWater
    } else {
        VItem::Water
    };
    Ok(item)
}
fn parse_bonus_item(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let bonus_item = parse_number!(parser, "bonus item");
    Ok(VItem::BonusItem(bonus_item))
}
fn parse_bonus_upgrade(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let bonus_upgrade = parse_number!(parser, "bonus upgrade");
    Ok(VItem::BonusUpgrade(bonus_upgrade))
}
fn parse_relic(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let zone = parse_number!(parser, "zone");
    Ok(VItem::Relic(zone))
}
fn parse_sys_message(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let kind = parse_number!(parser, "system message kind");
    let sys_message = match kind {
        SysMessageKind::RelicList => SysMessage::RelicList,
        SysMessageKind::MapRelicList => {
            parser.eat(TokenKind::Separator)?;
            let zone = parse_number!(parser, "zone");
            SysMessage::MapRelicList(zone)
        },
        SysMessageKind::PickupCount => SysMessage::PickupCount,
        SysMessageKind::GoalProgress => SysMessage::GoalProgress,
    };
    Ok(VItem::SysMessage(sys_message))
}
fn parse_wheel_command(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let kind = parse_number!(parser, "command kind");
    let command = match kind {
        WheelCommandKind::SetName => parse_wheel_set_name(parser),
        WheelCommandKind::SetDescription => parse_wheel_set_description(parser),
        WheelCommandKind::SetIcon => parse_wheel_set_icon(parser),
        WheelCommandKind::SetColor => parse_wheel_set_color(parser),
        WheelCommandKind::SetItem => parse_wheel_set_item(parser),
        WheelCommandKind::SetSticky => parse_wheel_set_sticky(parser),
        WheelCommandKind::SwitchWheel => parse_wheel_switch_wheel(parser),
        WheelCommandKind::RemoveItem => parse_wheel_remove_item(parser),
        WheelCommandKind::ClearAll => Ok(VWheelCommand::ClearAll),
    }?;
    Ok(VItem::WheelCommand(command))
}
fn parse_wheel_item_position(parser: &mut Parser) -> Result<(u32, WheelItemPosition), ParseError> {
    parser.eat(TokenKind::Separator)?;
    let wheel = parse_number!(parser, "wheel identifier");
    parser.eat(TokenKind::Separator)?;
    let position = parse_number!(parser, "wheel item position");
    Ok((wheel, position))
}
fn parse_wheel_set_name(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    let (wheel, position) = parse_wheel_item_position(parser)?;
    parser.eat(TokenKind::Separator)?;
    let name = VString(parse_string(parser)?.to_owned());
    Ok(VWheelCommand::SetName { wheel, position, name })
}
fn parse_wheel_set_description(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    let (wheel, position) = parse_wheel_item_position(parser)?;
    parser.eat(TokenKind::Separator)?;
    let description = VString(parse_string(parser)?.to_owned());
    Ok(VWheelCommand::SetDescription { wheel, position, description })
}
fn parse_wheel_set_icon(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    let (wheel, position) = parse_wheel_item_position(parser)?;
    parser.eat(TokenKind::Separator)?;
    let icon = parse_icon(parser)?;
    Ok(VWheelCommand::SetIcon { wheel, position, icon })
}
fn parse_wheel_set_color(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    let (wheel, position) = parse_wheel_item_position(parser)?;
    parser.eat(TokenKind::Separator)?;
    let r = parse_v_number!(parser, "red color channel");
    parser.eat(TokenKind::Separator)?;
    let g = parse_v_number!(parser, "red color channel");
    parser.eat(TokenKind::Separator)?;
    let b = parse_v_number!(parser, "red color channel");
    parser.eat(TokenKind::Separator)?;
    let a = parse_v_number!(parser, "red color channel");
    Ok(VWheelCommand::SetColor { wheel, position, r, g, b, a })
}
fn parse_wheel_set_item(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    let (wheel, position) = parse_wheel_item_position(parser)?;
    parser.eat(TokenKind::Separator)?;
    let bind = parse_number!(parser, "wheel bind");
    parser.eat(TokenKind::Separator)?;
    let item = Box::new(parse_item(parser)?);
    Ok(VWheelCommand::SetItem { wheel, position, bind, item })
}
fn parse_wheel_set_sticky(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let wheel = parse_number!(parser, "wheel identifier");
    parser.eat(TokenKind::Separator)?;
    let sticky = parse_v_ident!(parser, "sticky value");
    Ok(VWheelCommand::SetSticky { wheel, sticky })
}
fn parse_wheel_switch_wheel(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let wheel = parse_number!(parser, "wheel identifier");
    Ok(VWheelCommand::SwitchWheel { wheel })
}
fn parse_wheel_remove_item(parser: &mut Parser) -> Result<VWheelCommand, ParseError> {
    let (wheel, position) = parse_wheel_item_position(parser)?;
    Ok(VWheelCommand::RemoveItem { wheel, position })
}
fn parse_shop_command(parser: &mut Parser) -> Result<VItem, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let kind = parse_number!(parser, "command kind");
    parser.eat(TokenKind::Separator)?;
    let uber_identifier = parse_uber_identifier(parser)?;
    let command = match kind {
        ShopCommandKind::SetIcon => parse_shop_set_icon(parser, uber_identifier),
        ShopCommandKind::SetTitle => parse_shop_set_title(parser, uber_identifier),
        ShopCommandKind::SetDescription => parse_shop_set_description(parser, uber_identifier),
        ShopCommandKind::SetLocked => parse_shop_set_locked(parser, uber_identifier),
        ShopCommandKind::SetVisible => parse_shop_set_visible(parser, uber_identifier),
    }?;
    Ok(VItem::ShopCommand(command))
}
fn parse_optional_string(parser: &mut Parser) -> Result<Option<VString>, ParseError> {
    let string = if parser.current_token().kind == TokenKind::Separator {
        if matches!(parser.peek_token().kind, TokenKind::String { .. }) {
            parser.next_token();
            let string = VString(parse_string(parser)?.to_owned());
            Some(string)
        } else { None }
    } else { None };
    Ok(string)
}
fn parse_shop_set_icon(parser: &mut Parser, uber_identifier: UberIdentifier) -> Result<VShopCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let icon = parse_icon(parser)?;
    Ok(VShopCommand::SetIcon { uber_identifier, icon })
}
fn parse_shop_set_title(parser: &mut Parser, uber_identifier: UberIdentifier) -> Result<VShopCommand, ParseError> {
    let title = parse_optional_string(parser)?;
    Ok(VShopCommand::SetTitle { uber_identifier, title })
}
fn parse_shop_set_description(parser: &mut Parser, uber_identifier: UberIdentifier) -> Result<VShopCommand, ParseError> {
    let description = parse_optional_string(parser)?;
    Ok(VShopCommand::SetDescription { uber_identifier, description })
}
fn parse_shop_set_locked(parser: &mut Parser, uber_identifier: UberIdentifier) -> Result<VShopCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let locked = parse_v_ident!(parser, "sticky value");
    Ok(VShopCommand::SetLocked { uber_identifier, locked })
}
fn parse_shop_set_visible(parser: &mut Parser, uber_identifier: UberIdentifier) -> Result<VShopCommand, ParseError> {
    parser.eat(TokenKind::Separator)?;
    let visible = parse_v_ident!(parser, "visible value");
    Ok(VShopCommand::SetVisible { uber_identifier, visible })
}
