use super::{Compile, SnippetCompiler};
use crate::seed_language::{
    ast::{self, TriggerBinding},
    output::{Command, CommandVoid, Event, Literal, Trigger},
};
use wotw_seedgen_parse::{Error, Span};

impl<'source> Compile<'source> for ast::Content<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Content::Event(_, event) => {
                event.compile(compiler);
            }
            ast::Content::Function(_, function) => {
                function.compile(compiler);
            }
            ast::Content::Command(_, command) => {
                command.compile(compiler);
            }
            ast::Content::Annotation(_, annotation) => {
                annotation.compile(compiler);
            }
        }
    }
}

impl<'source> Compile<'source> for ast::Event<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let trigger = self.trigger.compile(compiler);
        let span = self.action.span();
        let command = self.action.compile(compiler);

        if let Some(Some(command)) = command {
            let command = command.expect_void(compiler, span);
            if let (Some(command), Some(trigger)) = (command, trigger) {
                compiler
                    .global
                    .output
                    .events
                    .push(Event { trigger, command });
            }
        }
    }
}

impl<'source> Compile<'source> for ast::Trigger<'source> {
    type Output = Option<Trigger>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Trigger::ClientEvent(client) => Some(Trigger::ClientEvent(client.data)),
            ast::Trigger::Binding(_, binding) => {
                let binding = binding.value.into_option()?;

                let span = binding.span();

                let uber_state = match binding {
                    TriggerBinding::UberIdentifier(uber_identifier) => {
                        uber_identifier.compile(compiler)?
                    }
                    TriggerBinding::Identifier(identifier) => {
                        match compiler.resolve(&identifier)? {
                            Literal::UberIdentifier(uber_state) => uber_state.clone(),
                            other => {
                                let found = other.literal_type();
                                compiler.errors.push(Error::error(
                                    format!("Expected UberIdentifier, but found {found}"),
                                    identifier.span,
                                ));
                                return None;
                            }
                        }
                    }
                };

                match uber_state.value {
                    None => Some(Trigger::Binding(uber_state.uber_identifier)),
                    Some(_) => {
                        let mut error = Error::error(
                            "cannot bind to an alias which resolves to an integer state comparison"
                                .to_string(),
                            span,
                        );

                        if let Some(entry) = compiler
                            .global
                            .uber_state_data
                            .id_lookup
                            .get(&uber_state.uber_identifier)
                        {
                            error = error.with_help(format!(
                                "maybe you could use the underlying quest state {}",
                                entry.preferred_name()
                            ))
                        }

                        compiler.errors.push(error);

                        None
                    }
                }
            }
            ast::Trigger::Condition(expression) => {
                expression.compile_into(compiler).map(Trigger::Condition)
            }
        }
    }
}

impl<'source> Compile<'source> for ast::FunctionDefinition<'source> {
    type Output = ();

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let commands = self
            .actions
            .content
            .into_iter()
            .flatten()
            .filter_map(|action| {
                let span = action.span();
                action.compile(compiler)?.expect_void(compiler, span)
            })
            .collect();

        let index = compiler
            .function_indices
            .get(self.identifier.data.0)
            .unwrap();

        compiler.global.output.command_lookup[*index] = CommandVoid::Multi { commands };
    }
}

impl<'source> Compile<'source> for ast::Action<'source> {
    type Output = Option<Command>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Action::Function(function_call) => function_call.compile(compiler),
            ast::Action::Condition(_, condition) => condition.compile(compiler),
            ast::Action::Multi(actions) => {
                let commands = actions
                    .content
                    .into_iter()
                    .flatten()
                    .filter_map(|action| {
                        let span = action.span();
                        action.compile(compiler)?.expect_void(compiler, span)
                    })
                    .collect();

                Some(Command::Void(CommandVoid::Multi { commands }))
            }
        }
    }
}

impl<'source> Compile<'source> for ast::ActionCondition<'source> {
    type Output = Option<Command>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        let condition = self.condition.compile_into(compiler);
        let span = self.action.span();
        let command = self.action.compile(compiler);

        Some(Command::Void(CommandVoid::If {
            condition: condition?,
            command: Box::new(command??.expect_void(compiler, span)?),
        }))
    }
}

impl<'source> Compile<'source> for ast::Annotation<'source> {
    type Output = ();

    fn compile(self, _compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {}
}
