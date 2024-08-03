use std::{cmp::Ordering, iter::FusedIterator, ops::Range};

use crate::languages::{Cursor, Token, TokenKind};

pub(super) struct TokenStream<'a> {
    cursor: Cursor<'a>,
    indent_stack: Vec<usize>,
    ongoing_dedent: Option<(usize, Range<usize>)>,
    last_was_number: bool,
}
impl Iterator for TokenStream<'_> {
    type Item = Token;

    fn next(&mut self) -> Option<Token> {
        self.advance_token()
    }
}
impl FusedIterator for TokenStream<'_> {}

pub(super) fn tokenize(input: &str) -> TokenStream {
    let cursor = Cursor::new(input);
    let indent_stack = Vec::new();
    let ongoing_dedent = None;
    let last_was_number = false;
    TokenStream {
        cursor,
        indent_stack,
        ongoing_dedent,
        last_was_number,
    }
}

impl TokenStream<'_> {
    fn advance_token(&mut self) -> Option<Token> {
        if let Some((spaces, range)) = self.ongoing_dedent.take() {
            let kind = self.dedent(spaces);
            Some(Token { kind, range })
        } else {
            let first_char = self.cursor.bump()?;
            let kind = match first_char {
                '\n' => self.newline(),
                c if is_whitespace(c) => self.whitespace(),
                '#' => self.comment(),
                '-' => self.minus(),
                '0'..='9' => self.number(),
                'x' if self.last_was_number && !self.cursor.first().is_ascii_lowercase() => {
                    TokenKind::X
                }
                c if is_ident_char(c) => self.ident(),
                '=' => TokenKind::Eq,
                ',' => TokenKind::Comma,
                ':' => TokenKind::Colon,
                '+' => TokenKind::Plus,
                _ => TokenKind::Unknown,
            };
            self.last_was_number = kind == TokenKind::Number;
            let range = self.cursor.reset_consumed_range();

            Some(Token { kind, range })
        }
    }

    fn newline(&mut self) -> TokenKind {
        let spaces = loop {
            self.cursor.eat_while(|c| c == '\n');
            let newlines = self.cursor.consumed_count();
            self.cursor.eat_while(|c| c == ' ');
            let spaces = self.cursor.consumed_count() - newlines;
            self.whitespace();
            if self.cursor.first() != '\n' {
                break spaces;
            }
        };

        match spaces.cmp(self.indent_stack.last().unwrap_or(&0)) {
            Ordering::Greater => {
                self.indent_stack.push(spaces);
                TokenKind::Indent
            }
            Ordering::Equal => TokenKind::Newline,
            Ordering::Less => {
                self.ongoing_dedent = Some((spaces, self.cursor.consumed_range()));
                TokenKind::Newline
            }
        }
    }
    fn dedent(&mut self, spaces: usize) -> TokenKind {
        self.indent_stack.pop();
        let matching = match spaces.cmp(self.indent_stack.last().unwrap_or(&0)) {
            Ordering::Greater => false,
            Ordering::Equal => true,
            Ordering::Less => {
                self.ongoing_dedent = Some((spaces, self.cursor.consumed_range()));
                true
            }
        };
        TokenKind::Dedent { matching }
    }
    fn whitespace(&mut self) -> TokenKind {
        self.cursor.eat_while(is_whitespace);
        if self.cursor.first() == '#' {
            return self.comment();
        }
        TokenKind::Whitespace
    }
    fn comment(&mut self) -> TokenKind {
        self.cursor.eat_while(|c| c != '\n');
        TokenKind::Whitespace
    }
    fn minus(&mut self) -> TokenKind {
        if self.cursor.first().is_ascii_digit() {
            self.number()
        } else {
            TokenKind::Unknown
        }
    }
    fn number(&mut self) -> TokenKind {
        let mut decimals = false;
        self.cursor.eat_while(|c| {
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
        self.cursor.eat_while(is_ident_char);
        if self.cursor.first() == '.' && is_ident_char(self.cursor.second()) {
            self.cursor.bump();
            self.cursor.bump();
            self.cursor.eat_while(is_ident_char);
        }
        TokenKind::Identifier
    }
}

fn is_whitespace(c: char) -> bool {
    c.is_whitespace() && c != '\n'
}
fn is_ident_char(c: char) -> bool {
    c.is_alphanumeric() || c == '_'
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn indent_dedent() {
        let source = "
,
  ,
     ,
     ,
  ,
  ,
 ,
,
";
        let tokens = tokenize(source)
            .map(|token| token.kind)
            .collect::<Vec<_>>();
        assert_eq!(
            tokens,
            vec![
                TokenKind::Newline,
                TokenKind::Comma,
                TokenKind::Indent,
                TokenKind::Comma,
                TokenKind::Indent,
                TokenKind::Comma,
                TokenKind::Newline,
                TokenKind::Comma,
                TokenKind::Newline,
                TokenKind::Dedent { matching: true },
                TokenKind::Comma,
                TokenKind::Newline,
                TokenKind::Comma,
                TokenKind::Newline,
                TokenKind::Dedent { matching: false },
                TokenKind::Comma,
                TokenKind::Newline,
                TokenKind::Comma,
                TokenKind::Newline,
            ]
        )
    }
}
