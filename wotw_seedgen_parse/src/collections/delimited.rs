use super::AstCollection;
use crate::{Ast, Error, Mode, Parser, Result, ResultMode, Span, SpanEnd, SpanStart, Tokenize};
use std::{
    any::type_name,
    ops::{ControlFlow, Range},
    vec,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delimited<Open, Content, Close> {
    pub open: Open,
    pub content: Result<Content>,
    pub close: Result<Close>,
}

impl<'source, T, Open, Content, Close> Ast<'source, T> for Delimited<Open, Content, Close>
where
    T: Tokenize,
    Open: Ast<'source, T>,
    Content: AstCollection<'source, T>,
    Close: Ast<'source, T>,
{
    fn ast_impl<M: Mode>(parser: &mut Parser<'source, T>) -> ControlFlow<M::Error, Self> {
        let open = Open::ast_impl::<M>(parser)?;

        let mut content = Content::ast_first_result(parser);

        let close = if let Ok(content) = &mut content {
            loop {
                match Close::ast_option(parser) {
                    Some(close) => break Ok(close),
                    None => {
                        if let ControlFlow::Break(err) = content.ast_item_impl::<ResultMode>(parser)
                        {
                            match err {
                                None => break Close::ast_result(parser),
                                Some(content_err) => {
                                    let Err(close_err) = Close::ast_result(parser) else {
                                        unreachable!()
                                    };
                                    break Err(Error::all_failed(vec![close_err, content_err]));
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
            Close::ast_result(parser)
        };

        if close.is_err() {
            let mut depth: u16 = 1;
            while !parser.is_finished() {
                if Close::ast_option(parser).is_some() {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                } else if Open::ast_option(parser).is_some() {
                    depth += 1;
                } else {
                    parser.step();
                }
            }
        }

        ControlFlow::Continue(Self {
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
