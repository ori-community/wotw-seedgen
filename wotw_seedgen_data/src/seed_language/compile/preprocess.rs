use std::{borrow::Cow, collections::hash_map::Entry, ops::RangeFrom};

use crate::seed_language::{
    ast::{self, get_command_arg_ref, Punctuated, RecoverContent},
    compile::{FunctionArg, FunctionSignature},
    output::IntermediateOutput,
    types::Type,
};
use rustc_hash::FxHashMap;
use wotw_seedgen_parse::{Error, Identifier, Recoverable, Span, Spanned, SpannedOption};

// TODO our preprocessing is a bit weird. For instance if you want to use an event
// from a parent file, it fails to resolve with an odd error message

pub(crate) struct Preprocessor {
    pub output: PreprocessorOutput,
    pub errors: Vec<Error>,
    next_function_index: RangeFrom<usize>,
}

// TODO we could probably use a lot more references if the compile trait didn't take ownership of self
#[derive(Default)]
pub(crate) struct PreprocessorOutput {
    pub config_sets: Vec<(String, String, String)>,
    pub includes: Vec<Spanned<String>>,
    pub functions: FxHashMap<String, PreprocessedFunction>,
}

#[derive(Debug, Clone)]
pub(crate) struct PreprocessedFunction {
    pub index: usize,
    pub signature: FunctionSignature,
}

impl Preprocessor {
    fn new(output: &IntermediateOutput) -> Self {
        Self {
            output: PreprocessorOutput::default(),
            errors: vec![],
            next_function_index: output.commands.lookup.len()..,
        }
    }

    pub(crate) fn preprocess(ast: &ast::Snippet, output: &IntermediateOutput) -> Self {
        let mut preprocessor = Self::new(output);
        preprocessor.preprocess_contents(&ast.contents);
        preprocessor
    }

    fn preprocess_contents(&mut self, contents: &[Recoverable<ast::Content, RecoverContent>]) {
        for content in contents
            .iter()
            .filter_map(|content| content.value.as_option())
        {
            match content {
                ast::Content::Command(_, content) => {
                    if let SpannedOption::Some(content) = &content.value {
                        match content {
                            ast::Command::Include(_, command) => {
                                if let SpannedOption::Some(command) = &command.value {
                                    if let Some(args) = &command.content {
                                        if self.output.snippet_included(args.0.path.data) {
                                            self.errors.push(Error::error(
                                                "Snippet already included".to_string(),
                                                args.0.path.span(),
                                            ));
                                        } else {
                                            self.output.includes.push(Spanned::new(
                                                args.0.path.data.to_string(),
                                                args.0.path.span(),
                                            ));
                                        }
                                    }
                                }
                            }
                            ast::Command::SetConfig(_, command) => {
                                if let SpannedOption::Some(command) = &command.value {
                                    if let Some(args) = &command.content {
                                        let identifier = get_command_arg_ref(&args.0.identifier);
                                        let value = get_command_arg_ref(&args.0.value);

                                        if let (Some(identifier), Some(value)) = (identifier, value)
                                        {
                                            let snippet_name = args.0.snippet_name.data.to_string();
                                            let identifier = identifier.data.0.to_string();
                                            let value = value.data.to_string();

                                            self.output.config_sets.push((
                                                snippet_name,
                                                identifier,
                                                value,
                                            ));
                                        }
                                    }
                                }
                            }
                            // TODO it seems difficult to evaluate ifs here but it's certainly odd to ignore the conditional compilation in this compiler.
                            // One side effect could be that a snippet successfully compiles which optionally declares a function behind an !if, but the client might error then
                            // Idea: All commands are evaluated in preprocessing. This way ifs can be evaluated here as well.
                            // Knowing the function structure isn't relevant in commands and includes can be handled immediately.
                            // Reassigning identifiers in let commands should be disallowed to avoid confusion where a later let commands influences an earlier function call.
                            ast::Command::If(_, command) => {
                                if let Some(contents) = &command.contents.content {
                                    self.preprocess_contents(contents)
                                }
                            }
                            _ => {}
                        }
                    }
                }
                ast::Content::Function(_, content) => {
                    if let SpannedOption::Some(function) = &content.value {
                        if let Some(signature) = &function.signature.content {
                            self.preprocess_function(&function.identifier, signature);
                        }
                    }
                }
                _ => {}
            }
        }
    }

    fn preprocess_function(
        &mut self,
        identifier: &Spanned<Identifier>,
        signature: &Punctuated<ast::FunctionArg, ','>,
    ) {
        match self.output.functions.entry(identifier.data.0.to_string()) {
            Entry::Occupied(_) => self.errors.push(Error::error(
                "already defined".to_string(),
                identifier.span.clone(),
            )),
            Entry::Vacant(vacant) => {
                let args = signature
                    .iter()
                    .map(|arg| {
                        if !matches!(
                            arg.ty.data,
                            Type::Boolean | Type::Integer | Type::Float | Type::String
                        ) {
                            self.errors.push(Error::error(
                                "unsupported type for function arguments".to_string(),
                                arg.ty.span.clone(),
                            ));
                        }

                        FunctionArg {
                            identifier: Cow::Owned(arg.identifier.data.0.to_string()),
                            ty: arg.ty.data,
                        }
                    })
                    .collect();

                let function = PreprocessedFunction {
                    index: self.next_function_index.next().unwrap(),
                    signature: FunctionSignature {
                        args,
                        return_ty: None,
                    },
                };

                vacant.insert(function);
            }
        }
    }
}

impl PreprocessorOutput {
    pub(crate) fn snippet_included(&self, identifier: &str) -> bool {
        self.includes
            .iter()
            .any(|include| include.data == identifier)
    }
}
