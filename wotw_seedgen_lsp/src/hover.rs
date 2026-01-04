use crate::convert::range_to_lsp;
use itertools::Itertools;
use std::ops::Range;
use tower_lsp::lsp_types::{Hover, HoverContents, MarkupContent, MarkupKind};
use wotw_seedgen_data::assets::UberStateData;
use wotw_seedgen_data::parse::{Span, SpannedOption};
use wotw_seedgen_data::seed_language::ast;
use wotw_seedgen_data::seed_language::ast::{Handler, Snippet, Traverse};
use wotw_seedgen_data::UberIdentifier;

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
            range: Some(range_to_lsp(span, self.document)),
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

        match uber_identifier {
            ast::UberIdentifier::Numeric(numeric) => {
                let SpannedOption::Some(member) = &numeric.member.value else {
                    return;
                };

                let identifier = UberIdentifier::new(numeric.group.data, member.data);
                if let Some(uber_state_data) = self.uber_state_data.id_lookup.get(&identifier) {
                    self.set_markdown_output(
                        match &uber_state_data.rando_name {
                            None => format!("Name: `{}`", uber_state_data.name),
                            Some(rando_name) => {
                                format!("Name: `{}` ({})", uber_state_data.name, rando_name)
                            }
                        },
                        span,
                    );
                }
            }
            ast::UberIdentifier::Name(name) => {
                let Some(group_lookup) = self.uber_state_data.name_lookup.get(name.group.data.0)
                else {
                    return;
                };

                let SpannedOption::Some(member) = &name.member.value else {
                    return;
                };

                let Some(member_lookup) = group_lookup.get(member.data.0) else {
                    return;
                };

                self.set_markdown_output(
                    match member_lookup.as_slice() {
                        [single_element] => single_element.to_string(),
                        elements => elements
                            .iter()
                            .format_with("\n", |alias, f| f(&format_args!("- {alias}")))
                            .to_string(),
                    },
                    span,
                );
            }
        }
    }
}
