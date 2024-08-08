use crate::ast::{self, RecoverContent};
use rustc_hash::FxHashSet;
use wotw_seedgen_parse::{Error, Recoverable, Span, Spanned};

// TODO our preprocessing is a bit weird. For instance if you want to use an event
// from a parent file, it fails to resolve with an odd error message

#[derive(Default)]
pub(crate) struct Preprocessor {
    pub output: PreprocessorOutput,
    pub errors: Vec<Error>,
}
#[derive(Default)]
pub(crate) struct PreprocessorOutput {
    pub includes: Vec<Spanned<String>>, // TODO can these be references?
    pub functions: FxHashSet<String>,   // TODO can these be references?
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
            .filter_map(|content| content.result.as_ref().ok())
        {
            match content {
                ast::Content::Command(_, content) => {
                    if let Ok(content) = &content.result {
                        match content {
                            ast::Command::Include(_, command) => {
                                if let Ok(command) = &command.result {
                                    if let Ok(args) = &command.content {
                                        if self
                                            .output
                                            .includes
                                            .iter()
                                            .any(|include| include.data == args.0 .0.data)
                                        {
                                            self.errors.push(Error::custom(
                                                "Snippet already included".to_string(),
                                                args.0 .0.span(),
                                            ));
                                        } else {
                                            self.output.includes.push(Spanned::new(
                                                args.0 .0.data.to_string(),
                                                args.0 .0.span(),
                                            ));
                                        }
                                    }
                                }
                            }
                            ast::Command::Event(_, command) => {
                                if let Ok(command) = &command.result {
                                    if let Ok(args) = &command.content {
                                        self.output.functions.insert(args.0 .0.data.0.to_string());
                                    }
                                }
                            }
                            // TODO it seems difficult to evaluate ifs here but it's certainly odd to ignore the conditional compilation in this compiler.
                            // One side effect could be that a snippet successfully compiles which optionally declares a function behind an !if, but the client might error then
                            ast::Command::If(_, command) => {
                                if let Ok(contents) = &command.contents.content {
                                    self.preprocess_contents(contents)
                                }
                            }
                            _ => {}
                        }
                    }
                }
                ast::Content::Function(_, content) => {
                    if let Ok(function) = &content.result {
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
