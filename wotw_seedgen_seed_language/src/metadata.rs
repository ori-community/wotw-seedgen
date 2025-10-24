pub use crate::output::Literal;

use crate::ast::{self, CommandArg};
use rustc_hash::FxHashMap;
use wotw_seedgen_parse::Recoverable;

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Metadata {
    pub hidden: bool,
    pub name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub config: FxHashMap<String, ConfigValue>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConfigValue {
    pub description: String,
    pub default: Literal,
}

impl Metadata {
    pub fn from_source(source: &str) -> Self {
        ast::parse::<ast::Snippet>(source)
            .parsed
            .ok()
            .as_ref()
            .map_or_else(Self::default, Self::from_ast)
    }

    pub fn from_ast(ast: &ast::Snippet) -> Self {
        let mut metadata = Self::default();
        ast.extract_metadata(&mut metadata);
        metadata
    }
}

trait ExtractMetadata {
    fn extract_metadata(&self, metadata: &mut Metadata);
}

impl<T: ExtractMetadata> ExtractMetadata for Vec<T> {
    fn extract_metadata(&self, metadata: &mut Metadata) {
        for t in self {
            t.extract_metadata(metadata);
        }
    }
}

impl<T: ExtractMetadata, R> ExtractMetadata for Recoverable<T, R> {
    fn extract_metadata(&self, metadata: &mut Metadata) {
        if let Ok(t) = &self.result {
            t.extract_metadata(metadata);
        }
    }
}

impl ExtractMetadata for ast::Snippet<'_> {
    fn extract_metadata(&self, metadata: &mut Metadata) {
        self.contents.extract_metadata(metadata);
    }
}

impl ExtractMetadata for ast::Content<'_> {
    fn extract_metadata(&self, metadata: &mut Metadata) {
        match self {
            ast::Content::Annotation(_, annotation) => annotation.extract_metadata(metadata),
            ast::Content::Command(_, command) => command.extract_metadata(metadata),
            _ => {}
        }
    }
}

impl ExtractMetadata for ast::Annotation<'_> {
    fn extract_metadata(&self, metadata: &mut Metadata) {
        match self {
            ast::Annotation::Hidden(_) => {
                metadata.hidden = true;
            }
            ast::Annotation::Name(_, args) => {
                if let Some(name) = extract_args(args) {
                    metadata.name = Some(name.data.to_string());
                }
            }
            ast::Annotation::Category(_, args) => {
                if let Some(category) = extract_args(args) {
                    metadata.category = Some(category.data.to_string());
                }
            }
            ast::Annotation::Description(_, args) => {
                if let Some(description) = extract_args(args) {
                    metadata.description = Some(description.data.to_string());
                }
            }
        }
    }
}

impl ExtractMetadata for ast::Command<'_> {
    fn extract_metadata(&self, metadata: &mut Metadata) {
        let ast::Command::Config(_, args) = self else {
            return;
        };

        let Some(config) = extract_args(args) else {
            return;
        };

        let (Some(default), Some(description)) = (
            extract_command_arg(&config.default),
            extract_command_arg(&config.description),
        ) else {
            return;
        };

        let default = match default.data {
            ast::Literal::Boolean(value) => Literal::Boolean(value),
            ast::Literal::Integer(value) => Literal::Integer(value),
            ast::Literal::Float(value) => Literal::Float(value),
            _ => return,
        };

        let value = ConfigValue {
            description: description.data.to_string(),
            default,
        };

        metadata
            .config
            .insert(config.identifier.data.to_string(), value);
    }
}

fn extract_args<Args>(args: &ast::CommandArgs<Args>) -> Option<&Args> {
    args.result
        .as_ref()
        .and_then(|args| args.content.as_ref())
        .ok()
        .map(|args| &args.0)
}

fn extract_command_arg<T>(arg: &CommandArg<T>) -> Option<&T> {
    arg.result
        .as_ref()
        .and_then(|(_, arg)| arg.result.as_ref())
        .ok()
}
