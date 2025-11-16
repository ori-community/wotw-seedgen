use super::AstCollection;
use crate::{span::SpannedOption, Ast, ErrorMode, Parser, Span, SpanEnd, SpanStart, Tokenize};
use std::{
    any::type_name,
    ops::{ControlFlow, Range},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Delimited<Open, Content, Close> {
    pub open: Open,
    pub content: Option<Content>,
    pub close: SpannedOption<Close>,
}

impl<'source, T, Open, Content, Close> Ast<'source, T> for Delimited<Open, Content, Close>
where
    T: Tokenize,
    Open: Ast<'source, T>,
    Content: AstCollection<'source, T>,
    Close: Ast<'source, T>,
{
    fn ast_impl<E: ErrorMode>(parser: &mut Parser<'source, T>) -> Option<Self> {
        let open = Open::ast_impl::<E>(parser)?;

        let mut content = Content::ast_first_spanned(parser);

        let close = match &mut content {
            SpannedOption::Some(content) => loop {
                match Close::ast_no_errors(parser) {
                    Some(close) => break SpannedOption::Some(close),
                    None => match content.ast_item(parser) {
                        ControlFlow::Continue(()) => {
                            if parser.is_finished() {
                                panic!(
                                    "{}::ast_item entered an infinite loop",
                                    type_name::<Content>()
                                );
                            }
                        }
                        ControlFlow::Break(Ok(())) => break Close::ast_spanned(parser),
                        ControlFlow::Break(Err(())) => {
                            break SpannedOption::None(parser.last_error_span())
                        }
                    },
                }
            },
            SpannedOption::None(span) => SpannedOption::None(span.clone()),
        };

        if close.as_option().is_none() {
            let mut depth: u16 = 1;
            while !parser.is_finished() {
                if Close::ast_no_errors(parser).is_some() {
                    depth -= 1;
                    if depth == 0 {
                        break;
                    }
                } else if Open::ast_no_errors(parser).is_some() {
                    depth += 1;
                } else {
                    parser.step();
                }
            }
        }

        Some(Self {
            open,
            content: content.into_option(),
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
