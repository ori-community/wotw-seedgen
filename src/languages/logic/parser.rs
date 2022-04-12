use decorum::R32;
use rustc_hash::FxHashMap;
use smallvec::SmallVec;

use super::tokenizer::{Token, TokenType, Metadata};
use crate::item::{Resource, Skill, Shard, Teleporter};
use crate::util::{Difficulty, Glitch, RefillType, NodeType, Enemy, Position};

#[derive(Debug)]
pub struct ParseError {
    pub description: String,
    pub position: usize,
}
impl ParseError {
    fn new(position: usize, description: String) -> ParseError {
        ParseError {
            description,
            position,
        }
    }
}

#[derive(Debug)]
pub enum Requirement<'a> {
    Free,
    Impossible,
    Definition(&'a str),
    Difficulty(Difficulty),
    Glitch(Glitch),
    Skill(Skill),
    EnergySkill(Skill, u16),
    SpiritLight(u16),
    Resource(Resource, u16),
    Shard(Shard),
    Teleporter(Teleporter),
    Water,
    State(&'a str),
    Damage(u16),
    Danger(u16),
    Combat(SmallVec<[(Enemy, u8); 12]>),
    Boss(u16),
    BreakWall(u16),
    BreakCrystal,
    ShurikenBreak(u16),
    SentryBreak(u16),
    HammerBreak,
    SpearBreak,
    SentryJump(u16),
    SwordSentryJump(u16),
    HammerSentryJump(u16),
    SentryBurn(u16),
    LaunchSwap,
    SentrySwap(u16),
    FlashSwap,
    BlazeSwap(u16),
    WaveDash,
    GrenadeJump,
    GrenadeCancel,
    HammerJump,
    SwordJump,
    GrenadeRedirect(u16),
    SentryRedirect(u16),
    GlideJump,
    GlideHammerJump,
    SpearJump(u16),
}
#[derive(Debug, Default)]
pub struct Line<'a> {
    pub ands: Vec<Requirement<'a>>,
    pub ors: Vec<Requirement<'a>>,
    pub group: Option<Group<'a>>,
}
#[derive(Debug)]
pub struct Group<'a> {
    pub lines: Vec<Line<'a>>
}
#[derive(Debug)]
pub struct Refill<'a> {
    pub name: RefillType,
    pub requirements: Option<Group<'a>>,
}
#[derive(Debug)]
pub struct Connection<'a> {
    pub name: NodeType,
    pub identifier: &'a str,
    pub requirements: Group<'a>,
}
#[derive(Debug)]
pub struct Anchor<'a> {
    pub identifier: &'a str,
    pub position: Option<Position>,
    pub can_spawn: bool,
    pub refills: Vec<Refill<'a>>,
    pub connections: Vec<Connection<'a>>,
}
#[derive(Debug)]
pub struct AreaTree<'a> {
    pub definitions: FxHashMap<&'a str, Group<'a>>,
    pub regions: FxHashMap<&'a str, Group<'a>>,
    pub anchors: Vec<Anchor<'a>>,
}

macro_rules! __expected_variants {
    ($expected:expr) => {
        $expected
    };
    ($expected:expr, $($more:expr),+) => {
        format!("{} or {}", $expected, __expected_variants!($($more),+))
    }
}

macro_rules! wrong_token {
    ($token:expr, $($expected:expr),+) => {
        return Err(ParseError::new($token.position, format!("Expected {} at line {}, instead found {}", __expected_variants!($($expected),+), $token.line, $token.name)))
    };
}
macro_rules! missing_token {
    ($($expected:expr),+) => {
        return Err(ParseError {
            description: format!("File ended abruptly, expected {}", __expected_variants!($($expected),+)),
            position: usize::MAX,
        })
    }
}

fn wrong_amount(token: &Token) -> ParseError {
    ParseError::new(token.position, format!("Failed to parse amount at line {}", token.line))
}
fn wrong_requirement(token: &Token) -> ParseError {
    ParseError::new(token.position, format!("Failed to parse requirement at line {}", token.line))
}
fn not_int(token: &Token) -> ParseError {
    ParseError::new(token.position, format!("Need an integer in {} at line {}", token.name, token.line))
}

macro_rules! next_token {
    ($tokens:expr, $position:expr, $($expected:expr),+) => {
        if let Some(token) = $tokens.get($position) {
            $position += 1;
            token
        } else {
            missing_token!($($expected),+);
        }
    };
}

#[inline]
fn eat<'a>(tokens: &[Token<'a>], position: &mut usize, expected: TokenType) -> Result<(), ParseError> {
    let token = next_token!(tokens, *position, expected);
    if token.name == expected {
        Ok(())
    } else {
        wrong_token!(token, expected)
    }
}

fn parse_requirement<'a>(token: &Token<'a>, metadata: &Metadata) -> Result<Requirement<'a>, ParseError> {
    let mut parts = token.value.split('=');
    let keyword = parts.next().unwrap();
    let amount = parts.next();
    if parts.next().is_some() {
        return Err(wrong_amount(token));
    }
    match amount {
        Some(amount) => {
            if keyword == "Combat" {
                let mut enemies = SmallVec::new();
                for enemy in amount.split('+') {
                    let mut parts = enemy.split('x');
                    let amount = parts.next().ok_or_else(|| wrong_requirement(token))?;
                    let (enemy, amount) = match parts.next() {
                        Some(enemy) => (enemy, amount),
                        None => (amount, "1"),
                    };
                    if parts.next().is_some() {
                        return Err(wrong_requirement(token));
                    }
                    let amount: u8 = match amount.parse() {
                        Ok(result) => result,
                        Err(_) => return Err(not_int(token)),
                    };
                    let enemy = match enemy {
                        "Mantis" => Ok(Enemy::Mantis),
                        "Slug" => Ok(Enemy::Slug),
                        "WeakSlug" => Ok(Enemy::WeakSlug),
                        "BombSlug" => Ok(Enemy::BombSlug),
                        "CorruptSlug" => Ok(Enemy::CorruptSlug),
                        "SneezeSlug" => Ok(Enemy::SneezeSlug),
                        "ShieldSlug" => Ok(Enemy::ShieldSlug),
                        "Lizard" => Ok(Enemy::Lizard),
                        "Bat" => Ok(Enemy::Bat),
                        "Hornbug" => Ok(Enemy::Hornbug),
                        "Skeeto" => Ok(Enemy::Skeeto),
                        "SmallSkeeto" => Ok(Enemy::SmallSkeeto),
                        "Bee" => Ok(Enemy::Bee),
                        "Nest" => Ok(Enemy::Nest),
                        "Crab" => Ok(Enemy::Crab),
                        "SpinCrab" => Ok(Enemy::SpinCrab),
                        "Tentacle" => Ok(Enemy::Tentacle),
                        "Balloon" => Ok(Enemy::Balloon),
                        "Miner" => Ok(Enemy::Miner),
                        "MaceMiner" => Ok(Enemy::MaceMiner),
                        "ShieldMiner" => Ok(Enemy::ShieldMiner),
                        "CrystalMiner" => Ok(Enemy::CrystalMiner),
                        "ShieldCrystalMiner" => Ok(Enemy::ShieldCrystalMiner),
                        "Sandworm" => Ok(Enemy::Sandworm),
                        "Spiderling" => Ok(Enemy::Spiderling),
                        "EnergyRefill" => Ok(Enemy::EnergyRefill),
                        _ => Err(wrong_requirement(token)),
                    }?;
                    enemies.push((enemy, amount));
                }
                return Ok(Requirement::Combat(enemies));
            }
            let amount: u16 = match amount.parse() {
                Ok(result) => result,
                Err(_) => return Err(not_int(token)),
            };
            match keyword {
                "Blaze" => Ok(Requirement::EnergySkill(Skill::Blaze, amount)),
                "BlazeSwap" => Ok(Requirement::BlazeSwap(amount)),
                "Boss" => Ok(Requirement::Boss(amount)),
                "Bow" => Ok(Requirement::EnergySkill(Skill::Bow, amount)),
                "BreakWall" => Ok(Requirement::BreakWall(amount)),
                "Damage" => Ok(Requirement::Damage(amount)),
                "Danger" => Ok(Requirement::Danger(amount)),
                "Energy" => Ok(Requirement::Resource(Resource::Energy, amount)),
                "Flash" => Ok(Requirement::EnergySkill(Skill::Flash, amount)),
                "Grenade" => Ok(Requirement::EnergySkill(Skill::Grenade, amount)),
                "GrenadeRedirect" => Ok(Requirement::GrenadeRedirect(amount)),
                "HammerSJump" => Ok(Requirement::HammerSentryJump(amount)),
                "Health" => Ok(Requirement::Resource(Resource::Health, amount)),
                "Keystone" => Ok(Requirement::Resource(Resource::Keystone, amount)),
                "Ore" => Ok(Requirement::Resource(Resource::Ore, amount)),
                "Sentry" => Ok(Requirement::EnergySkill(Skill::Sentry, amount)),
                "SentryBreak" => Ok(Requirement::SentryBreak(amount)),
                "SentryBurn" => Ok(Requirement::SentryBurn(amount)),
                "SentryJump" => Ok(Requirement::SentryJump(amount)),
                "SentryRedirect" => Ok(Requirement::SentryRedirect(amount)),
                "SentrySwap" => Ok(Requirement::SentrySwap(amount)),
                "ShardSlot" => Ok(Requirement::Resource(Resource::ShardSlot, amount)),
                "Shuriken" => Ok(Requirement::EnergySkill(Skill::Shuriken, amount)),
                "ShurikenBreak" => Ok(Requirement::ShurikenBreak(amount)),
                "Spear" => Ok(Requirement::EnergySkill(Skill::Spear, amount)),
                "SpearJump" => Ok(Requirement::SpearJump(amount)),
                "SpiritLight" => Ok(Requirement::SpiritLight(amount)),
                "SwordSJump" => Ok(Requirement::SwordSentryJump(amount)),
                _ => Err(wrong_requirement(token))
            }
        }
        None => match keyword {
            "Arcing" => Ok(Requirement::Shard(Shard::Arcing)),
            "Bash" => Ok(Requirement::Skill(Skill::Bash)),
            "Blaze" => Ok(Requirement::Skill(Skill::Blaze)),
            "Bow" => Ok(Requirement::Skill(Skill::Bow)),
            "BreakCrystal" => Ok(Requirement::BreakCrystal),
            "Burrow" => Ok(Requirement::Skill(Skill::Burrow)),
            "BurrowsTP" => Ok(Requirement::Teleporter(Teleporter::Burrows)),
            "Catalyst" => Ok(Requirement::Shard(Shard::Catalyst)),
            "Dash" => Ok(Requirement::Skill(Skill::Dash)),
            "Deflector" => Ok(Requirement::Shard(Shard::Deflector)),
            "DenTP" => Ok(Requirement::Teleporter(Teleporter::Den)),
            "DepthsTP" => Ok(Requirement::Teleporter(Teleporter::Depths)),
            "DoubleJump" => Ok(Requirement::Skill(Skill::DoubleJump)),
            "EastPoolsTP" => Ok(Requirement::Teleporter(Teleporter::EastLuma)),
            "EastWastesTP" => Ok(Requirement::Teleporter(Teleporter::EastWastes)),
            "EastWoodsTP" => Ok(Requirement::Teleporter(Teleporter::EastWoods)),
            "EnergyHarvest" => Ok(Requirement::Shard(Shard::EnergyHarvest)),
            "Flap" => Ok(Requirement::Skill(Skill::Flap)),
            "Flash" => Ok(Requirement::Skill(Skill::Flash)),
            "FlashSwap" => Ok(Requirement::FlashSwap),
            "Fracture" => Ok(Requirement::Shard(Shard::Fracture)),
            "free" => Ok(Requirement::Free),
            "GladesTP" => Ok(Requirement::Teleporter(Teleporter::Glades)),
            "Glide" => Ok(Requirement::Skill(Skill::Glide)),
            "GlideHammerJump" => Ok(Requirement::GlideHammerJump),
            "GlideJump" => Ok(Requirement::GlideJump),
            "gorlek" => Ok(Requirement::Difficulty(Difficulty::Gorlek)),
            "Grapple" => Ok(Requirement::Skill(Skill::Grapple)),
            "Grenade" => Ok(Requirement::Skill(Skill::Grenade)),
            "GrenadeCancel" => Ok(Requirement::GrenadeCancel),
            "GrenadeJump" => Ok(Requirement::GrenadeJump),
            "Hammer" => Ok(Requirement::Skill(Skill::Hammer)),
            "HammerBreak" => Ok(Requirement::HammerBreak),
            "HammerJump" => Ok(Requirement::HammerJump),
            "HollowTP" => Ok(Requirement::Teleporter(Teleporter::Hollow)),
            "Impossible" => Ok(Requirement::Impossible),
            "InnerRuinsTP" => Ok(Requirement::Teleporter(Teleporter::InnerRuins)),
            "kii" => Ok(Requirement::Difficulty(Difficulty::Kii)),
            "Launch" => Ok(Requirement::Skill(Skill::Launch)),
            "LaunchSwap" => Ok(Requirement::LaunchSwap),
            "LifeHarvest" => Ok(Requirement::Shard(Shard::LifeHarvest)),
            "Magnet" => Ok(Requirement::Shard(Shard::Magnet)),
            "MarshTP" => Ok(Requirement::Teleporter(Teleporter::Marsh)),
            "moki" => Ok(Requirement::Difficulty(Difficulty::Moki)),
            "OuterRuinsTP" => Ok(Requirement::Teleporter(Teleporter::OuterRuins)),
            "Overflow" => Ok(Requirement::Shard(Shard::Overflow)),
            "PauseHover" => Ok(Requirement::Glitch(Glitch::PauseHover)),
            "ReachTP" => Ok(Requirement::Teleporter(Teleporter::Reach)),
            "RemoveKillPlane" => Ok(Requirement::Glitch(Glitch::RemoveKillPlane)),
            "Regenerate" => Ok(Requirement::Skill(Skill::Regenerate)),
            "Seir" => Ok(Requirement::Skill(Skill::Seir)),
            "Sentry" => Ok(Requirement::Skill(Skill::Sentry)),
            "ShriekTP" => Ok(Requirement::Teleporter(Teleporter::Shriek)),
            "Shuriken" => Ok(Requirement::Skill(Skill::Shuriken)),
            "Spear" => Ok(Requirement::Skill(Skill::Spear)),
            "SpearBreak" => Ok(Requirement::SpearBreak),
            "Sticky" => Ok(Requirement::Shard(Shard::Sticky)),
            "Sword" => Ok(Requirement::Skill(Skill::Sword)),
            "SwordJump" => Ok(Requirement::SwordJump),
            "TripleJump" => Ok(Requirement::Shard(Shard::TripleJump)),
            "Thorn" => Ok(Requirement::Shard(Shard::Thorn)),
            "UltraBash" => Ok(Requirement::Shard(Shard::UltraBash)),
            "UltraGrapple" => Ok(Requirement::Shard(Shard::UltraGrapple)),
            "unsafe" => Ok(Requirement::Difficulty(Difficulty::Unsafe)),
            "WallJump" => Ok(Requirement::Skill(Skill::WallJump)),
            "WaterBreath" => Ok(Requirement::Skill(Skill::WaterBreath)),
            "WaterDash" => Ok(Requirement::Skill(Skill::WaterDash)),
            "Water" => Ok(Requirement::Water),
            "WaveDash" => Ok(Requirement::WaveDash),
            "WellspringTP" => Ok(Requirement::Teleporter(Teleporter::Wellspring)),
            "WestPoolsTP" => Ok(Requirement::Teleporter(Teleporter::WestLuma)),
            "WestWastesTP" => Ok(Requirement::Teleporter(Teleporter::WestWastes)),
            "WestWoodsTP" => Ok(Requirement::Teleporter(Teleporter::WestWoods)),
            "WillowTP" => Ok(Requirement::Teleporter(Teleporter::Willow)),
            _ if metadata.definitions.contains(keyword) => Ok(Requirement::Definition(keyword)),
            _ if metadata.states.contains(keyword) || metadata.quests.contains(keyword) => Ok(Requirement::State(keyword)),
            "BlazeSwap" | "Boss" | "BreakWall" | "Combat" | "Damage" | "Danger" | "Energy" | "GrenadeRedirect" | "Health" | "Keystone" | "Ore" | "SentryBreak" | "SentryBurn" | "SentryJump"| "SentryRedirect" | "SentrySwap" | "SwordSJump" | "HammerSJump" | "ShardSlot" | "ShurikenBreak" | "SpiritLight"
                => Err(wrong_amount(token)),
            _ => Err(wrong_requirement(token))
        }
    }
}

fn parse_line<'a>(tokens: &[Token<'a>], position: &mut usize, metadata: &Metadata) -> Result<Line<'a>, ParseError> {
    let mut ands = Vec::new();
    let mut ors = Vec::new();
    let mut group = None;
    loop {
        let token = next_token!(tokens, *position, TokenType::Requirement, TokenType::Free);
        let requirement = match token.name {
            TokenType::Requirement => parse_requirement(token, metadata)?,
            TokenType::Free => Requirement::Free,
            _ => wrong_token!(token, TokenType::Requirement, TokenType::Free),
        };

        let token = next_token!(tokens, *position, TokenType::And, TokenType::Or, TokenType::Newline, TokenType::Group);
        match token.name {
            TokenType::And => ands.push(requirement),
            TokenType::Or => ors.push(requirement),
            TokenType::Newline => {
                ors.push(requirement);
                break;
            },
            TokenType::Group => {
                if tokens.get(*position).map_or(false, |token| token.name == TokenType::Requirement) {
                    ands.push(requirement);
                } else {
                    ors.push(requirement);
                    group = Some(parse_group(tokens, position, metadata)?);
                    break;
                }
            },
            _ => wrong_token!(token, TokenType::And, TokenType::Or, TokenType::Newline, TokenType::Group),
        }
    }

    Ok(Line { ands, ors, group })
}

fn parse_group<'a>(tokens: &[Token<'a>], position: &mut usize, metadata: &Metadata) -> Result<Group<'a>, ParseError> {
    let mut lines = Vec::new();

    let token = next_token!(tokens, *position, TokenType::Free, TokenType::Indent);
    match token.name {
        TokenType::Free => {
            eat(tokens, position, TokenType::Newline)?;
            lines.push(Line {
                ands: vec![Requirement::Free],
                ..Line::default()
            });
        },
        TokenType::Indent => {
            loop {
                lines.push(parse_line(tokens, position, metadata)?);
                if tokens.get(*position).map_or(true, |token| token.name == TokenType::Dedent) {
                    *position += 1;
                    break;
                }
            }
        }
        _ => wrong_token!(token, TokenType::Free, TokenType::Indent),
    }

    Ok(Group { lines })
}

fn parse_refill<'a>(tokens: &[Token<'a>], position: &mut usize, identifier: &str, metadata: &Metadata) -> Result<Refill<'a>, ParseError> {
    let mut requirements = None;

    let token = next_token!(tokens, *position, TokenType::Group, TokenType::Newline);
    match token.name {
        TokenType::Group => requirements = Some(parse_group(tokens, position, metadata)?),
        TokenType::Newline => {},
        _ => wrong_token!(token, TokenType::Group, TokenType::Newline),
    }

    let name = if identifier == "Checkpoint" {
        RefillType::Checkpoint
    } else if identifier == "Full" {
        RefillType::Full
    } else if let Some(amount) = identifier.strip_prefix("Health=") {
        let amount: u16 = match amount.parse() {
            Ok(result) => result,
            Err(_) => return Err(not_int(&tokens[*position - 2])),
        };
        RefillType::Health(f32::from(amount))
    } else if identifier == "Health" {
        RefillType::Health(1.0)
    } else if let Some(amount) = identifier.strip_prefix("Energy=") {
        let amount: u16 = match amount.parse() {
            Ok(result) => result,
            Err(_) => return Err(not_int(&tokens[*position - 2])),
        };
        RefillType::Energy(f32::from(amount))
    } else {
        wrong_token!(tokens[*position - 2], "Checkpoint", "Full", "Health", "Energy");
    };

    Ok(Refill {
        name,
        requirements,
    })
}
#[inline]
fn parse_connection<'a>(tokens: &[Token<'a>], position: &mut usize, identifier: &'a str, metadata: &Metadata, name: NodeType) -> Result<Connection<'a>, ParseError> {
    eat(tokens, position, TokenType::Group)?;
    let requirements = parse_group(tokens, position, metadata)?;

    Ok(Connection { name, identifier, requirements })
}
fn parse_state<'a>(tokens: &[Token<'a>], position: &mut usize, identifier: &'a str, metadata: &Metadata) -> Result<Connection<'a>, ParseError> {
    parse_connection(tokens, position, identifier, metadata, NodeType::State)
}
fn parse_quest<'a>(tokens: &[Token<'a>], position: &mut usize, identifier: &'a str, metadata: &Metadata) -> Result<Connection<'a>, ParseError> {
    parse_connection(tokens, position, identifier, metadata, NodeType::Quest)
}
fn parse_pickup<'a>(tokens: &[Token<'a>], position: &mut usize, identifier: &'a str, metadata: &Metadata) -> Result<Connection<'a>, ParseError> {
    parse_connection(tokens, position, identifier, metadata, NodeType::Pickup)
}
fn parse_anchor_connection<'a>(tokens: &[Token<'a>], position: &mut usize, identifier: &'a str, metadata: &Metadata) -> Result<Connection<'a>, ParseError> {
    parse_connection(tokens, position, identifier, metadata, NodeType::Anchor)
}

fn parse_anchor<'a>(tokens: &[Token<'a>], position: &mut usize, identifier: &'a str, metadata: &Metadata) -> Result<Anchor<'a>, ParseError> {
    let mut token = next_token!(tokens, *position, TokenType::Position, TokenType::Group);

    let mut anchor_position = None;
    if token.name == TokenType::Position {
        let mut coords = token.value.split(',');
        let x = coords.next().unwrap().trim().parse::<R32>().map_err(|_| not_int(token))?;
        let y = if let Some(y) = coords.next() {
            y.trim().parse::<R32>().map_err(|_| not_int(token))?
        } else {
            return Err(not_int(token));
        };
        anchor_position = Some(Position { x, y });

        if let Some(next) = tokens.get(*position) {
            *position += 1;
            token = next;
        } else {
            missing_token!(TokenType::Group)
        }
    }

    if token.name != TokenType::Group {
        wrong_token!(token, TokenType::Group);
    }
    eat(tokens, position, TokenType::Indent)?;

    token = next_token!(tokens, *position, TokenType::Refill, TokenType::State, TokenType::Quest, TokenType::Pickup, TokenType::Connection, TokenType::NoSpawn, TokenType::Dedent);
    let can_spawn = if token.name == TokenType::NoSpawn {
        eat(tokens, position, TokenType::Newline)?;
        token = next_token!(tokens, *position, TokenType::Refill, TokenType::State, TokenType::Quest, TokenType::Pickup, TokenType::Connection, TokenType::Dedent);
        false
    } else { true };

    let mut refills = Vec::new();
    let mut connections = Vec::new();

    loop {
        match token.name {
            TokenType::Refill => refills.push(parse_refill(tokens, position, token.value, metadata)?),
            TokenType::State => connections.push(parse_state(tokens, position, token.value, metadata)?),
            TokenType::Quest => connections.push(parse_quest(tokens, position, token.value, metadata)?),
            TokenType::Pickup => connections.push(parse_pickup(tokens, position, token.value, metadata)?),
            TokenType::Connection => connections.push(parse_anchor_connection(tokens, position, token.value, metadata)?),
            TokenType::Dedent => return Ok(Anchor { identifier, position: anchor_position, can_spawn, refills, connections }),
            _ => wrong_token!(token, TokenType::Refill, TokenType::State, TokenType::Quest, TokenType::Pickup, TokenType::Connection, TokenType::Dedent),
        }
        token = next_token!(tokens, *position, TokenType::Refill, TokenType::State, TokenType::Quest, TokenType::Pickup, TokenType::Connection, TokenType::Dedent);
    }
}

pub fn parse_areas<'a>(tokens: Vec<Token<'a>>, metadata: &Metadata) -> Result<AreaTree<'a>, ParseError> {
    let end = tokens.len();
    let mut definitions = FxHashMap::default();
    let mut regions = FxHashMap::default();
    regions.reserve(20);
    let mut anchors = Vec::with_capacity(end / 200);

    let mut position = 0;

    while let Some(token) = tokens.get(position) {
        position += 1;
        match token.name {
            TokenType::Definition => {
                eat(&tokens, &mut position, TokenType::Group)?;
                let requirements = parse_group(&tokens, &mut position, metadata)?;
                if definitions.insert(token.value, requirements).is_some() {
                    return Err(ParseError::new(token.position, format!("Requirement name {} already in use at line {}", token.value, token.line)));
                }
            },
            TokenType::Region => {
                eat(&tokens, &mut position, TokenType::Group)?;
                let requirements = parse_group(&tokens, &mut position, metadata)?;
                if regions.insert(token.value, requirements).is_some() {
                    return Err(ParseError::new(token.position, format!("Region name {} already in use at line {}", token.value, token.line)));
                }
            },
            TokenType::Anchor => anchors.push(parse_anchor(&tokens, &mut position, token.value, metadata)?),
            TokenType::Newline => {},
            _ => wrong_token!(token, TokenType::Definition, TokenType::Anchor),
        }
    }

    Ok(AreaTree {
        definitions,
        regions,
        anchors,
    })
}
