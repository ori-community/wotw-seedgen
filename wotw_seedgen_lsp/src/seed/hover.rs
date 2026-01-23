use crate::convert;
use crate::seed::helpers::uber_identifier_info;
use std::ops::Range;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};
use wotw_seedgen_data::assets::UberStateData;
use wotw_seedgen_data::parse::Span;
use wotw_seedgen_data::seed_language::ast;
use wotw_seedgen_data::seed_language::ast::{Handler, Snippet, Traverse};

pub fn hover(
    document: &str,
    ast: Option<Snippet>,
    text_document_byte_index: usize,
    uber_state_data: &UberStateData,
) -> Option<Hover> {
    let mut hover_handler = HoverHandler::new(document, text_document_byte_index, uber_state_data);
    ast.traverse(&mut hover_handler);
    hover_handler.output
}

struct HoverHandler<'a, 'b> {
    document: &'a str,
    text_document_byte_index: usize,
    uber_state_data: &'b UberStateData,
    output: Option<Hover>,
}

impl<'a, 'b> HoverHandler<'a, 'b> {
    fn new(
        document: &'a str,
        text_document_byte_index: usize,
        uber_state_data: &'b UberStateData,
    ) -> Self {
        Self {
            document,
            text_document_byte_index,
            uber_state_data,
            output: None,
        }
    }

    fn set_markdown_output(&mut self, value: String, span: Range<usize>) {
        self.output = Some(Hover {
            range: Some(convert::range_to_lsp(span, self.document)),
            contents: HoverContents::Markup(MarkupContent {
                kind: MarkupKind::Markdown,
                value,
            }),
        });
    }
}

impl Handler for HoverHandler<'_, '_> {
    fn uber_identifier(&mut self, uber_identifier: &ast::UberIdentifier) {
        let span = uber_identifier.span();

        if !span.contains(&self.text_document_byte_index) {
            return;
        }

        let Some(info) = uber_identifier_info(uber_identifier, self.uber_state_data) else {
            return;
        };

        self.set_markdown_output(info, span);
    }
}
