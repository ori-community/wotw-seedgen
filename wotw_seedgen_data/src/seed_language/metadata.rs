use std::fmt::{self, Display};

use crate::seed_language::ast::{
    self, get_command_arg_ref, get_command_args_ref, inspect_command_args, Handler, Traverse,
};
use indexmap::IndexMap;
use ordered_float::OrderedFloat;
use rustc_hash::{FxBuildHasher, FxHashMap};
use serde::Serialize;
use utoipa::ToSchema;
use wotw_seedgen_parse::{Identifier, Spanned, SpannedOption, Symbol};

/// Metadata about a snippet
#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, ToSchema)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    /// Whether the snippet should be hidden from the options when generating seeds
    pub hidden: bool,
    /// Display name
    pub name: Option<String>,
    /// Category shared with other snippets
    pub category: Option<String>,
    /// Longer description
    pub description: Option<String>,
    /// Included snippets
    pub includes: Vec<String>,
    /// Available configuration
    #[schema(value_type = FxHashMap<String, ConfigArg>)]
    pub config: IndexMap<String, ConfigArg, FxBuildHasher>,
    /// Whether this snippet requires local files, preventing it from being inlined
    ///
    /// Note that included snippets may require local files even if this one doesn't
    pub requires_local_files: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
pub struct ConfigArg {
    pub name: String,
    pub description: Option<String>,
    pub value: ConfigValue,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, ToSchema)]
#[serde(tag = "type")]
pub enum ConfigValue {
    Boolean {
        default: bool,
    },
    Integer {
        default: i32,
    },
    IntegerRange {
        default: i32,
        min: i32,
        max: i32,
    },
    Float {
        #[schema(value_type = f32)]
        default: OrderedFloat<f32>,
    },
    FloatRange {
        #[schema(value_type = f32)]
        default: OrderedFloat<f32>,
        #[schema(value_type = f32)]
        min: OrderedFloat<f32>,
        #[schema(value_type = f32)]
        max: OrderedFloat<f32>,
    },
}

impl ConfigValue {
    pub fn display_default(&self) -> DisplayDefault<'_> {
        DisplayDefault { value: self }
    }
}

pub struct DisplayDefault<'a> {
    value: &'a ConfigValue,
}

impl Display for DisplayDefault<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self.value {
            ConfigValue::Boolean { default } => default.fmt(f),
            ConfigValue::Integer { default } => default.fmt(f),
            ConfigValue::IntegerRange { default, .. } => default.fmt(f),
            ConfigValue::Float { default } => default.fmt(f),
            ConfigValue::FloatRange { default, .. } => default.fmt(f),
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

    fn include(&mut self, args: &ast::CommandArgs<ast::IncludeArgs>) {
        let Some(args) = get_command_args_ref(args) else {
            return;
        };

        self.includes.push(args.path.data.to_string());
    }

    fn config_boolean(&mut self, args: &ast::CommandArgs<ast::ConfigBooleanArgs>) {
        let Some(args) = get_command_args_ref(args) else {
            return;
        };

        let Some(default) = get_config_boolean(&args.0.default) else {
            return;
        };

        self.config(&args.0, ConfigValue::Boolean { default });
    }

    fn config_integer(&mut self, args: &ast::CommandArgs<ast::ConfigIntegerArgs>) {
        let Some(args) = get_command_args_ref(args) else {
            return;
        };

        let Some(default) = get_config_integer(&args.0.default) else {
            return;
        };

        self.config(&args.0, ConfigValue::Integer { default });
    }

    fn config_integer_range(&mut self, args: &ast::CommandArgs<Box<ast::ConfigIntegerRangeArgs>>) {
        let Some(args) = get_command_args_ref(args) else {
            return;
        };

        let default = get_config_integer(&args.0.default);
        let min = get_config_integer(&args.0.min);
        let max = get_config_integer(&args.0.max);

        let (Some(default), Some(min), Some(max)) = (default, min, max) else {
            return;
        };

        self.config_range(&args.0, ConfigValue::IntegerRange { default, min, max });
    }

    fn config_float(&mut self, args: &ast::CommandArgs<ast::ConfigFloatArgs>) {
        let Some(args) = get_command_args_ref(args) else {
            return;
        };

        let Some(default) = get_config_float(&args.0.default) else {
            return;
        };

        self.config(&args.0, ConfigValue::Float { default });
    }

    fn config_float_range(&mut self, args: &ast::CommandArgs<Box<ast::ConfigFloatRangeArgs>>) {
        let Some(args) = get_command_args_ref(args) else {
            return;
        };

        let default = get_config_float(&args.0.default);
        let min = get_config_float(&args.0.min);
        let max = get_config_float(&args.0.max);

        let (Some(default), Some(min), Some(max)) = (default, min, max) else {
            return;
        };

        self.config_range(&args.0, ConfigValue::FloatRange { default, min, max });
    }

    fn config(&mut self, args: &ast::ConfigArgs, value: ConfigValue) {
        self.insert_config(&args.identifier, &args.name, &args.description, value);
    }

    fn config_range(&mut self, args: &ast::ConfigRangeArgs, value: ConfigValue) {
        self.insert_config(&args.identifier, &args.name, &args.description, value);
    }

    fn insert_config(
        &mut self,
        identifier: &Spanned<Identifier>,
        name: &ast::CommandArg<Spanned<&str>>,
        description: &SpannedOption<(Symbol<','>, Spanned<&str>)>,
        value: ConfigValue,
    ) {
        let Some(name) = get_command_arg_ref(name) else {
            return;
        };

        let name = name.data.to_string();
        let description = description
            .as_option()
            .map(|(_, description)| description.data.to_string());

        self.config.insert(
            identifier.data.0.to_string(),
            ConfigArg {
                name,
                description,
                value,
            },
        );
    }
}

impl Handler for Metadata {
    fn command(&mut self, command: &ast::Command) {
        match command {
            ast::Command::Include(_, args) => self.include(args),
            ast::Command::IncludeIcon(..) => self.requires_local_files = true,
            ast::Command::ConfigBoolean(_, args) => self.config_boolean(args),
            ast::Command::ConfigInteger(_, args) => self.config_integer(args),
            ast::Command::ConfigIntegerRange(_, args) => self.config_integer_range(args),
            ast::Command::ConfigFloat(_, args) => self.config_float(args),
            ast::Command::ConfigFloatRange(_, args) => self.config_float_range(args),
            _ => {}
        }
    }

    fn annotation(&mut self, annotation: &ast::Annotation) {
        match annotation {
            ast::Annotation::Hidden(_) => self.hidden = true,
            ast::Annotation::Name(_, args) => {
                inspect_command_args(args, |name| self.name = Some(name.data.to_string()));
            }
            ast::Annotation::Category(_, args) => inspect_command_args(args, |category| {
                self.category = Some(category.data.to_string());
            }),
            ast::Annotation::Description(_, args) => inspect_command_args(args, |description| {
                self.description = Some(description.data.to_string());
            }),
        }
    }
}

fn get_config_boolean(literal: &ast::CommandArg<Spanned<ast::Literal>>) -> Option<bool> {
    let literal = get_command_arg_ref(literal)?;

    match literal.data {
        ast::Literal::Boolean(value) => Some(value),
        _ => None,
    }
}

fn get_config_integer(literal: &ast::CommandArg<Spanned<ast::Literal>>) -> Option<i32> {
    let literal = get_command_arg_ref(literal)?;

    match literal.data {
        ast::Literal::Integer(value) => Some(value),
        _ => None,
    }
}

fn get_config_float(literal: &ast::CommandArg<Spanned<ast::Literal>>) -> Option<OrderedFloat<f32>> {
    let literal = get_command_arg_ref(literal)?;

    match literal.data {
        ast::Literal::Float(value) => Some(value),
        _ => None,
    }
}
