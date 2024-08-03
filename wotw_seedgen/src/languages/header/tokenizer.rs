use std::iter::FusedIterator;

use crate::languages::{CommentKind, Cursor, Token, TokenKind};

pub(crate) struct TokenStream<'a> {
    cursor: Cursor<'a>,
}
impl Iterator for TokenStream<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.cursor.advance_token()
    }
}
impl FusedIterator for TokenStream<'_> {}

pub(super) fn tokenize(input: &str) -> TokenStream {
    let cursor = Cursor::new(input);
    TokenStream { cursor }
}

impl Cursor<'_> {
    fn advance_token(&mut self) -> Option<Token> {
        let first_char = self.bump()?;
        let kind = match first_char {
            '\n' => self.newline(),
            c if is_whitespace(c) => self.whitespace(),
            '/' => self.comment(),
            '-' => self.minus(),
            '0'..='9' => self.number(),
            c if is_ident_char(c) => self.ident(),
            '"' => self.string(),
            '|' => TokenKind::Separator,
            '=' => TokenKind::Eq,
            '>' => TokenKind::Greater,
            '<' => TokenKind::Less,
            ',' => TokenKind::Comma,
            '.' => TokenKind::Dot,
            ':' => TokenKind::Colon,
            '!' => TokenKind::Bang,
            '$' => TokenKind::Dollar,
            '(' => TokenKind::OpenParen,
            ')' => TokenKind::CloseParen,
            '[' => TokenKind::OpenBracket,
            ']' => TokenKind::CloseBracket,
            '{' => TokenKind::OpenBrace,
            '}' => TokenKind::CloseBrace,
            '+' => TokenKind::Plus,
            '#' => TokenKind::Pound,
            _ => TokenKind::Unknown,
        };
        let range = self.reset_consumed_range();

        Some(Token { kind, range })
    }

    fn newline(&mut self) -> TokenKind {
        self.eat_while(|c| c == '\n');
        TokenKind::Newline
    }
    fn whitespace(&mut self) -> TokenKind {
        self.eat_while(is_whitespace);
        TokenKind::Whitespace
    }
    fn comment(&mut self) -> TokenKind {
        if self.first() == '/' {
            self.bump();
            let kind = if self.first() == '/' {
                self.bump();
                if self.first() == '/' {
                    self.bump();
                    if self.first() == '/' {
                        CommentKind::Note
                    } else {
                        CommentKind::ConfigDoc
                    }
                } else {
                    CommentKind::HeaderDoc
                }
            } else {
                CommentKind::Note
            };

            self.eat_while(|c| c != '\n');
            TokenKind::Comment { kind }
        } else {
            TokenKind::Unknown
        }
    }
    fn minus(&mut self) -> TokenKind {
        if self.first().is_ascii_digit() {
            self.number()
        } else {
            TokenKind::Minus
        }
    }
    fn number(&mut self) -> TokenKind {
        let mut decimals = false;
        self.eat_while(|c| {
            if c == '.' && !decimals {
                decimals = true;
                true
            } else {
                c.is_ascii_digit()
            }
        });
        TokenKind::Number
    }
    fn ident(&mut self) -> TokenKind {
        self.eat_while(is_ident_char);
        TokenKind::Identifier
    }
    fn string(&mut self) -> TokenKind {
        self.eat_while(|c| c != '"');
        let terminated = self.bump().is_some();
        TokenKind::String { terminated }
    }
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() && c != '\n'
}
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}
