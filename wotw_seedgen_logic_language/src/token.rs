use logos::Logos;
use std::{cmp::Ordering, iter};
use wotw_seedgen_parse::{
    ParseFloatToken, ParseIdentToken, ParseIntToken, TokenDisplay, Tokenize, TokenizeOutput,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Logos, TokenDisplay)]
#[logos(skip r"[ \r]+|#[^\n]*")]
pub enum Token {
    #[regex(r"\n([ \r\n]+|#[^\n]*)*")]
    Newline,
    Indent,
    Dedent,
    #[token("requirement")]
    Requirement,
    #[token("region")]
    Region,
    #[token("anchor")]
    Anchor,
    #[token("at")]
    At,
    #[token("door")]
    Door,
    #[token("id")]
    Id,
    #[token("target")]
    Target,
    #[token("enter")]
    Enter,
    #[token("nospawn")]
    NoSpawn,
    #[token("tprestriction")]
    TpRestriction,
    #[token("refill")]
    Refill,
    #[token("state")]
    State,
    #[token("quest")]
    Quest,
    #[token("pickup")]
    Pickup,
    #[token("conn")]
    Connection,
    #[token("OR")]
    Or,
    #[regex(r"[_a-vyzA-Z]\w*")]
    Identifier,
    #[regex(r"[_a-vyzA-Z]\w*\.\w+")]
    LogicIdentifier,
    #[regex(r"-?\d+\.\d*")]
    Float,
    #[regex(r"-?\d+")]
    Integer,
    #[regex(r".", priority = 0)]
    Symbol,
    Eof,
}

impl ParseIntToken for Token {
    #[inline]
    fn is_int(&self) -> bool {
        matches!(self, Token::Integer)
    }
}
impl ParseFloatToken for Token {
    #[inline]
    fn is_float(&self) -> bool {
        matches!(self, Token::Float | Token::Integer)
    }
}
impl ParseIdentToken for Token {
    #[inline]
    fn is_ident(&self) -> bool {
        matches!(self, Token::Identifier)
    }
}

pub struct Tokenizer;
impl Tokenize for Tokenizer {
    type Token = Token;

    fn tokenize(self, source: &str) -> TokenizeOutput<Self::Token> {
        let mut indent_stack = vec![];
        let mut tokens = vec![]; // TODO capacity guess based on source length?

        for (token, span) in Token::lexer(source)
            .spanned()
            .skip_while(|(token, _)| matches!(token, Ok(Token::Newline)))
        {
            let token = match token {
                Ok(Token::Newline) => {
                    let slice = &source[span.clone()];
                    let last_newline = slice.rfind('\n').unwrap();
                    let spaces = slice[last_newline..].len() - 1;

                    match spaces.cmp(indent_stack.last().unwrap_or(&0)) {
                        Ordering::Less => loop {
                            indent_stack.pop();
                            if spaces >= indent_stack.last().copied().unwrap_or(0) {
                                tokens.push((Token::Dedent, span.clone()));
                                break Token::Newline;
                            }
                            tokens.push((Token::Dedent, span.clone()));
                        },
                        Ordering::Equal => Token::Newline,
                        Ordering::Greater => {
                            indent_stack.push(spaces);
                            Token::Indent
                        }
                    }
                }
                Ok(other) => other,
                Err(_) => Token::Symbol,
            };
            tokens.push((token, span));
        }

        while matches!(tokens.last(), Some((Token::Newline, _))) {
            tokens.pop();
        }

        let span = tokens
            .last()
            .map_or(0..source.len(), |(_, span)| span.clone());
        tokens.extend(
            iter::repeat(Token::Dedent)
                .take(indent_stack.len())
                .map(|token| (token, span.clone())),
        );

        TokenizeOutput {
            tokens,
            eof_token: Token::Eof,
        }
    }
}
