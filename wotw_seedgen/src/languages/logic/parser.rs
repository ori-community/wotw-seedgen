use std::ops::Range;
use std::str::FromStr;

use smallvec::SmallVec;
use wotw_seedgen_derive::{Display, FromStr};
use crate::generator::doors::DoorId;
use crate::item;
use crate::languages::parser::{
    parse_ident, parse_number, parse_value, read_ident, ParseErrorCollection,
};
use crate::languages::{ParseError, TokenKind};
use crate::settings::{Difficulty, Trick};
use crate::util::{Enemy, NodeKind, Position, RefillValue};

use super::tokenizer::{tokenize, TokenStream};

type Parser<'a> = crate::languages::Parser<'a, TokenStream<'a>>;
fn new(input: &str) -> Parser {
    crate::languages::Parser::new(input, tokenize(input))
}

/// Syntax Tree representing an areas file
///
/// This is one needed component that has to be passed to [`logic::build`](crate::logic::build)
///
/// Use [`Areas::parse`] to parse a string into this format
#[derive(Debug, Clone)]
pub struct Areas<'a> {
    pub(super) contents: Vec<AreaContent<'a>>,
}
#[derive(Debug, Clone)]
pub enum AreaContent<'a> {
    Requirement(NamedGroup<'a>),
    Region(NamedGroup<'a>),
    Anchor(Anchor<'a>),
}
#[derive(Debug, Clone)]
pub struct NamedGroup<'a> {
    pub name: &'a str,
    pub group: Group<'a>,
}
#[derive(Debug, Clone)]
pub struct Anchor<'a> {
    pub identifier: &'a str,
    pub position: Option<Position>,
    pub can_spawn: bool,
    pub teleport_restriction: Option<Group<'a>>,
    pub refills: Vec<Refill<'a>>,
    pub connections: Vec<Connection<'a>>,
    pub door: Option<Door<'a>>,
}
#[derive(Debug, Clone)]
pub struct Refill<'a> {
    pub value: RefillValue,
    pub requirements: Option<Group<'a>>,
}
#[derive(Debug, Clone)]
pub struct Connection<'a> {
    pub kind: NodeKind,
    pub identifier: &'a str,
    pub requirements: Group<'a>,
}
#[derive(Debug, Clone)]
pub struct Door<'a> {
    pub door_id: DoorId,
    pub target: &'a str,
    pub enter: Group<'a>,
}
#[derive(Debug, Clone)]
pub struct Group<'a> {
    pub lines: Vec<Line<'a>>,
}
#[derive(Debug, Clone)]
pub struct Line<'a> {
    pub ands: Vec<Requirement<'a>>,
    pub ors: Vec<Requirement<'a>>,
    pub group: Option<Group<'a>>,
}
#[derive(Debug, Clone)]
pub struct Requirement<'a> {
    pub value: RequirementValue<'a>,
    pub range: Range<usize>,
}
#[derive(Debug, Clone)]
pub enum RequirementValue<'a> {
    Free,
    Impossible,
    Macro(&'a str),
    Difficulty(Difficulty),
    Trick(Trick),
    Skill(Skill),
    UseSkill(Skill, u32),
    SpiritLight(u32),
    Resource(Resource, u32),
    Shard(Shard),
    Teleporter(Teleporter),
    Water,
    State(&'a str),
    Damage(u32),
    Danger(u32),
    Combat(SmallVec<[(Enemy, u8); 12]>),
    Boss(u32),
    BreakWall(u32),
    BreakCrystal,
    ShurikenBreak(u32),
    SentryBreak(u32),
    HammerBreak,
    SpearBreak,
    SentryJump(u32),
    SwordSentryJump(u32),
    HammerSentryJump(u32),
    SentryBurn(u32),
    LaunchSwap,
    SentrySwap(u32),
    FlashSwap,
    BlazeSwap(u32),
    AbilitySwap(u32),
    WaveDash,
    GrenadeJump,
    GrenadeCancel,
    BowCancel,
    SwordJump,
    GrenadeRedirect(u32),
    SentryRedirect(u32),
    GlideJump,
    AerialHammerJump,
    GlideHammerJump,
    CoyoteHammerJump,
    WallHammerJump,
    GroundedHammerJump,
    HammerExtension,
    SpearJump(u32),
    GlideBashChain,
    DoubleJumpBashChain,
    DashBashChain,
    LaunchBashChain,
}

impl<'a> Areas<'a> {
    /// Parses the input string into the [`Areas`] representation
    pub fn parse(input: &'a str) -> Result<Areas<'a>, ParseErrorCollection> {
        let mut contents = Vec::new();
        let mut errors = ParseErrorCollection::default();
        let mut parser = new(input);
        loop {
            parser.skip_while(|kind| kind == TokenKind::Newline || kind == TokenKind::Whitespace);
            if parser.current_token().kind == TokenKind::Eof {
                break;
            }
            match parse_content(&mut parser) {
                Ok(content) => contents.push(content),
                Err(err) => errors.push(err),
            }
        }

        fill_macros_and_states(&mut contents, &parser).unwrap_or_else(|err| errors.push(err));

        match errors.is_empty() {
            true => Ok(Self { contents }),
            false => Err(errors),
        }
    }
}

impl<'a> Anchor<'a> {
    pub fn region(&self) -> &'a str {
        self.identifier
            .split_once('.')
            .map_or(self.identifier, |parts| parts.0)
    }
}

#[derive(Display)]
enum Suggestion {
    Content,
    Identifier,
    Integer,
    Float,
    Requirement,
    Enemy,
    AnchorContent,
    DoorContent,
    Refill,
}

fn recover(parser: &mut Parser, recover_if: fn(TokenKind) -> bool) {
    let mut depth = 0;
    loop {
        let token = parser.current_token();
        if token.kind == TokenKind::Eof {
            break;
        }
        if token.kind == TokenKind::Indent {
            depth += 1
        }
        if depth > 0 {
            if matches!(token.kind, TokenKind::Dedent { .. }) {
                depth -= 1
            }
        } else if recover_if(token.kind) {
            break;
        }
        parser.next_token();
    }
}
fn check_dedent(parser: &mut Parser) -> Result<bool, ParseError> {
    let token = parser.current_token();
    match token.kind {
        TokenKind::Dedent { matching } => {
            let token = parser.next_token();
            match matching {
                true => Ok(true),
                false => Err(parser.error("malformed Indent", token.range)),
            }
        }
        _ => Ok(false),
    }
}

#[derive(FromStr)]
#[ParseFromIdentifier]
enum ContentKind {
    Requirement,
    Region,
    Anchor,
}
fn parse_content<'a>(parser: &mut Parser<'a>) -> Result<AreaContent<'a>, ParseError> {
    let content_kind = parse_ident!(parser, Suggestion::Content)?;
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::Content)?;
    match content_kind {
        ContentKind::Requirement => parse_macro(parser),
        ContentKind::Region => parse_region(parser),
        ContentKind::Anchor => parse_anchor(parser),
    }
}
fn parse_macro<'a>(parser: &mut Parser<'a>) -> Result<AreaContent<'a>, ParseError> {
    let name = read_ident!(parser, Suggestion::Identifier)?;
    let group = parse_group(parser)?;
    Ok(AreaContent::Requirement(NamedGroup { name, group }))
}
fn parse_region<'a>(parser: &mut Parser<'a>) -> Result<AreaContent<'a>, ParseError> {
    let name = read_ident!(parser, Suggestion::Identifier)?;
    let group = parse_group(parser)?;
    Ok(AreaContent::Region(NamedGroup { name, group }))
}
fn parse_anchor<'a>(parser: &mut Parser<'a>) -> Result<AreaContent<'a>, ParseError> {
    let token_range = parser.current_token().range.clone();
    let identifier = read_ident!(parser, Suggestion::Identifier)?;
    if identifier == "Random" || identifier == "FullyRandom" {
        return Err(parser.error(
            "An anchor cannot be named \"Random\" or \"FullyRandom\"",
            token_range,
        ));
    }
    parser.skip(TokenKind::Whitespace);
    let position = parse_anchor_position(parser)?;
    parser.skip(TokenKind::Whitespace);
    parser.eat(TokenKind::Indent)?;

    let mut can_spawn = true;
    let mut teleport_restriction = None;
    let mut refills = Vec::new();
    let mut connections = Vec::new();
    let mut door = None;

    loop {
        if check_dedent(parser)? {
            break;
        }

        let start = parser.current_token().range.start;

        match parse_anchor_content(parser) {
            Ok(content) => match content {
                AnchorContent::NoSpawn => can_spawn = false,
                AnchorContent::TpRestriction(requirement) => {
                    if teleport_restriction.replace(requirement).is_some() {
                        let range = start..parser.current_token().range.start;
                        recover(parser, |kind| matches!(kind, TokenKind::Dedent { .. }));
                        check_dedent(parser)?;
                        return Err(
                            parser.error("An anchor may only have one teleport restriction", range)
                        );
                    }
                }
                AnchorContent::Refill(refill) => refills.push(refill),
                AnchorContent::Connection(connection) => connections.push(connection),
                AnchorContent::Door(requirement) => {
                    if door.replace(requirement).is_some() {
                        let range = start..parser.current_token().range.start;
                        recover(parser, |kind| matches!(kind, TokenKind::Dedent { .. }));
                        check_dedent(parser)?;
                        return Err(parser.error("An anchor may only have one door", range));
                    }
                }
            },
            Err(err) => {
                recover(parser, |kind| matches!(kind, TokenKind::Dedent { .. }));
                check_dedent(parser)?;
                return Err(err);
            }
        }
    }

    Ok(AreaContent::Anchor(Anchor {
        identifier,
        position,
        can_spawn,
        teleport_restriction,
        refills,
        connections,
        door,
    }))
}
fn parse_anchor_position(parser: &mut Parser) -> Result<Option<Position>, ParseError> {
    let token = parser.next_token();
    let position = match token.kind {
        TokenKind::Identifier if parser.read_token(&token) == "at" => {
            parser.eat(TokenKind::Whitespace)?;
            let x = parse_number!(parser, Suggestion::Float)?;
            parser.eat(TokenKind::Comma)?;
            parser.skip(TokenKind::Whitespace);
            let y = parse_number!(parser, Suggestion::Float)?;
            parser.eat_or_suggest(TokenKind::Colon, Suggestion::Float)?;
            Some(Position { x, y })
        }
        TokenKind::Colon => None,
        _ => return Err(parser.error("Expected Colon or at", token.range)),
    };
    Ok(position)
}
#[derive(FromStr)]
#[ParseFromIdentifier]
enum AnchorContentKind {
    NoSpawn,
    TpRestriction,
    Refill,
    State,
    Quest,
    Pickup,
    Conn,
    Door,
}
enum AnchorContent<'a> {
    NoSpawn,
    TpRestriction(Group<'a>),
    Refill(Refill<'a>),
    Connection(Connection<'a>),
    Door(Door<'a>),
}
#[derive(FromStr)]
#[ParseFromIdentifier]
enum DoorContentKind {
    Id,
    Target,
    Enter,
}
enum DoorContent<'a> {
    Id(DoorId),
    Target(&'a str),
    Enter(Group<'a>),
}
fn parse_anchor_content<'a>(parser: &mut Parser<'a>) -> Result<AnchorContent<'a>, ParseError> {
    let kind = parse_ident!(parser, Suggestion::AnchorContent)?;
    let content = match kind {
        AnchorContentKind::NoSpawn => {
            parser.skip(TokenKind::Whitespace);
            parser.eat_or_suggest(TokenKind::Newline, Suggestion::AnchorContent)?;
            AnchorContent::NoSpawn
        }
        AnchorContentKind::TpRestriction => AnchorContent::TpRestriction(parse_group(parser)?),
        AnchorContentKind::Refill => AnchorContent::Refill(parse_anchor_refill(parser)?),
        AnchorContentKind::State => {
            AnchorContent::Connection(parse_anchor_connection(parser, NodeKind::State)?)
        }
        AnchorContentKind::Quest => {
            AnchorContent::Connection(parse_anchor_connection(parser, NodeKind::Quest)?)
        }
        AnchorContentKind::Pickup => {
            AnchorContent::Connection(parse_anchor_connection(parser, NodeKind::Pickup)?)
        }
        AnchorContentKind::Conn => {
            AnchorContent::Connection(parse_anchor_connection(parser, NodeKind::Anchor)?)
        }
        AnchorContentKind::Door => AnchorContent::Door(parse_door(parser)?),
    };
    Ok(content)
}
fn parse_optional_group<'a>(parser: &mut Parser<'a>) -> Result<Option<Group<'a>>, ParseError> {
    let group = match parser.current_token().kind {
        TokenKind::Colon => Some(parse_group(parser)?),
        _ => {
            parser.skip(TokenKind::Whitespace);
            parser.eat_or_suggest(TokenKind::Newline, Suggestion::Refill)?;
            None
        }
    };

    Ok(group)
}
#[derive(FromStr)]
#[ParseFromIdentifier]
enum RefillKind {
    Full,
    Checkpoint,
    Health,
    Energy,
}
fn parse_anchor_refill<'a>(parser: &mut Parser<'a>) -> Result<Refill<'a>, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::AnchorContent)?;
    let value = parse_refill_value(parser)?;
    let requirements = parse_optional_group(parser)?;

    Ok(Refill {
        value,
        requirements,
    })
}
fn parse_refill_value(parser: &mut Parser) -> Result<RefillValue, ParseError> {
    let kind = parse_ident!(parser, Suggestion::Refill)?;
    match kind {
        RefillKind::Full => Ok(RefillValue::Full),
        RefillKind::Checkpoint => Ok(RefillValue::Checkpoint),
        RefillKind::Health => {
            parse_value!(parser, Suggestion::Float, Suggestion::Refill).map(RefillValue::Health)
        }
        RefillKind::Energy => {
            parse_value!(parser, Suggestion::Float, Suggestion::Refill).map(RefillValue::Energy)
        }
    }
}
fn parse_anchor_connection<'a>(
    parser: &mut Parser<'a>,
    kind: NodeKind,
) -> Result<Connection<'a>, ParseError> {
    parser.eat_or_suggest(TokenKind::Whitespace, Suggestion::AnchorContent)?;
    let identifier = read_ident!(parser, Suggestion::Identifier)?;
    let requirements = parse_group(parser)?;
    Ok(Connection {
        kind,
        identifier,
        requirements,
    })
}
fn parse_door<'a>(parser: &mut Parser<'a>) -> Result<Door<'a>, ParseError> {
    parser.eat(TokenKind::Colon)?;
    parser.skip(TokenKind::Whitespace);
    parser.eat(TokenKind::Indent)?;

    let mut door_id: Option<DoorId> = None;
    let mut target: Option<&'a str> = None;
    let mut enter: Option<Group<'a>> = None;
    let start = parser.current_token().range.start;

    loop {
        match parse_door_content(parser)? {
            DoorContent::Id(id) => door_id = Some(id),
            DoorContent::Target(t) =>  target = Some(t),
            DoorContent::Enter(requirements) => enter = Some(requirements),
        }

        parser.skip(TokenKind::Whitespace);

        if check_dedent(parser)? {
            break
        }

        parser.eat(TokenKind::Newline)?;
    }

    let end = parser.current_token().range.start;

    let door_id = door_id.ok_or_else(|| parser.error(
        "Door is missing id",
        start..end,
    ))?;
    let target = target.ok_or_else(|| parser.error(
        "Door is missing default_target: <anchor name>",
        start..end,
    ))?;
    let enter = enter.ok_or_else(|| parser.error(
        "Door is missing enter requirements (enter: ...)",
        start..end,
    ))?;

    Ok(Door {
        door_id,
        target,
        enter,
    })
}
fn parse_door_content<'a>(parser: &mut Parser<'a>) -> Result<DoorContent<'a>, ParseError> {
    let kind = parse_ident!(parser, Suggestion::DoorContent)?;

    let content = match kind {
        DoorContentKind::Id => {
            parser.eat(TokenKind::Colon)?;
            parser.skip(TokenKind::Whitespace);
            DoorContent::Id(parse_number!(parser, Suggestion::Integer)?)
        }
        DoorContentKind::Target => {
            parser.eat(TokenKind::Colon)?;
            parser.skip(TokenKind::Whitespace);
            DoorContent::Target(read_ident!(parser, Suggestion::Identifier)?)
        }
        DoorContentKind::Enter => {
            DoorContent::Enter(parse_group(parser)?)
        }
    };

    Ok(content)
}

fn parse_group<'a>(parser: &mut Parser<'a>) -> Result<Group<'a>, ParseError> {
    parser.eat(TokenKind::Colon)?;
    parser.skip(TokenKind::Whitespace);

    let mut lines = Vec::new();
    if parser.current_token().kind == TokenKind::Indent {
        parser.next_token();

        loop {
            match parse_line(parser) {
                Ok(line) => {
                    lines.push(line);
                    if check_dedent(parser)? {
                        break;
                    }
                }
                Err(err) => {
                    recover(parser, |kind| matches!(kind, TokenKind::Dedent { .. }));
                    check_dedent(parser)?;
                    return Err(err);
                }
            }
        }
    } else {
        let line = parse_line(parser)?;
        lines.push(line);
    }

    Ok(Group { lines })
}

fn parse_line<'a>(parser: &mut Parser<'a>) -> Result<Line<'a>, ParseError> {
    let mut ands = Vec::new();
    let mut ors = Vec::with_capacity(1);
    let mut has_seen_or = false;

    let group = loop {
        let requirement = parse_requirement(parser)?;
        parser.skip(TokenKind::Whitespace);

        let token = parser.current_token();
        match parser.current_token().kind {
            TokenKind::Comma => match has_seen_or {
                true => return Err(parser.error("Comma after OR", token.range.clone())),
                false => ands.push(requirement),
            },
            TokenKind::Identifier if parser.read_token(token) == "OR" => {
                ors.push(requirement);
                has_seen_or = true;
            }
            TokenKind::Newline => {
                ors.push(requirement);
                parser.next_token();
                break None;
            }
            TokenKind::Colon => {
                let group = parse_group(parser)?;
                ors.push(requirement);
                break Some(group);
            }
            TokenKind::Indent => return Err(parser.error("unexpected Indent", token.range.clone())),
            _ => return Err(parser.error("expected separator or Newline", token.range.clone())),
        }
        parser.next_token();
        parser.skip(TokenKind::Whitespace);
    };

    Ok(Line { ands, ors, group })
}

#[derive(Display, FromStr)]
#[ParseFromIdentifier]
pub enum RequirementKind {
    Free,
    Impossible,
    SpiritLight,
    Water,
    Damage,
    Danger,
    Combat,
    Boss,
    BreakWall,
    BreakCrystal,
    ShurikenBreak,
    SentryBreak,
    HammerBreak,
    SpearBreak,
    SentryJump,
    SwordSJump,
    HammerSJump,
    SentryBurn,
    LaunchSwap,
    SentrySwap,
    FlashSwap,
    BlazeSwap,
    AbilitySwap,
    WaveDash,
    GrenadeJump,
    GrenadeCancel,
    BowCancel,
    SwordJump,
    GrenadeRedirect,
    SentryRedirect,
    GlideJump,
    AerialHammerJump,
    GlideHammerJump,
    CoyoteHammerJump,
    WallHammerJump,
    GroundedHammerJump,
    HammerExtension,
    SpearJump,
    GlideBashChain,
    DoubleJumpBashChain,
    DashBashChain,
    LaunchBashChain,
}
#[derive(Debug, Clone, Copy, FromStr)]
#[ParseFromIdentifier]
pub enum Resource {
    Health = 0,
    Energy = 1,
    Ore = 2,
    Keystone = 3,
    ShardSlot = 4,
}
impl From<Resource> for item::Resource {
    fn from(resource: Resource) -> item::Resource {
        item::Resource::try_from(resource as u8).unwrap()
    }
}
#[derive(Debug, Clone, Copy, FromStr)]
#[ParseFromIdentifier]
pub enum Skill {
    Bash = 0,
    WallJump = 3,
    DoubleJump = 5,
    Launch = 8,
    Glide = 14,
    WaterBreath = 23,
    Grenade = 51,
    Grapple = 57,
    Flash = 62,
    Spear = 74,
    Regenerate = 77,
    Bow = 97,
    Hammer = 98,
    Sword = 100,
    Burrow = 101,
    Dash = 102,
    WaterDash = 104,
    Shuriken = 106,
    Seir = 108,
    Blaze = 115,
    Sentry = 116,
    Flap = 118,
}
impl From<Skill> for item::Skill {
    fn from(skill: Skill) -> item::Skill {
        item::Skill::try_from(skill as u8).unwrap()
    }
}
#[derive(Debug, Clone, Copy, FromStr)]
#[ParseFromIdentifier]
pub enum Shard {
    TripleJump = 2,
    Bounty = 4,
    Magnet = 8,
    Quickshot = 14,
    LifeHarvest = 23,
    EnergyHarvest = 25,
    Sense = 30,
    UltraBash = 32,
    UltraGrapple = 33,
    Thorn = 35,
    Catalyst = 36,
    Turmoil = 38,
    Sticky = 39,
    Deflector = 44,
    Fracture = 46,
    Arcing = 47,
}
impl From<Shard> for item::Shard {
    fn from(shard: Shard) -> item::Shard {
        item::Shard::try_from(shard as u8).unwrap()
    }
}
#[derive(Debug, Clone, Copy, FromStr)]
#[ParseFromIdentifier]
pub enum Teleporter {
    MarshTP = 16,
    DenTP = 1,
    HollowTP = 5,
    GladesTP = 17,
    WellspringTP = 3,
    BurrowsTP = 0,
    WestWoodsTP = 7,
    EastWoodsTP = 8,
    ReachTP = 4,
    DepthsTP = 6,
    EastPoolsTP = 2,
    WestPoolsTP = 13,
    WestWastesTP = 9,
    EastWastesTP = 10,
    OuterRuinsTP = 11,
    InnerRuinsTP = 14,
    WillowTP = 12,
    ShriekTP = 15,
}
impl From<Teleporter> for item::Teleporter {
    fn from(shard: Teleporter) -> item::Teleporter {
        item::Teleporter::try_from(shard as u8).unwrap()
    }
}
fn parse_requirement<'a>(parser: &mut Parser<'a>) -> Result<Requirement<'a>, ParseError> {
    let start = parser.current_token().range.start;
    let identifier = read_ident!(parser, Suggestion::Requirement)?;
    let value = match RequirementKind::from_str(identifier) {
        Ok(kind) => parse_special_requirement(parser, &kind)?,
        Err(_) => match Difficulty::from_str(identifier) {
            Ok(difficulty) => RequirementValue::Difficulty(difficulty),
            Err(_) => match Trick::from_str(identifier) {
                Ok(trick) => RequirementValue::Trick(trick),
                Err(_) => match Skill::from_str(identifier) {
                    Ok(skill) => match parser.current_token().kind {
                        TokenKind::Eq => {
                            let value =
                                parse_value!(parser, Suggestion::Integer, Suggestion::Requirement)?;
                            match skill {
                                Skill::Regenerate => return Err(parser.error("Explicit Regenerate amounts are forbidden, try removing the value", start..parser.current_token().range.start)),  // The game has buggy logic for when you're allowed to regenerate and there should never be a reason to explicitely force an amount of Regenerates
                                _ => RequirementValue::UseSkill(skill, value),
                            }
                        }
                        _ => RequirementValue::Skill(skill),
                    },
                    Err(_) => match Resource::from_str(identifier) {
                        Ok(resource) => RequirementValue::Resource(
                            resource,
                            parse_value!(parser, Suggestion::Integer, Suggestion::Requirement)?,
                        ),
                        Err(_) => match Shard::from_str(identifier) {
                            Ok(shard) => RequirementValue::Shard(shard),
                            Err(_) => match Teleporter::from_str(identifier) {
                                Ok(teleporter) => RequirementValue::Teleporter(teleporter),
                                Err(_) => RequirementValue::State(identifier), // This will get checked against the available states later
                            },
                        },
                    },
                },
            },
        },
    };
    let range = start..parser.current_token().range.start;
    Ok(Requirement { value, range })
}
fn parse_special_requirement<'a>(
    parser: &mut Parser<'a>,
    kind: &RequirementKind,
) -> Result<RequirementValue<'a>, ParseError> {
    let requirement = match kind {
        RequirementKind::Free => RequirementValue::Free,
        RequirementKind::Impossible => RequirementValue::Impossible,
        RequirementKind::Water => RequirementValue::Water,
        RequirementKind::BreakCrystal => RequirementValue::BreakCrystal,
        RequirementKind::HammerBreak => RequirementValue::HammerBreak,
        RequirementKind::SpearBreak => RequirementValue::SpearBreak,
        RequirementKind::LaunchSwap => RequirementValue::LaunchSwap,
        RequirementKind::FlashSwap => RequirementValue::FlashSwap,
        RequirementKind::WaveDash => RequirementValue::WaveDash,
        RequirementKind::GrenadeJump => RequirementValue::GrenadeJump,
        RequirementKind::GrenadeCancel => RequirementValue::GrenadeCancel,
        RequirementKind::BowCancel => RequirementValue::BowCancel,
        RequirementKind::SwordJump => RequirementValue::SwordJump,
        RequirementKind::GlideJump => RequirementValue::GlideJump,
        RequirementKind::AerialHammerJump => RequirementValue::AerialHammerJump,
        RequirementKind::GlideHammerJump => RequirementValue::GlideHammerJump,
        RequirementKind::CoyoteHammerJump => RequirementValue::CoyoteHammerJump,
        RequirementKind::WallHammerJump => RequirementValue::WallHammerJump,
        RequirementKind::GroundedHammerJump => RequirementValue::GroundedHammerJump,
        RequirementKind::HammerExtension => RequirementValue::HammerExtension,
        RequirementKind::GlideBashChain => RequirementValue::GlideBashChain,
        RequirementKind::DoubleJumpBashChain => RequirementValue::DoubleJumpBashChain,
        RequirementKind::DashBashChain => RequirementValue::DashBashChain,
        RequirementKind::LaunchBashChain => RequirementValue::LaunchBashChain,
        RequirementKind::Combat => parse_combat_requirement(parser)?,
        _ => {
            let amount = parse_value!(parser, Suggestion::Integer, Suggestion::Requirement)?;
            match kind {
                RequirementKind::SpiritLight => RequirementValue::SpiritLight(amount),
                RequirementKind::Damage => RequirementValue::Damage(amount),
                RequirementKind::Danger => RequirementValue::Danger(amount),
                RequirementKind::Boss => RequirementValue::Boss(amount),
                RequirementKind::BreakWall => RequirementValue::BreakWall(amount),
                RequirementKind::ShurikenBreak => RequirementValue::ShurikenBreak(amount),
                RequirementKind::SentryBreak => RequirementValue::SentryBreak(amount),
                RequirementKind::SentryJump => RequirementValue::SentryJump(amount),
                RequirementKind::SwordSJump => RequirementValue::SwordSentryJump(amount),
                RequirementKind::HammerSJump => RequirementValue::HammerSentryJump(amount),
                RequirementKind::SentryBurn => RequirementValue::SentryBurn(amount),
                RequirementKind::SentrySwap => RequirementValue::SentrySwap(amount),
                RequirementKind::BlazeSwap => RequirementValue::BlazeSwap(amount),
                RequirementKind::AbilitySwap => RequirementValue::AbilitySwap(amount),
                RequirementKind::GrenadeRedirect => RequirementValue::GrenadeRedirect(amount),
                RequirementKind::SentryRedirect => RequirementValue::SentryRedirect(amount),
                RequirementKind::SpearJump => RequirementValue::SpearJump(amount),
                _ => panic!("Missing implementation for requirement {}", kind),
            }
        }
    };
    Ok(requirement)
}
fn parse_combat_requirement<'a>(
    parser: &mut Parser<'a>,
) -> Result<RequirementValue<'a>, ParseError> {
    parser.eat_or_suggest(TokenKind::Eq, Suggestion::Requirement)?;
    let mut enemies = SmallVec::new();
    loop {
        let amount = if parser.current_token().kind == TokenKind::Number {
            let amount = parse_number!(parser, Suggestion::Integer)?;
            parser.eat_or_suggest(TokenKind::X, Suggestion::Integer)?;
            amount
        } else {
            1
        };
        let enemy = parse_ident!(parser, Suggestion::Enemy)?;
        enemies.push((enemy, amount));
        if parser.current_token().kind == TokenKind::Plus {
            parser.next_token();
            continue;
        }
        break;
    }
    Ok(RequirementValue::Combat(enemies))
}

fn fill_macros_and_states(
    contents: &mut Vec<AreaContent>,
    parser: &Parser,
) -> Result<(), ParseError> {
    let mut macros = Vec::new();
    let mut states = Vec::new();
    for content in contents.iter_mut() {
        match content {
            AreaContent::Requirement(named_group) => macros.push(named_group.name),
            AreaContent::Anchor(anchor) => {
                for connection in &anchor.connections {
                    if connection.kind == NodeKind::State || connection.kind == NodeKind::Quest {
                        states.push(connection.identifier);
                    }
                }
            }
            AreaContent::Region(_) => {}
        }
    }

    for content in contents {
        match content {
            AreaContent::Requirement(named_group) | AreaContent::Region(named_group) => {
                fill_group(&mut named_group.group, &macros, &states, parser)?
            }
            AreaContent::Anchor(anchor) => {
                for refill in &mut anchor.refills {
                    if let Some(requirement) = &mut refill.requirements {
                        fill_group(requirement, &macros, &states, parser)?;
                    }
                }
                for connection in &mut anchor.connections {
                    fill_group(&mut connection.requirements, &macros, &states, parser)?;
                }
            }
        }
    }

    Ok(())
}
fn fill_group(
    group: &mut Group,
    macros: &[&str],
    states: &[&str],
    parser: &Parser,
) -> Result<(), ParseError> {
    for line in &mut group.lines {
        for and in &mut line.ands {
            fill_requirement(and, macros, states, parser)?;
        }
        for or in &mut line.ors {
            fill_requirement(or, macros, states, parser)?;
        }
        if let Some(group) = &mut line.group {
            fill_group(group, macros, states, parser)?;
        }
    }
    Ok(())
}
fn fill_requirement(
    requirement: &mut Requirement,
    macros: &[&str],
    states: &[&str],
    parser: &Parser,
) -> Result<(), ParseError> {
    if let RequirementValue::State(identifier) = requirement.value {
        if macros.contains(&identifier) {
            requirement.value = RequirementValue::Macro(identifier);
        } else if !states.contains(&identifier) {
            return Err(parser.error("unknown requirement", requirement.range.clone()));
        };
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn logic_parse() {
        let input = std::fs::read_to_string("areas.wotw").unwrap();
        if let Err(err) = Areas::parse(&input) {
            panic!("{}", err.verbose_display());
        }
    }
}
