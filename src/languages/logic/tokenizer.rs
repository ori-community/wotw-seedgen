use std::cmp::Ordering;

use rustc_hash::FxHashSet;
use smallvec::SmallVec;

#[derive(Debug, seedgen_derive::Display, Copy, Clone, PartialEq)]
pub enum TokenType {
    Whitespace,
    Definition,
    Region,
    Anchor,
    Position,
    Indent,
    Dedent,
    Newline,
    Refill,
    State,
    Quest,
    Pickup,
    Connection,
    Requirement,
    Free,
    Group,
    And,
    Or,
    NoSpawn,
}

#[derive(Debug, Clone)]
pub struct Token<'a> {
    pub name: TokenType,
    pub value: &'a str,
    pub line: usize,
    pub position: usize,
}
impl<'a> Token<'a> {
    #[inline]
    fn blank(name: TokenType, context: &TokenContext) -> Token<'a> {
        Token::value(name, "", context)
    }
    #[inline]
    fn value(name: TokenType, value: &'a str, context: &TokenContext) -> Token<'a> {
        Token {
            name,
            value,
            line: context.line,
            position: context.position,
        }
    }
}

struct TokenContext<'a> {
    input: &'a str,
    metadata: Metadata<'a>,
    line: usize,
    position: usize,
    indent_stack: SmallVec<[usize; 8]>,
}
impl<'a> TokenContext<'a> {
    #[inline]
    fn new(input: &'a str) -> Self {
        TokenContext {
            input,
            metadata: Metadata::default(),
            line: 1,
            position: 0,
            indent_stack: SmallVec::new(),
        }
    }

    #[inline]
    fn progress(&mut self, offset: usize, next: &'a str) {
        self.input = next;
        self.position += offset;
    }
}

#[derive(Debug, Default)]
pub struct Metadata<'a> {
    pub definitions: FxHashSet<&'a str>,
    pub states: FxHashSet<&'a str>,
    pub quests: FxHashSet<&'a str>,
}

#[inline]
fn tokenize_whitespace<'a>(tokens: &mut Vec<Token<'a>>, context: &mut TokenContext<'a>) {
    let mut comment = false;
    let mut indent = 0;
    let mut lines = 0;
    let mut next = "";
    let mut offset = 0;

    for (index, char) in context.input.char_indices() {
        if char == '\n' {
            lines += 1; indent = 0;
            comment = false;
        } else if comment {
            // pass
        } else if char == '#' {
            comment = true;
        } else if char == ' ' {
            indent += 1;
        } else if !char.is_whitespace() {
            next = &context.input[index..];
            offset = index;
            break;
        }
    }

    if lines > 0 {
        let mut last_indent = context.indent_stack.last().cloned().unwrap_or(0);
        match indent.cmp(&last_indent) {
            Ordering::Equal => tokens.push(Token::blank(TokenType::Newline, context)),
            Ordering::Greater => {
                context.indent_stack.push(indent);
                tokens.push(Token::blank(TokenType::Indent, context));
            },
            Ordering::Less => {
                tokens.push(Token::blank(TokenType::Newline, context));
                while indent < last_indent {
                    context.indent_stack.pop();
                    last_indent = context.indent_stack.last().cloned().unwrap_or(0);

                    tokens.push(Token::blank(TokenType::Dedent, context));
                }
                if indent > last_indent {
                    tokens.push(Token::blank(TokenType::Indent, context));
                }
            },
        };
    }

    context.progress(offset, next);
    context.line += lines;
}

#[inline]
fn tokenize_blank<'a>(pattern: &str, name: TokenType, context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    context.input.strip_prefix(pattern).map(|next| {
        let token = Token::blank(name, context);
        context.progress(pattern.len(), next);
        token
    })
}
#[inline]
fn tokenize_by_delimiter<'a>(name: TokenType, delimit_pattern: fn(char) -> bool, context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    let offset = context.input.find(delimit_pattern).unwrap_or_else(|| context.input.len());
    let token = Token::value(name, &context.input[..offset], context);
    context.progress(offset, &context.input[offset..]);
    Some(token)
}
#[inline]
fn tokenize_named_key<'a>(keyword: &str, name: TokenType, delimit_pattern: Option<fn(char) -> bool>, context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    context.input.strip_prefix(keyword).and_then(|next| {
        context.progress(keyword.len(), next);

        let delimit_pattern = delimit_pattern.unwrap_or(|char: char| char == ':' || char.is_whitespace());
        tokenize_by_delimiter(name, delimit_pattern, context)
    })
}

fn tokenize_and<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_blank(",", TokenType::And, context)
}
fn tokenize_or<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_blank("OR ", TokenType::Or, context)
}
fn tokenize_group<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_blank(":", TokenType::Group, context)
}
fn tokenize_connection<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("conn ", TokenType::Connection, None, context)
}
fn tokenize_pickup<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("pickup ", TokenType::Pickup, None, context)
}
fn tokenize_refill<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("refill ", TokenType::Refill, None, context)
}
fn tokenize_anchor<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("anchor ", TokenType::Anchor, None, context)
}
fn tokenize_position<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("at ", TokenType::Position, Some(|char: char| char == ':' || char == '\n'), context)
}
fn tokenize_state<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("state ", TokenType::State, None, context)
}
fn tokenize_quest<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("quest ", TokenType::Quest, None, context)
}
fn tokenize_region<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("region ", TokenType::Region, None, context)
}
fn tokenize_definition<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_named_key("requirement ", TokenType::Definition, None, context)
}
fn tokenize_free<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_blank("free", TokenType::Free, context)
}
fn tokenize_nospawn<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_blank("nospawn", TokenType::NoSpawn, context)
}
fn tokenize_requirement<'a>(context: &mut TokenContext<'a>) -> Option<Token<'a>> {
    tokenize_by_delimiter(TokenType::Requirement, |c: char| c.is_whitespace() || c == ',' || c == ':' || c == '#', context)
}

const TOKENIZERS: [for<'a> fn(&mut TokenContext<'a>) -> Option<Token<'a>>; 15] = [
    tokenize_and,           // 8511 occurences
    tokenize_or,            // 5676
    tokenize_group,         // 4301
    tokenize_connection,    // 601
    tokenize_pickup,        // 410
    tokenize_free,          // 286
    tokenize_refill,        // 270
    tokenize_anchor,        // 242
    tokenize_position,      // 238
    tokenize_state,         // 148
    tokenize_quest,         // 47
    tokenize_region,        // 20
    tokenize_definition,    // 1
    tokenize_nospawn,       // 0
    tokenize_requirement,
];

pub fn tokenize(input: &str) -> Result<(Vec<Token>, Metadata), String> {
    let mut context = TokenContext::new(input);
    let mut tokens = Vec::with_capacity(input.len() / 9);

    'outer: loop {
        tokenize_whitespace(&mut tokens, &mut context);
        if context.input.is_empty() { break }

        for tokenizer in &TOKENIZERS {
            if let Some(token) = tokenizer(&mut context) {
                match token.name {
                    TokenType::Definition => { context.metadata.definitions.insert(token.value); }
                    TokenType::State => { context.metadata.states.insert(token.value); }
                    TokenType::Quest => { context.metadata.quests.insert(token.value); }
                    _ => {}
                }
                tokens.push(token);
                continue 'outer;
            }
        }
        return Err(format!("Failed to read line {}: {}", context.line, context.input.lines().next().unwrap_or("")));
    }

    Ok((tokens, context.metadata))
}
