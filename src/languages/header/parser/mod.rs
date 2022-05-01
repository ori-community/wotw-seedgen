mod preprocess;
mod postprocess;
mod parse_item;
mod header_command;

pub(super) use preprocess::preprocess;
pub use postprocess::postprocess;
use seedgen_derive::FromStr;

use std::{str::FromStr, iter::Peekable, ops::Range, fmt, error::Error, slice::SliceIndex};

use crate::{
    VItem,
    util::{VUberState, extensions::StrExtension, UberIdentifier, Icon},
};

use super::{HeaderCommand, Annotation, HeaderContent, TimerDefinition, VPickup, V, VString, tokenizer::{TokenStream, Token, TokenKind, tokenize, CommentKind}};

pub struct Parser<'a> {
    input: &'a str,
    tokens: Peekable<TokenStream<'a>>,
    current_token: Token,
    eof_token: Token,
}

impl<'a> Parser<'a> {
    /// Returns a new [`Parser`] for the input string
    pub(crate) fn new(input: &'a str) -> Self {
        let mut tokens = tokenize(input).peekable();
        let len = input.len();
        let eof_token = Token { kind: TokenKind::Eof, range: len..len };
        let current_token = tokens.next().unwrap_or_else(|| eof_token.clone());
        Self { input, tokens, current_token, eof_token }
    }

    /// Expects the current [`Token`] to match the [`TokenKind`], returns it and steps to the next [`Token`]
    pub(crate) fn eat(&mut self, kind: TokenKind) -> Result<Token, ParseError> {
        let token = self.next_token();
        expect!(token, kind);
        Ok(token)
    }
    /// If the current [`Token`] matches the [`TokenKind`], steps to the next [`Token`]
    pub(crate) fn skip(&mut self, kind: TokenKind) {
        if self.current_token.kind == kind {
            self.next_token();
        }
    }
    /// Skips [`Token`]s as long as they fulfill a condition
    pub(crate) fn skip_while(&mut self, condition: fn(TokenKind) -> bool) {
        while condition(self.current_token.kind) {
            let token = self.next_token();
            if token.kind == TokenKind::Eof { break }
        }
    }

    /// Returns the current [`Token`], or an end-of-file [`Token`] if no [`Token`]s are left and steps to the next [`Token`]
    pub(crate) fn next_token(&mut self) -> Token {
        let next = self.tokens.next().unwrap_or_else(|| self.eof_token.clone());
        std::mem::replace(&mut self.current_token, next)
    }
    /// Returns a reference to the current [`Token`], or an end-of-file [`Token`] if no [`Token`]s are left
    pub(crate) fn current_token(&self) -> &Token {
        &self.current_token
    }
    /// Returns the next [`Token`] without committing to step forwards
    pub(crate) fn peek_token(&mut self) -> &Token {
        self.tokens.peek().unwrap_or(&self.eof_token)
    }

    /// Returns the string corresponding to a [`Token`]
    pub(crate) fn read_token(&self, token: &Token) -> &str {
        &self.input[token.range.clone()]
    }
    /// Returns the string corresponding to the index
    pub(crate) fn read<I: SliceIndex<str>>(&self, index: I) -> &I::Output {
        &self.input[index]
    }

    /// Returns the remaining portion of the input string
    pub(crate) fn remaining(&self) -> &str {
        let start = self.current_token.range.start;
        &self.input[start..]
    }

    /// Returns a multiline visual representation of a [`ParseError`]
    /// 
    /// # Panics
    /// 
    /// Panics if the [`ParseError`] is out of bounds of the [`Parser`]s input string
    pub(crate) fn error_display(&self, err: ParseError) -> String {
        if self.input.is_empty() { return format!("{}\n(input was empty)", err.message) }
        let (line_number, line_range) = self.input.line_ranges()
            .enumerate()
            .find(|(_, line_range)| line_range.end >= err.range.end)
            .expect("Error range out of bounds");
        let message = err.message;
        let line_number = format!("line {}", line_number + 1);
        let indent = " ".repeat(line_number.len());
        let err_offset = " ".repeat(err.range.start - line_range.start);
        let err_underline = "^".repeat(err.range.len());
        let line = &self.input[line_range];
        let newline = if line.ends_with('\n') { "" } else { "\n" };
        format!("\
            {message}\n\
            {line_number}: {line}{newline}\
            {indent}  {err_offset}{err_underline}\
        ")
    }
}

#[derive(Debug)]
pub struct ParseError {
    message: String,
    range: Range<usize>,
}
impl ParseError {
    fn new(message: impl AsRef<str>, range: Range<usize>) -> Self {
        let message = message.as_ref().to_string();
        Self { message, range }
    }
}
impl fmt::Display for ParseError {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(fmt, "{}", self.message)
    }
}
impl Error for ParseError {}

#[derive(FromStr)]
#[ParseFromIdentifier]
enum InterpolationCommand {
    PARAM,
}

macro_rules! expect {
    ($token:ident, $expected:expr) => {
        if $token.kind != $expected {
            return Err($crate::header::parser::ParseError::new(format!("Expected {}", $expected), $token.range));
        }
    };
}
use expect;
macro_rules! invalid {
    ($token:ident, $expected:expr) => {
        |err| $crate::header::parser::ParseError::new(format!("Invalid {}: {}", $expected, err), $token.range)
    };
}
use invalid;

macro_rules! parse_token {
    ($token_kind:path, $parser:expr, $description:expr) => {
        {
            let token = $parser.eat($token_kind)?;
            let string = $parser.read_token(&token);
            string.parse().map_err($crate::header::parser::invalid!(token, $description))?
        }
    };
}
use parse_token;
macro_rules! parse_number {
    ($parser:expr, $description:expr) => {
        $crate::header::parser::parse_token!($crate::header::tokenizer::TokenKind::Number, $parser, $description)
    };
}
use parse_number;
macro_rules! parse_removable_number {
    ($parser:expr, $description:expr) => {
        {
            let token = $parser.eat($crate::header::tokenizer::TokenKind::Number)?;
            let mut string = $parser.read_token(&token);
            let remove = if let Some(remove) = string.strip_prefix('-') {
                string = remove;
                true
            } else { false };
            let parsed = string.parse().map_err($crate::header::parser::invalid!(token, $description))?;
            (parsed, remove)
        }
    };
}
use parse_removable_number;
macro_rules! parse_value {
    ($parser:expr, $description:expr) => {
        {
            $parser.eat($crate::header::tokenizer::TokenKind::Eq)?;
            $crate::header::parser::parse_number!($parser, $description)
        }
    };
}
use parse_value;
macro_rules! parse_ident {
    ($parser:expr, $description:expr) => {
        $crate::header::parser::parse_token!($crate::header::tokenizer::TokenKind::Ident, $parser, $description)
    };
}
use parse_ident;
fn parse_string<'a>(parser: &'a mut Parser) -> Result<&'a str, ParseError> {
    let token = parser.next_token();
    if let TokenKind::String { terminated } = token.kind {
        if !terminated {
            return Err(ParseError::new("Unterminated string", token.range));
        }
    } else {
        return Err(ParseError::new("Expected string", token.range));
    }
    let content_range = token.range.start + 1..token.range.end - 1;
    Ok(parser.read(content_range))
}
fn parse_v_param<T: FromStr>(parser: &mut Parser) -> Result<V<T>, ParseError> {
    parser.eat(TokenKind::OpenParen)?;
    let token = parser.eat(TokenKind::Ident)?;
    let param = parser.read_token(&token).to_owned();
    parser.eat(TokenKind::CloseParen)?;
    Ok(V::Parameter(param))
}
macro_rules! parse_v {
    ($parser:expr, $token:ident, $description:expr) => {
        match $token.kind {
            $crate::header::tokenizer::TokenKind::Dollar => {
                let command = $crate::header::parser::parse_ident!($parser, "interpolation command");
                match command {
                    $crate::header::parser::InterpolationCommand::PARAM => $crate::header::parser::parse_v_param($parser)?,
                }
            },
            _ => return Err($crate::header::parser::ParseError::new(format!("Expected {}", $description), $token.range)),
        }
    };
}
use parse_v;
macro_rules! parse_v_or_kind {
    ($parser:expr, $description:expr, $kind:pat) => {
        {
            let token = $parser.next_token();
            match token.kind {
                $kind => {
                    let string = $parser.read_token(&token);
                    let literal = string.parse().map_err($crate::header::parser::invalid!(token, $description))?;
                    $crate::header::V::Literal(literal)
                },
                _ => $crate::header::parser::parse_v!($parser, token, $description)
            }
        }
    };
}
use parse_v_or_kind;
macro_rules! parse_v_ident {
    ($parser:expr, $description:expr) => {
        $crate::header::parser::parse_v_or_kind!($parser, $description, $crate::header::tokenizer::TokenKind::Ident)
    };
}
use parse_v_ident;
macro_rules! parse_v_number {
    ($parser:expr, $description:expr) => {
        $crate::header::parser::parse_v_or_kind!($parser, $description, $crate::header::tokenizer::TokenKind::Number)
    };
}
use parse_v_number;
macro_rules! parse_v_removable_number {
    ($parser:expr, $description:expr) => {
        {
            let token = $parser.next_token();
            match token.kind {
                $crate::header::tokenizer::TokenKind::Number => {
                    let mut string = $parser.read_token(&token);
                    let remove = if let Some(remove) = string.strip_prefix('-') {
                        string = remove;
                        true
                    } else { false };
                    let parsed = string.parse().map_err($crate::header::parser::invalid!(token, $description))?;
                    (V::Literal(parsed), remove)
                },
                $crate::header::tokenizer::TokenKind::Minus => {
                    ($crate::header::parser::parse_v_number!($parser, $description), true)
                },
                _ => ($crate::header::parser::parse_v!($parser, token, $description), false)
            }
        }
    };
}
use parse_v_removable_number;

fn trim_comment(input: &str) -> &str {
    input.find("//").map_or(input, |index| &input[..index]).trim_end()
}

fn parse_uber_identifier(parser: &mut Parser) -> Result<UberIdentifier, ParseError> {
    let uber_group = parse_number!(parser, "uber group");
    parser.eat(TokenKind::Separator)?;
    let uber_id = parse_number!(parser, "uber id");
    Ok(UberIdentifier { uber_group, uber_id })
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
    let kind = parse_ident!(parser, "icon kind");
    parser.eat(TokenKind::Colon)?;
    let icon = if kind == IconKind::File {
        let path = parse_string(parser)?.to_owned();
        Icon::File(path)
    } else {
        let id = parse_number!(parser, "icon identifier");
        match kind {
            IconKind::Shard => Icon::Shard(id),
            IconKind::Spell => Icon::Spell(id),
            IconKind::Opher => Icon::Opher(id),
            IconKind::Lupo => Icon::Lupo(id),
            IconKind::Grom => Icon::Grom(id),
            IconKind::Tuley => Icon::Tuley(id),
            IconKind::File => unreachable!(),
        }
    };
    Ok(icon)
}

/// Returns the annotations of a header and removes them from the input string
pub(super) fn parse_annotations(input: &mut String) -> Result<Vec<Annotation>, String> {
    let mut annotations = vec![];
    let mut after_annotations = input.len();

    for range in input.line_ranges() {
        let range_start = range.start;
        let line = &input[range];
        if let Some(annotation) = line.strip_prefix('#') {
            let annotation = trim_comment(annotation);
            let annotation = Annotation::from_str(annotation)?;
            annotations.push(annotation);
        } else if !line.is_empty() {
            after_annotations = range_start;
            break;
        }
    }

    input.replace_range(0..after_annotations, "");

    Ok(annotations)
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
pub(super) fn parse_header_contents(parser: &mut Parser) -> Result<Vec<HeaderContent>, Vec<ParseError>> {
    let mut context = ParseContext::new(parser);
    let mut errors = vec![];

    loop {
        parse_whitespace(&mut context);
        if context.parser.current_token().kind == TokenKind::Eof { break }
        match parse_expression(&mut context) {
            Ok(content) => {
                context.contents.push(content);
                context.parser.skip_while(|kind| matches!(kind, TokenKind::Whitespace | TokenKind::Comment { .. }));
                if context.parser.current_token().kind == TokenKind::Eof { break }
                if let Err(err) = context.parser.eat(TokenKind::Newline) {
                    recover(err, &mut errors, &mut context.parser);
                }
            },
            Err(err) => recover(err, &mut errors, &mut context.parser),
        }
        context.skip_validation = false;
    }

    if errors.is_empty() {
        Ok(context.contents)
    } else {
        Err(errors)
    }
}

fn recover(err: ParseError, errors: &mut Vec<ParseError>, parser: &mut Parser) {
    errors.push(err);
    parser.skip_while(|kind| kind != TokenKind::Newline);
    parser.next_token();
}

fn parse_whitespace(context: &mut ParseContext) {
    loop {
        let current_token = context.parser.current_token();
        match current_token.kind {
            TokenKind::Newline | TokenKind::Whitespace => {},
            TokenKind::Comment { kind } => {
                let comment = context.parser.read_token(current_token);
                match kind {
                    CommentKind::Note => if comment[2..].trim() == "skip-validate" { context.skip_validation = true },
                    CommentKind::HeaderDoc => context.contents.push(HeaderContent::OuterDocumentation(comment[3..].trim().to_owned())),
                    CommentKind::ConfigDoc => context.contents.push(HeaderContent::InnerDocumentation(comment[4..].trim().to_owned()))
                }
            },
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
    let parser = &mut context.parser;
    let current_token = parser.current_token();
    match current_token.kind {
        TokenKind::Ident => {
            let kind = parse_ident!(parser, "expression");
            parser.eat(TokenKind::Colon)?;
            parser.skip(TokenKind::Whitespace);
            match kind {
                ExpressionIdentKind::Flags => parse_flags(parser),
                ExpressionIdentKind::Timer => parse_timer(parser),
            }
        },
        TokenKind::Bang => {
            parser.next_token();
            if parser.current_token().kind == TokenKind::Bang {
                parser.next_token();
                parse_command(parser)
            } else {
                parse_pickup(context, true)
            }
        },
        TokenKind::Number | TokenKind::Dollar => parse_pickup(context, false),
        _ => Err(ParseError::new("expected expression", current_token.range.clone())),
    }
}
fn parse_flags(parser: &mut Parser) -> Result<HeaderContent, ParseError> {
    let mut flags = vec![];

    loop {
        let flag = VString(parse_string(parser)?.to_owned());
        flags.push(flag);

        if parser.current_token().kind == TokenKind::Comma {
            parser.next_token();
            parser.skip(TokenKind::Whitespace);
        } else { break }
    }

    Ok(HeaderContent::Flags(flags))
}
fn parse_timer(parser: &mut Parser) -> Result<HeaderContent, ParseError> {
    let toggle = parse_uber_identifier(parser)?;
    parser.eat(TokenKind::Separator)?;
    let timer = parse_uber_identifier(parser)?;

    let timer_definition = TimerDefinition { toggle, timer };
    Ok(HeaderContent::Timer(timer_definition))
}
fn parse_command(parser: &mut Parser) -> Result<HeaderContent, ParseError> {
    HeaderCommand::parse(parser).map(HeaderContent::Command)
}
fn parse_pickup(context: &mut ParseContext, ignore: bool) -> Result<HeaderContent, ParseError> {
    let parser = &mut context.parser;

    let identifier = parse_uber_identifier(parser)?;
    let value = if parser.current_token().kind == TokenKind::Eq {
        parser.next_token();
        parse_v_number!(parser, "")
    } else { V::Literal(String::new()) };
    let trigger = VUberState { identifier, value };

    parser.eat(TokenKind::Separator)?;

    let item = VItem::parse(parser)?;
    let skip_validation = context.skip_validation;

    let pickup = VPickup { trigger, item, ignore, skip_validation };
    Ok(HeaderContent::Pickup(pickup))
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use crate::item::*;
    use crate::util::*;

    #[test]
    fn item_parsing() {
        assert_eq!(Item::from_str("0|5000"), Ok(Item::SpiritLight(5000)));
        assert_eq!(Item::from_str("0|-5000"), Ok(Item::RemoveSpiritLight(5000)));
        assert_eq!(Item::from_str("1|2"), Ok(Item::Resource(Resource::Ore)));
        assert!(Item::from_str("1|-2").is_err());
        assert!(Item::from_str("1|5").is_err());
        assert_eq!(Item::from_str("2|8"), Ok(Item::Skill(Skill::Launch)));
        assert_eq!(Item::from_str("2|120"), Ok(Item::Skill(Skill::AncestralLight1)));
        assert_eq!(Item::from_str("2|121"), Ok(Item::Skill(Skill::AncestralLight2)));
        assert!(Item::from_str("2|25").is_err());
        assert!(Item::from_str("2|-9").is_err());
        assert_eq!(Item::from_str("3|28"), Ok(Item::Shard(Shard::LastStand)));
        assert_eq!(Item::from_str("5|16"), Ok(Item::Teleporter(Teleporter::Marsh)));
        assert_eq!(Item::from_str("9|0"), Ok(Item::Water));
        assert_eq!(Item::from_str("9|-0"), Ok(Item::RemoveWater));
        assert_eq!(Item::from_str("11|0"), Ok(Item::BonusUpgrade(BonusUpgrade::RapidHammer)));
        assert_eq!(Item::from_str("10|31"), Ok(Item::BonusItem(BonusItem::EnergyRegeneration)));
        assert!(Item::from_str("8|5|3|6").is_err());
        assert!(Item::from_str("8||||").is_err());
        assert!(Item::from_str("8|5|3|in|3").is_err());
        assert!(Item::from_str("8|5|3|bool|3").is_err());
        assert!(Item::from_str("8|5|3|float|hm").is_err());
        assert_eq!(Item::from_str("8|5|3|int|6"), Ok(UberState::from_parts("5", "3=6").unwrap().to_item(UberType::Int)));
        assert_eq!(Item::from_str("4|0"), Ok(Item::Command(Command::Autosave)));
        assert!(Item::from_str("12").is_err());
        assert!(Item::from_str("").is_err());
        assert!(Item::from_str("0|").is_err());
        assert!(Item::from_str("0||400").is_err());
        assert!(Item::from_str("7|3").is_err());
        assert!(Item::from_str("-0|65").is_err());
    }
}
