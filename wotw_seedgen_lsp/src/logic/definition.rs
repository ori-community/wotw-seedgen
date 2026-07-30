use tower_lsp::lsp_types::{GotoDefinitionResponse, Location, Url};
use wotw_seedgen_data::{
    logic_language::ast::{Handler, LogicIdentifier, Paths, PlainRequirement, Traverse},
    parse::{Identifier, Spanned},
};

use crate::convert;

pub fn goto_definition(
    ast: Option<Paths>,
    index: usize,
    uri: &Url,
    document: &str,
) -> Option<GotoDefinitionResponse> {
    let mut handler: DefinitionHandler<'_, '_> = DefinitionHandler::new(index);
    ast.traverse(&mut handler);
    handler.finish(uri, document)
}

struct DefinitionHandler<'ast, 'source> {
    index: usize,
    state: DefinitionHandlerState<'ast, 'source>,
}

#[derive(Default)]
struct DefinitionHandlerState<'ast, 'source> {
    macro_defs: Vec<&'ast Spanned<Identifier<'source>>>,
    anchor_defs: Vec<&'ast Spanned<LogicIdentifier<'source>>>,
    state_defs: Vec<&'ast Spanned<LogicIdentifier<'source>>>,
    query: Option<DefinitionQuery<'source>>,
}

enum DefinitionQuery<'source> {
    Anchor(LogicIdentifier<'source>),
    State(LogicIdentifier<'source>),
    Requirement(Identifier<'source>),
}

impl DefinitionHandler<'_, '_> {
    fn new(index: usize) -> Self {
        Self {
            index,
            state: DefinitionHandlerState::default(),
        }
    }

    fn finish(self, uri: &Url, document: &str) -> Option<GotoDefinitionResponse> {
        let locations = match self.state.query? {
            DefinitionQuery::Anchor(query) => {
                find_defs(query, self.state.anchor_defs, uri, document)
            }
            DefinitionQuery::State(query) => find_defs(query, self.state.state_defs, uri, document),
            DefinitionQuery::Requirement(query) => {
                find_defs(query, self.state.macro_defs, uri, document)
            }
        };

        match locations.len() {
            0 => None,
            1 => locations
                .into_iter()
                .next()
                .map(GotoDefinitionResponse::Scalar),
            _ => Some(GotoDefinitionResponse::Array(locations)),
        }
    }
}

fn find_defs<'source, T, D>(query: T, defs: D, uri: &Url, document: &str) -> Vec<Location>
where
    T: PartialEq + 'source,
    D: IntoIterator<Item = &'source Spanned<T>>,
{
    defs.into_iter()
        .filter(|def| def.data == query)
        .map(|def| {
            Location::new(
                uri.clone(),
                convert::range_to_lsp(def.span.clone(), document),
            )
        })
        .collect()
}

impl<'ast, 'source> Handler<'ast, 'source> for DefinitionHandler<'ast, 'source> {
    fn macro_def(&mut self, identifier: &'ast Spanned<Identifier<'source>>) {
        self.state.macro_defs.push(identifier);
    }

    fn anchor_def(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        self.state.anchor_defs.push(identifier);
    }

    fn anchor_use(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        if identifier.span.contains(&self.index) {
            self.state.query = Some(DefinitionQuery::Anchor(identifier.data));
        }
    }

    fn state_def(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        self.state.state_defs.push(identifier);
    }

    fn state_use(&mut self, identifier: &'ast Spanned<LogicIdentifier<'source>>) {
        if identifier.span.contains(&self.index) {
            self.state.query = Some(DefinitionQuery::State(identifier.data));
        }
    }

    fn plain_requirement(&mut self, requirement: &'ast PlainRequirement<'source>) {
        let identifier = &requirement.identifier;

        if identifier.span.contains(&self.index) {
            self.state.query = Some(DefinitionQuery::Requirement(identifier.data));
        }
    }
}
