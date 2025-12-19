use crate::seed_language::ast::{self, inspect_command_arg, RecoverContent};
use rustc_hash::FxHashSet;
use wotw_seedgen_parse::{Error, Recoverable, Span, Spanned, SpannedOption};

// TODO our preprocessing is a bit weird. For instance if you want to use an event
// from a parent file, it fails to resolve with an odd error message

#[derive(Default)]
pub(crate) struct Preprocessor {
    pub output: PreprocessorOutput,
    pub errors: Vec<Error>,
}

#[derive(Default)]
pub(crate) struct PreprocessorOutput {
    pub config_sets: Vec<(String, String, String)>, // TODO can these be references?
    pub includes: Vec<Spanned<String>>,             // TODO can these be references?
    pub functions: FxHashSet<String>,               // TODO can these be references?
}

impl Preprocessor {
    pub(crate) fn preprocess(ast: &ast::Snippet) -> Self {
        let mut preprocessor = Self::default();
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
                                            self.errors.push(Error::custom(
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
                                        let identifier = inspect_command_arg(&args.0.identifier);
                                        let value = inspect_command_arg(&args.0.value);

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
                        self.output
                            .functions
                            .insert(function.identifier.data.0.to_string());
                    }
                }
                _ => {}
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
