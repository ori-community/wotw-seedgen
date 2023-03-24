mod header_command;
mod parse_item;
mod postprocess;
mod preprocess;

pub use postprocess::postprocess;
pub(super) use preprocess::preprocess;
use wotw_seedgen_derive::{Display, FromStr};

use std::str::FromStr;

use crate::{
    languages::parser::ParseErrorCollection,
    uber_state::{UberIdentifier, UberStateComparator, VUberStateCondition, VUberStateTrigger},
    util::Icon,
    VItem,
};

use super::{
    tokenizer::tokenize, Annotation, HeaderCommand, HeaderContent, TimerDefinition, VPickup, V,
};

use crate::languages::parser::{parse_ident, parse_number};
use crate::languages::{CommentKind, ParseError, TokenKind};
pub(crate) type Parser<'a> = crate::languages::Parser<'a, super::TokenStream<'a>>;
pub(crate) fn new(input: &str) -> Parser {
    crate::languages::Parser::new(input, tokenize(input))
}

#[derive(FromStr)]
#[ParseFromIdentifier]
enum InterpolationCommand {
    Param,
}

macro_rules! parse_removable_number {
    ($parser:expr, $expected:path) => {{
        let token = $parser.eat_or_suggest($crate::languages::TokenKind::Number, $expected)?;
        let mut string = $parser.read_token(&token);
        let remove = if let Some(remove) = string.strip_prefix('-') {
            string = remove;
            true
        } else {
            false
        };
        let parsed = string.parse().map_err($crate::languages::parser::invalid!(
            token, $parser, $expected
        ))?;
        (parsed, remove)
    }};
}
use parse_removable_number;
fn parse_string<'a>(parser: &'a mut Parser) -> &'a str {
    let start = parser.current_token().range.start;
    parser.skip_while(|kind| !matches!(kind, TokenKind::Newline | TokenKind::Comment { .. }));
    let end = parser.current_token().range.start;

    parser.read(start..end).trim_end()
}
fn parse_v_param<T: FromStr>(parser: &mut Parser) -> Result<V<T>, ParseError> {
    parser.eat_or_suggest(TokenKind::OpenParen, Suggestion::InterpolationCommand)?;
    let token = parser.eat(TokenKind::Identifier)?;
    let param = parser.read_token(&token).to_owned();
    parser.eat(TokenKind::CloseParen)?;
    Ok(V::Parameter(param))
}
macro_rules! parse_v {
    ($parser:expr, $token:ident, $expected:path) => {
        match $token.kind {
            $crate::languages::TokenKind::Dollar => {
                let command = $crate::languages::parser::parse_ident!(
                    $parser,
                    $crate::header::parser::Suggestion::InterpolationCommand
                )?;
                match command {
                    $crate::header::parser::InterpolationCommand::Param => {
                        $crate::header::parser::parse_v_param($parser)?
                    }
                }
            }
            _ => {
                return Err($parser
                    .error(format!("Expected {}", $expected), $token.range)
                    .with_suggestion($expected))
            }
        }
    };
}
use parse_v;
macro_rules! parse_v_or_kind {
    ($parser:expr, $expected:path, $kind:pat) => {{
        let token = $parser.next_token();
        match token.kind {
            $kind => {
                let string = $parser.read_token(&token);
                let literal = string.parse().map_err($crate::languages::parser::invalid!(
                    token, $parser, $expected
                ))?;
                $crate::header::V::Literal(literal)
            }
            _ => $crate::header::parser::parse_v!($parser, token, $expected),
        }
    }};
}
use parse_v_or_kind;
macro_rules! parse_v_ident {
    ($parser:expr, $expected:path) => {
        $crate::header::parser::parse_v_or_kind!(
            $parser,
            $expected,
            $crate::languages::TokenKind::Identifier
        )
    };
}
use parse_v_ident;
macro_rules! parse_v_number {
    ($parser:expr, $expected:path) => {
        $crate::header::parser::parse_v_or_kind!(
            $parser,
            $expected,
            $crate::languages::TokenKind::Number
        )
    };
}
use parse_v_number;
macro_rules! parse_v_removable_number {
    ($parser:expr, $expected:path) => {{
        let token = $parser.next_token();
        match token.kind {
            $crate::languages::TokenKind::Number => {
                let mut string = $parser.read_token(&token);
                let remove = if let Some(remove) = string.strip_prefix('-') {
                    string = remove;
                    true
                } else {
                    false
                };
                let parsed = string.parse().map_err($crate::languages::parser::invalid!(
                    token, $parser, $expected
                ))?;
                (V::Literal(parsed), remove)
            }
            $crate::languages::TokenKind::Minus => (
                $crate::header::parser::parse_v_number!($parser, $expected),
                true,
            ),
            _ => (
                $crate::header::parser::parse_v!($parser, token, $expected),
                false,
            ),
        }
    }};
}
use parse_v_removable_number;

fn trim_comment(input: &str) -> &str {
    input
        .find("//")
        .map_or(input, |index| &input[..index])
        .trim_end()
}

#[derive(Display)]
pub(crate) enum Suggestion {
    UberGroup,
    UberId,
    UberTriggerValue,
    Integer,
    Float,
    Boolean,
    NumericBoolean,
    Identifier,
    IconKind,
    ShardIcon,
    SpellIcon,
    OpherIcon,
    LupoIcon,
    GromIcon,
    TuleyIcon,
    Annotation,
    Expression,
    InterpolationCommand,
    UberConditionValue,
    ItemKind,
    Resource,
    Skill,
    Shard,
    CommandKind,
    ToggleCommandKind,
    EquipSlot,
    Spell,
    Teleporter,
    MessageFlag,
    UberType,
    WorldEvent,
    BonusItem,
    BonusUpgrade,
    Zone,
    SysMessageKind,
    WheelCommandKind,
    WheelItemPosition,
    WheelBind,
    ShopCommandKind,
    HeaderCommand,
    ParameterType,
    PickupFlag,
}

fn parse_uber_identifier(parser: &mut Parser) -> Result<UberIdentifier, ParseError> {
    let uber_group = parse_number!(parser, Suggestion::UberGroup)?;
    parser.eat_or_suggest(TokenKind::Separator, Suggestion::UberGroup)?;
    let uber_id = parse_number!(parser, Suggestion::UberId)?;
    Ok(UberIdentifier {
        uber_group,
        uber_id,
    })
}
#[derive(PartialEq, FromStr)]
#[ParseFromIdentifier]
enum IconKind {
    Shard,
    Spell,
    Opher,
    Lupo,
    Grom,
    Tuley,
    File,
}
fn parse_icon(parser: &mut Parser) -> Result<Icon, ParseError> {
    let kind = parse_ident!(parser, Suggestion::IconKind)?;
    parser.eat_or_suggest(TokenKind::Colon, Suggestion::IconKind)?;
    let icon = match kind {
        IconKind::Shard => Icon::Shard(parse_number!(parser, Suggestion::ShardIcon)?),
        IconKind::Spell => Icon::Spell(parse_number!(parser, Suggestion::SpellIcon)?),
        IconKind::Opher => Icon::Opher(parse_number!(parser, Suggestion::OpherIcon)?),
        IconKind::Lupo => Icon::Lupo(parse_number!(parser, Suggestion::LupoIcon)?),
        IconKind::Grom => Icon::Grom(parse_number!(parser, Suggestion::GromIcon)?),
        IconKind::Tuley => Icon::Tuley(parse_number!(parser, Suggestion::TuleyIcon)?),
        IconKind::File => Icon::File(parse_string(parser).to_owned()),
    };
    Ok(icon)
}

struct ParseContext<'a, 'b> {
    parser: &'a mut Parser<'b>,
    contents: Vec<HeaderContent>,
    skip_validation: bool,
}
impl<'b> ParseContext<'_, '_> {
    fn new<'a>(parser: &'a mut Parser<'b>) -> ParseContext<'a, 'b> {
        ParseContext {
            parser,
            contents: Vec::default(),
            skip_validation: bool::default(),
        }
    }
}
pub(super) fn parse_header_contents(
    parser: &mut Parser,
) -> Result<Vec<HeaderContent>, ParseErrorCollection> {
    let mut context = ParseContext::new(parser);
    let mut errors = ParseErrorCollection::default();

    loop {
        parse_whitespace(&mut context);
        if context.parser.current_token().kind == TokenKind::Eof {
            break;
        }
        match parse_expression(&mut context) {
            Ok(header_content) => {
                context.contents.push(header_content);
                context.parser.skip_while(|kind| {
                    matches!(kind, TokenKind::Whitespace | TokenKind::Comment { .. })
                });
                if context.parser.current_token().kind == TokenKind::Eof {
                    break;
                }
                if let Err(err) = context.parser.eat(TokenKind::Newline) {
                    recover(err, &mut errors, context.parser);
                }
            }
            Err(err) => recover(err, &mut errors, context.parser),
        }
        context.skip_validation = false;
    }

    if errors.is_empty() {
        Ok(context.contents)
    } else {
        Err(errors)
    }
}

fn recover(err: ParseError, errors: &mut ParseErrorCollection, parser: &mut Parser) {
    errors.push(err);
    parser.skip_while(|kind| kind != TokenKind::Newline);
    parser.next_token();
}

fn parse_whitespace(context: &mut ParseContext) {
    loop {
        let current_token = context.parser.current_token();
        match current_token.kind {
            TokenKind::Newline | TokenKind::Whitespace => {}
            TokenKind::Comment { kind } => {
                let comment = context.parser.read_token(current_token);
                match kind {
                    CommentKind::Note => {
                        if comment[2..].trim() == "skip-validate" {
                            context.skip_validation = true
                        }
                    }
                    CommentKind::HeaderDoc => context.contents.push(
                        HeaderContent::OuterDocumentation(comment[3..].trim().to_owned()),
                    ),
                    CommentKind::ConfigDoc => context.contents.push(
                        HeaderContent::InnerDocumentation(comment[4..].trim().to_owned()),
                    ),
                }
            }
            _ => return,
        }
        context.parser.next_token();
    }
}
#[derive(FromStr)]
#[ParseFromIdentifier]
enum ExpressionIdentKind {
    Flags,
    Timer,
}
fn parse_expression(context: &mut ParseContext) -> Result<HeaderContent, ParseError> {
    let parser = &mut (*context.parser);
    let current_token = parser.current_token();
    match current_token.kind {
        TokenKind::Identifier => {
            let kind = parse_ident!(parser, Suggestion::Expression)?;
            parser.eat_or_suggest(TokenKind::Colon, Suggestion::Expression)?;
            parser.skip(TokenKind::Whitespace);
            match kind {
                ExpressionIdentKind::Flags => parse_flags(parser),
                ExpressionIdentKind::Timer => parse_timer(parser),
            }
        }
        TokenKind::Bang => {
            parser.next_token();
            if parser.current_token().kind == TokenKind::Bang {
                parser.next_token();
                parse_command(parser)
            } else {
                parse_pickup(context, true)
            }
        }
        TokenKind::Number | TokenKind::Dollar => parse_pickup(context, false),
        TokenKind::Pound => {
            parser.next_token();
            parse_annotation(parser)
        }
        _ => Err(parser.error("expected expression", current_token.range.clone())),
    }
}
fn parse_flags(parser: &mut Parser) -> Result<HeaderContent, ParseError> {
    let mut flags = vec![];

    loop {
        let start = parser.current_token().range.start;
        parser.skip_while(|kind| {
            !matches!(
                kind,
                TokenKind::Newline | TokenKind::Comment { .. } | TokenKind::Comma
            )
        });
        let end = parser.current_token().range.start;

        let flag = parser.read(start..end).trim_end().to_owned();
        flags.push(flag);

        if parser.current_token().kind == TokenKind::Comma {
            parser.next_token();
            parser.skip(TokenKind::Whitespace);
        } else {
            break;
        }
    }

    Ok(HeaderContent::Flags(flags))
}
fn parse_timer(parser: &mut Parser) -> Result<HeaderContent, ParseError> {
    let switch = parse_uber_identifier(parser)?;
    parser.eat_or_suggest(TokenKind::Separator, Suggestion::UberId)?;
    let counter = parse_uber_identifier(parser)?;

    let timer_definition = TimerDefinition { switch, counter };
    Ok(HeaderContent::Timer(timer_definition))
}
fn parse_command(parser: &mut Parser) -> Result<HeaderContent, ParseError> {
    HeaderCommand::parse(parser).map(HeaderContent::Command)
}
#[derive(FromStr)]
#[ParseFromIdentifier]
enum PickupFlag {
    Mute,
}
fn parse_pickup(context: &mut ParseContext, ignore: bool) -> Result<HeaderContent, ParseError> {
    let parser = &mut (*context.parser);

    let (trigger, suggestion) = parse_trigger(parser)?;
    parser.eat_or_suggest(TokenKind::Separator, suggestion)?;

    let item = VItem::parse(parser)?;
    let skip_validation = context.skip_validation;

    let hide_others = if parser.current_token().kind == TokenKind::Separator {
        parser.next_token();
        let _: PickupFlag = parse_ident!(parser, Suggestion::PickupFlag)?;
        true
    } else {
        false
    };

    let pickup = VPickup {
        trigger,
        item,
        ignore,
        hide_others,
        skip_validation,
    };
    Ok(HeaderContent::Pickup(pickup))
}

fn parse_trigger(parser: &mut Parser) -> Result<(VUberStateTrigger, Suggestion), ParseError> {
    let identifier = parse_uber_identifier(parser)?;
    let (condition, suggestion) = parse_uber_state_condition(parser)?;
    let trigger = VUberStateTrigger {
        identifier,
        condition,
    };

    Ok((trigger, suggestion))
}
fn parse_uber_state_condition(
    parser: &mut Parser,
) -> Result<(Option<VUberStateCondition>, Suggestion), ParseError> {
    match parse_comparator(parser) {
        Some(comparator) => {
            let value = parse_v_number!(parser, Suggestion::UberTriggerValue);
            Ok((
                Some(VUberStateCondition { comparator, value }),
                Suggestion::UberTriggerValue,
            ))
        }
        None => Ok((None, Suggestion::UberId)),
    }
}
fn parse_comparator(parser: &mut Parser) -> Option<UberStateComparator> {
    match parser.current_token().kind {
        TokenKind::Eq => {
            parser.next_token();
            Some(UberStateComparator::Equals)
        }
        TokenKind::Greater => {
            parser.next_token();
            if parser.current_token().kind == TokenKind::Eq {
                parser.next_token();
                Some(UberStateComparator::GreaterOrEquals)
            } else {
                Some(UberStateComparator::Greater)
            }
        }
        TokenKind::Less => {
            parser.next_token();
            if parser.current_token().kind == TokenKind::Eq {
                parser.next_token();
                Some(UberStateComparator::LessOrEquals)
            } else {
                Some(UberStateComparator::Less)
            }
        }
        _ => None,
    }
}

impl VUberStateTrigger {
    pub(crate) fn parse(parser: &mut Parser) -> Result<VUberStateTrigger, ParseError> {
        parse_trigger(parser).map(|(trigger, _)| trigger)
    }
}

#[derive(FromStr)]
#[ParseFromIdentifier]
enum AnnotationKind {
    Hide,
    Category,
}
impl Annotation {
    pub(crate) fn parse(parser: &mut Parser) -> Result<Annotation, ParseError> {
        let kind = parse_ident!(parser, Suggestion::Annotation)?;
        match kind {
            AnnotationKind::Hide => Ok(Annotation::Hide),
            AnnotationKind::Category => parse_category(parser),
        }
    }
}
impl FromStr for Annotation {
    type Err = ParseError;
    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut parser = new(input);
        let annotation = Annotation::parse(&mut parser)?;
        parser.expect_end()?;
        Ok(annotation)
    }
}
fn parse_category(parser: &mut Parser) -> Result<Annotation, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::Annotation)?;
    let category = parse_string(parser).to_owned();
    Ok(Annotation::Category(category))
}
fn parse_annotation(parser: &mut Parser) -> Result<HeaderContent, ParseError> {
    Annotation::parse(parser).map(HeaderContent::Annotation)
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::item::*;
    use crate::uber_state::*;

    #[test]
    fn item_parsing() {
        assert_eq!(Item::from_str("0|5000"), Ok(Item::SpiritLight(5000)));
        assert_eq!(Item::from_str("0|-5000"), Ok(Item::RemoveSpiritLight(5000)));
        assert_eq!(
            Item::from_str("1|2"),
            Ok(Item::Resource(Resource::GorlekOre))
        );
        assert!(Item::from_str("1|-2").is_err());
        assert!(Item::from_str("1|5").is_err());
        assert_eq!(Item::from_str("2|8"), Ok(Item::Skill(Skill::Launch)));
        assert_eq!(
            Item::from_str("2|120"),
            Ok(Item::Skill(Skill::GladesAncestralLight))
        );
        assert_eq!(
            Item::from_str("2|121"),
            Ok(Item::Skill(Skill::InkwaterAncestralLight))
        );
        assert!(Item::from_str("2|25").is_err());
        assert!(Item::from_str("2|-9").is_err());
        assert_eq!(Item::from_str("3|28"), Ok(Item::Shard(Shard::LastStand)));
        assert_eq!(
            Item::from_str("5|16"),
            Ok(Item::Teleporter(Teleporter::Marsh))
        );
        assert_eq!(Item::from_str("9|0"), Ok(Item::Water));
        assert_eq!(Item::from_str("9|-0"), Ok(Item::RemoveWater));
        assert_eq!(
            Item::from_str("11|0"),
            Ok(Item::BonusUpgrade(BonusUpgrade::RapidHammer))
        );
        assert_eq!(
            Item::from_str("10|31"),
            Ok(Item::BonusItem(BonusItem::EnergyRegeneration))
        );
        assert!(Item::from_str("8|5|3|6").is_err());
        assert!(Item::from_str("8||||").is_err());
        assert!(Item::from_str("8|5|3|in|3").is_err());
        assert!(Item::from_str("8|5|3|bool|3").is_err());
        assert!(Item::from_str("8|5|3|float|hm").is_err());
        assert_eq!(
            Item::from_str("8|5|3|int|6"),
            Ok(UberStateItem::simple_setter(
                UberIdentifier::new(5, 3),
                UberType::Int,
                UberStateValue::Number((6.).into())
            ))
        );
        assert_eq!(Item::from_str("4|0"), Ok(Item::Command(Command::Autosave)));
        assert!(Item::from_str("12").is_err());
        assert!(Item::from_str("").is_err());
        assert!(Item::from_str("0|").is_err());
        assert!(Item::from_str("0||400").is_err());
        assert!(Item::from_str("7|3").is_err());
        assert!(Item::from_str("-0|65").is_err());
    }
}
