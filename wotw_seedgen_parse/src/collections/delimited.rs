use super::AstCollection;
use crate::{Ast, Error, Parser, Result, Span, Tokenize};
use std::{
    ops::{ControlFlow, Range},
    vec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delimited<Open, Content, Close> {
    pub open: Open,
    pub content: Result<Content>,
    pub close: Result<Close>, // TODO this is a newer addition, probably have to adjust compilation to propagate these errors. Maybe make a function on compiler to handle delimited
}
impl<'source, T, Open, Content, Close> Ast<'source, T> for Delimited<Open, Content, Close>
where
    T: Tokenize,
    Open: Ast<'source, T>,
    Content: AstCollection<'source, T>,
    Close: Ast<'source, T>,
{
    fn ast(parser: &mut Parser<'source, T>) -> Result<Self> {
        let open = Open::ast(parser)?;
        let content = (|| {
            let mut content = Content::ast_first(parser)?;
            loop {
                match Close::ast(parser) {
                    Ok(close) => return Ok((content, close)),
                    Err(close_err) => {
                        if let ControlFlow::Break(err) = content.ast_item(parser) {
                            return match err {
                                None => Close::ast(parser).map(|close| (content, close)),
                                Some(content_err) => {
                                    Err(Error::all_failed(vec![close_err, content_err]))
                                }
                            };
                        }
                    }
                }
            }
        })();
        let s = match content {
            Ok((content, close)) => Self {
                open,
                content: Ok(content),
                close: Ok(close),
            },
            Err(err) => {
                let mut depth: u16 = 1;
                let close = loop {
                    match Close::ast(parser) {
                        Ok(close) => {
                            depth -= 1;
                            if depth == 0 {
                                break Ok(close);
                            }
                        }
                        Err(close_err) => {
                            if Open::ast(parser).is_ok() {
                                depth += 1;
                            } else {
                                parser.step();
                            }
                            if parser.is_finished() {
                                break Err(close_err);
                            }
                        }
                    }
                };
                Self {
                    open,
                    content: Err(err),
                    close,
                }
            }
        };
        Ok(s)
    }
}
impl<Open: Span, Content, Close: Span> Span for Delimited<Open, Content, Close> {
    fn span(&self) -> Range<usize> {
        let open_span = self.open.span();
        match &self.close {
            Ok(close) => open_span.start..close.span().end,
            Err(_) => open_span,
        }
    }
}
