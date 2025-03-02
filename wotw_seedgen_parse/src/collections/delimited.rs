use super::AstCollection;
use crate::{Ast, Error, Parser, Result, Span, SpanEnd, SpanStart, Tokenize};
use std::{
    any::type_name,
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

        let mut content = Content::ast_first(parser);

        let mut close = if let Ok(content) = &mut content {
            loop {
                match Close::ast(parser) {
                    Ok(close) => break Ok(close),
                    Err(close_err) => {
                        if let ControlFlow::Break(err) = content.ast_item(parser) {
                            match err {
                                None => break Close::ast(parser),
                                Some(content_err) => {
                                    break Err(Error::all_failed(vec![close_err, content_err]))
                                }
                            }
                        } else if parser.is_finished() {
                            panic!(
                                "{}::ast_item entered an infinite loop",
                                type_name::<Content>()
                            );
                        }
                    }
                }
            }
        } else {
            Close::ast(parser)
        };

        if close.is_err() {
            let mut depth: u16 = 1;
            close = loop {
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
        }

        Ok(Self {
            open,
            content,
            close,
        })
    }
}
impl<Open: SpanStart, Content, Close: SpanEnd> Span for Delimited<Open, Content, Close> {
    #[inline]
    fn span(&self) -> Range<usize> {
        self.span_start()..self.span_end()
    }
}
impl<Open: SpanStart, Content, Close> SpanStart for Delimited<Open, Content, Close> {
    #[inline]
    fn span_start(&self) -> usize {
        self.open.span_start()
    }
}
impl<Open, Content, Close: SpanEnd> SpanEnd for Delimited<Open, Content, Close> {
    #[inline]
    fn span_end(&self) -> usize {
        self.close.span_end()
    }
}
