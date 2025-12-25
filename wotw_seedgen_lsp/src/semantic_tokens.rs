use std::{mem, ops::Range};
use tower_lsp::lsp_types::{SemanticToken, SemanticTokenType, SemanticTokensLegend};
use wotw_seedgen_data::{
    parse::Identifier,
    seed_language::ast::{Handler, Snippet, Span, Spanned, Traverse, UberIdentifier},
};

use crate::convert;

pub fn semantic_tokens(source: &str, ast: Option<Snippet>) -> Vec<SemanticToken> {
    let mut builder = TokenBuilder::new(source);
    ast.traverse(&mut builder);
    builder.finish()
}

pub fn semantic_tokens_legend() -> SemanticTokensLegend {
    SemanticTokensLegend {
        token_types: vec![
            SemanticTokenType::TYPE,
            SemanticTokenType::VARIABLE,
            SemanticTokenType::ENUM_MEMBER,
            SemanticTokenType::FUNCTION,
            SemanticTokenType::MACRO,
            SemanticTokenType::KEYWORD,
            SemanticTokenType::STRING,
            SemanticTokenType::NUMBER,
            SemanticTokenType::OPERATOR,
        ],
        token_modifiers: vec![],
    }
}

#[repr(u32)]
enum TokenType {
    Type,
    Variable,
    EnumMember,
    Function,
    Macro,
    Keyword,
    String,
    Number,
    Operator,
}

struct TokenBuilder<'source> {
    source: &'source str,
    tokens: Vec<SemanticToken>,
    previous_line: usize,
    previous_offset: usize,
}

impl<'source> TokenBuilder<'source> {
    fn new(source: &'source str) -> Self {
        Self {
            source,
            tokens: Default::default(),
            previous_line: Default::default(),
            previous_offset: Default::default(),
        }
    }

    fn push_token(&mut self, span: Range<usize>, token_type: TokenType) {
        let (line, line_start) = convert::last_line(&self.source[..span.start]);

        let delta_line = (line - mem::replace(&mut self.previous_line, line)) as u32;

        let previous_offset = if delta_line == 0 {
            self.previous_offset
        } else {
            line_start
        };
        self.previous_offset = span.start;

        let delta_start = self.source[previous_offset..span.start]
            .encode_utf16()
            .count() as u32;

        let length = self.source[span].encode_utf16().count() as u32;

        self.tokens.push(SemanticToken {
            delta_line,
            delta_start,
            length,
            token_type: token_type as u32,
            token_modifiers_bitset: 0,
        })
    }

    fn finish(self) -> Vec<SemanticToken> {
        self.tokens
    }
}

impl Handler for TokenBuilder<'_> {
    fn keyword(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Keyword);
    }

    fn command_keyword(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Macro);
    }

    fn annotation_keyword(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Macro);
    }

    fn command_symbol(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Macro);
    }

    fn annotation_symbol(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Macro);
    }

    fn operator(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Operator);
    }

    fn boolean(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Keyword);
    }

    fn integer(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Number);
    }

    fn float(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Number);
    }

    fn string(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::String);
    }

    fn constant(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::EnumMember);
    }

    fn ty(&mut self, span: &Range<usize>) {
        self.push_token(span.clone(), TokenType::Type);
    }

    fn uber_identifier(&mut self, uber_identifier: &UberIdentifier) {
        self.push_token(uber_identifier.span(), TokenType::Number);
    }

    fn identifier(&mut self, identifier: &Spanned<Identifier>) {
        self.push_token(identifier.span.clone(), TokenType::Variable);
    }

    fn identifier_def(&mut self, identifier: &Spanned<Identifier>) {
        self.push_token(identifier.span.clone(), TokenType::Variable);
    }

    fn identifier_use(&mut self, identifier: &Spanned<Identifier>) {
        self.push_token(identifier.span.clone(), TokenType::Variable);
    }

    fn function_def(&mut self, identifier: &Spanned<Identifier>) {
        self.push_token(identifier.span.clone(), TokenType::Function);
    }

    fn function_use(&mut self, identifier: &Spanned<Identifier>) {
        self.push_token(identifier.span.clone(), TokenType::Function);
    }
}
