use std::str::FromStr;

use wotw_seedgen_derive::FromStr;

use crate::VItem;

use crate::header::{HeaderCommand, ParameterDefault, V, VString, ParameterType, GoalmodeHack};
use crate::languages::TokenKind;

use super::{Parser, ParseError, parse_ident, parse_number, parse_v_number, parse_string, parse_icon, Suggestion};

#[derive(FromStr)]
#[ParseFromIdentifier]
enum HeaderCommandKind {
    Include,
    Exclude,
    Add,
    Remove,
    Name,
    Display,
    Description,
    Price,
    Icon,
    Parameter,
    Set,
    #[Ident = "if"] StartIf,
    EndIf,
    #[Ident = "__goalmode_hack"] GoalmodeHack,
}

impl HeaderCommand {
    pub(crate) fn parse(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
        let kind = parse_ident!(parser, Suggestion::HeaderCommand)?;
        match kind {
            HeaderCommandKind::Include => parse_include(parser),
            HeaderCommandKind::Exclude => parse_exclude(parser),
            HeaderCommandKind::Add => parse_add(parser),
            HeaderCommandKind::Remove => parse_remove(parser),
            HeaderCommandKind::Name => parse_name(parser),
            HeaderCommandKind::Display => parse_display(parser),
            HeaderCommandKind::Description => parse_description(parser),
            HeaderCommandKind::Price => parse_price(parser),
            HeaderCommandKind::Icon => parse_icon_command(parser),
            HeaderCommandKind::Parameter => parse_parameter(parser),
            HeaderCommandKind::Set => parse_set(parser),
            HeaderCommandKind::StartIf => parse_if(parser),
            HeaderCommandKind::EndIf => Ok(HeaderCommand::EndIf),
            HeaderCommandKind::GoalmodeHack => parse_goalmode(parser),
        }
    }
}
impl FromStr for HeaderCommand {
    type Err = ParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = super::new(input);
        let command = HeaderCommand::parse(&mut parser)?;
        parser.expect_end()?;
        Ok(command)
    }
}

fn parse_item_amount(parser: &mut Parser) -> Result<V<i32>, ParseError> {
    if match parser.current_token().kind {
        TokenKind::Number => {
            let peeked = parser.peek_token();
            if peeked.kind == TokenKind::Identifier {
                let range = peeked.range.clone();
                parser.read(range) == "x"
            } else { false }
        },
        TokenKind::Dollar => true,
        _ => false,
    } {
        let amount = parse_v_number!(parser, Suggestion::Integer);
        parser.next_token();
        parser.skip(TokenKind::Whitespace);
        return Ok(amount);
    }
    Ok(V::Literal(1))
}

fn parse_include(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let name = parse_ident!(parser, Suggestion::Identifier)?;
    Ok(HeaderCommand::Include { name })
}
fn parse_exclude(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let name = parse_ident!(parser, Suggestion::Identifier)?;
    Ok(HeaderCommand::Exclude { name })
}
fn parse_add(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let amount = parse_item_amount(parser)?;
    let item = VItem::parse(parser)?;
    Ok(HeaderCommand::Add { item, amount })
}
fn parse_remove(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let amount = parse_item_amount(parser)?;
    let item = VItem::parse(parser)?;
    Ok(HeaderCommand::Remove { item, amount })
}
fn parse_name(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let name = VString(parse_string(parser).to_owned());
    Ok(HeaderCommand::Name { item, name })
}
fn parse_display(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let name = VString(parse_string(parser).to_owned());
    Ok(HeaderCommand::Display { item, name })
}
fn parse_description(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let description = VString(parse_string(parser).to_owned());
    Ok(HeaderCommand::Description { item, description })
}
fn parse_price(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let price = parse_v_number!(parser, Suggestion::Integer);
    Ok(HeaderCommand::Price { item, price })
}
fn parse_icon_command(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let item = VItem::parse(parser)?;
    parser.eat(TokenKind::Whitespace)?;
    let icon = parse_icon(parser)?;
    Ok(HeaderCommand::Icon { item, icon })
}
fn parse_parameter(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let identifier = parse_ident!(parser, Suggestion::Identifier)?;
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::Identifier)?;
    let parameter_type = parse_ident!(parser, Suggestion::ParameterType)?;
    parser.eat_or_suggest(TokenKind::Colon, Suggestion::ParameterType)?;
    let default = match parameter_type {
        ParameterType::Bool => ParameterDefault::Bool(parse_ident!(parser, Suggestion::Boolean)?),
        ParameterType::Int => ParameterDefault::Int(parse_number!(parser, Suggestion::Integer)?),
        ParameterType::Float => ParameterDefault::Float(parse_number!(parser, Suggestion::Float)?),
        ParameterType::String => ParameterDefault::String(parse_string(parser).to_owned()),
    };
    Ok(HeaderCommand::Parameter { identifier, default })
}
fn parse_set(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let mut state = String::new();
    loop {
        let token = parser.eat(TokenKind::Identifier)?;
        state.push_str(parser.read_token(&token));
        if parser.current_token().kind == TokenKind::Dot {
            state.push('.');
            parser.next_token();
        } else {
            break;
        }
    }
    Ok(HeaderCommand::Set { state })
}
fn parse_if(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;
    let parameter = parse_ident!(parser, Suggestion::Identifier)?;
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::Identifier)?;
    let token = parser.next_token();
    let value = parser.read_token(&token).to_owned();
    Ok(HeaderCommand::If { parameter, value })
}
#[derive(FromStr)]
#[ParseFromIdentifier]
enum Goalmode {
    Trees,
    Wisps,
    Quests,
    Relics,
}
fn parse_goalmode(parser: &mut Parser) -> Result<HeaderCommand, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::HeaderCommand)?;

    let goal = match parse_ident!(parser, Suggestion::Integer)? {
        Goalmode::Trees => GoalmodeHack::Trees,
        Goalmode::Wisps => GoalmodeHack::Wisps,
        Goalmode::Quests => GoalmodeHack::Quests,
        Goalmode::Relics => {
            parser.eat(TokenKind::Whitespace)?;
            let chance = parse_v_number!(parser, Suggestion::Float);
            parser.eat(TokenKind::Whitespace)?;
            let amount = parse_v_number!(parser, Suggestion::Integer);
            GoalmodeHack::Relics { chance, amount }
        },
    };

    Ok(HeaderCommand::GoalmodeHack(goal))
}
