use std::fmt::{self, Display};

use crate::seed_language::ast::{
    self, get_command_arg_ref, inspect_command_args, Handler, Traverse,
};
use ordered_float::OrderedFloat;
use rustc_hash::FxHashMap;
use serde::Serialize;
use utoipa::ToSchema;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, ToSchema)]
pub struct Metadata {
    pub hidden: bool,
    pub name: Option<String>,
    pub category: Option<String>,
    pub description: Option<String>,
    pub config: FxHashMap<String, ConfigValue>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ConfigValue {
    pub description: String,
    pub default: ConfigDefault,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub enum ConfigDefault {
    Boolean(bool),
    Integer(i32),
    #[schema(value_type = f32)]
    Float(OrderedFloat<f32>),
}

impl Display for ConfigDefault {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Boolean(value) => value.fmt(f),
            Self::Integer(value) => value.fmt(f),
            Self::Float(value) => value.fmt(f),
        }
    }
}

impl Metadata {
    pub fn from_source(source: &str) -> Self {
        ast::Snippet::parse(source)
            .parsed
            .as_ref()
            .map_or_else(Self::default, Self::from_ast)
    }

    pub fn from_ast(ast: &ast::Snippet) -> Self {
        let mut metadata = Self::default();
        ast.traverse(&mut metadata);
        metadata
    }
}

impl Handler for Metadata {
    fn annotation(&mut self, annotation: &ast::Annotation) {
        match annotation {
            ast::Annotation::Hidden(_) => self.hidden = true,
            ast::Annotation::Name(_, args) => {
                inspect_command_args(args, |name| self.name = Some(name.data.to_string()))
            }
            ast::Annotation::Category(_, args) => inspect_command_args(args, |category| {
                self.category = Some(category.data.to_string())
            }),
            ast::Annotation::Description(_, args) => inspect_command_args(args, |description| {
                self.description = Some(description.data.to_string())
            }),
        }
    }

    fn config(&mut self, config: &ast::ConfigArgs) {
        let (Some(default), Some(description)) = (
            get_command_arg_ref(&config.default),
            get_command_arg_ref(&config.description),
        ) else {
            return;
        };

        let default = match default.data {
            ast::Literal::Boolean(value) => ConfigDefault::Boolean(value),
            ast::Literal::Integer(value) => ConfigDefault::Integer(value),
            ast::Literal::Float(value) => ConfigDefault::Float(value),
            _ => return,
        };

        let value = ConfigValue {
            description: description.data.to_string(),
            default,
        };

        self.config
            .insert(config.identifier.data.to_string(), value);
    }
}
