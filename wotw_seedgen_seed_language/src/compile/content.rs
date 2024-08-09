use super::{Compile, SnippetCompiler};
use crate::{
    ast::{self, TriggerBinding},
    output::{intermediate::Literal, Command, CommandVoid, Event, Trigger},
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
                                compiler.errors.push(Error::custom(
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
                        compiler.errors.push(Error::custom(
                            "cannot bind to an alias which resolves to an integer state comparison"
                                .to_string(),
                            span,
                        ));
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
        let commands = compiler
            .consume_result(self.actions.content)
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
                let commands = compiler
                    .consume_result(actions.content)
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

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Annotation::Hidden(_) => {}
            ast::Annotation::Name(_, name) => {
                compiler.consume_result(name.result);
            }
            ast::Annotation::Category(_, category) => {
                compiler.consume_result(category.result);
            }
            ast::Annotation::Description(_, description) => {
                compiler.consume_result(description.result);
            }
        }
    }
}
