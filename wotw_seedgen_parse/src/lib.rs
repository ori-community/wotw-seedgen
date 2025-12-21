//! # Ast Derive
//!
//! `ast_derive` is a parsing library that aims to derive at least a good chunk of your parser directly from your Ast ([Abstract Syntax Tree](https://en.wikipedia.org/wiki/Abstract_syntax_tree))
//! and allows you to manually implement parsing for any parts of it where the derive does not suffice.
//!
//! The resulting parser will be a recursive descent parser capable of generating partial syntax trees
//! and collecting as many errors in one parse as possible. The Ast may contain Spans for all its parsed nodes if desired.
//!
//! # How does it work
//!
//! Your Ast nodes implement the [`Ast`] trait. This allows freely mixing derived and manual [`Ast`] implementations.
//! For any type implementing [`Ast`] you can use [`parse_ast`] to parse it from a [`&str`](str).
//! This means in addition to parsing the full Ast, you will be able to parse any individual Ast node.
//!
//! The Ast may contain [`Option`]s to allow collecting multiple errors from one parse.
//! This relies on you providing implementations of the [`Recover`] trait.
//!
//! The Ast may contain [`Spanned`] wrappers which store the span of their parsed content.
//! You can derive or manually implement [`Span`] on all your Ast nodes to expose the span of higher-level Ast nodes by building it from their children's spans.
//!
//! # Features
//!
//! - `ordered_float`: implements [`Ast`] for the ordered_float types [`OrderedFloat<f32>`] and [`OrderedFloat<f64>`]
//!
//! [`OrderedFloat<f32>`]: ordered_float::OrderedFloat
//! [`OrderedFloat<f64>`]: ordered_float::OrderedFloat

#![warn(clippy::todo)]

// TODO document collections
// TODO maybe more collections could implement span?

mod ast;
mod collections;
mod error;
mod helpers;
mod mode;
mod parser;
mod recover;
mod span;
mod tokenizer;

use serde::{Deserialize, Serialize};
use utoipa::ToSchema;
pub use wotw_seedgen_derive::{Ast, Span, TokenDisplay};

pub use ast::{parse_ast, Ast, ParseResult};
pub use collections::{
    AstCollection, AstCollectionInit, Delimited, Once, Punctuated, Separated, SeparatedNonEmpty,
};
pub use error::{Error, ErrorKind, ErrorWithSource, Result};
pub use helpers::{Identifier, NoTrailingInput, Symbol};
pub use mode::{ErrorMode, Errors, NoErrors};
pub use parser::{
    ParseBoolToken, ParseFloatToken, ParseIdentToken, ParseIntToken, ParseStringToken, Parser,
};
pub use recover::{Recover, Recoverable};
pub use span::{Span, SpanEnd, SpanStart, Spanned, SpannedOption};
#[cfg(feature = "logos")]
pub use tokenizer::LogosTokenizer;
pub use tokenizer::{Tokenize, TokenizeOutput};

/// Representation of a source file with the necessary information to display useful error messages.
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize, ToSchema)]
pub struct Source {
    /// An identifier to be used in error messages that should allow the reader to determine which file the error originated from.
    ///
    /// This might be the file path relative to the workspace root, or just the filename.
    pub id: String,
    /// The contents of the file, which will be referenced for better error messages.
    ///
    /// This should be the same contents you were parsing, otherwise error messages will reference arbitrary spans in your source and possibly panic.
    pub content: String, // TODO maybe use &str?
}

impl Source {
    /// Creates a new `Source` from its parts
    pub fn new(id: String, content: String) -> Self {
        Self { id, content }
    }
}
