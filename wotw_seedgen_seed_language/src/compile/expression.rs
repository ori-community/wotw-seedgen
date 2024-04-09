use super::{Compile, SnippetCompiler};
use crate::{
    ast::{self, Operator, UberStateType},
    output::{
        intermediate::{Constant, Literal},
        ArithmeticOperator, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
        CommandVoid, CommandZone, Comparator, EqualityComparator, Icon, LogicOperator, Operation,
        StringOrPlaceholder,
    },
    types::{common_type, InferType, Type},
};
use ordered_float::OrderedFloat;
use std::ops::Range;
use wotw_seedgen_assets::UberStateAlias;
use wotw_seedgen_data::UberIdentifier;
use wotw_seedgen_parse::{Error, Span, Spanned};

impl Command {
    pub(crate) fn expect_void<S: Span>(
        self,
        compiler: &mut SnippetCompiler<'_, '_, '_, '_>,
        span: S,
    ) -> Option<CommandVoid> {
        let result = match self {
            Command::Void(command) => Ok(command),
            _ => Err(Error::custom(
                "unexpected return value".to_string(),
                span.span(),
            )),
        };
        compiler.consume_result(result)
    }
}

impl<'source> ast::Expression<'source> {
    pub(crate) fn compile_into<T: CompileInto>(
        self,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<T> {
        match self {
            ast::Expression::Value(value) => value.compile_into(compiler),
            ast::Expression::Operation(operation) => T::compile_operation(*operation, compiler),
        }
    }
}
impl<'source> ast::ExpressionValue<'source> {
    pub(crate) fn compile_into<T: CompileInto>(
        self,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<T> {
        match self {
            ast::ExpressionValue::Group(group) => compiler
                .consume_result(group.content)?
                .0
                .compile_into(compiler),
            ast::ExpressionValue::Action(action) => T::compile_action(action, compiler),
            ast::ExpressionValue::Literal(literal) => T::compile_literal(literal, compiler),
            ast::ExpressionValue::Identifier(identifier) => compiler
                .resolve(&identifier)
                .cloned()
                .and_then(|literal| T::coerce_literal(literal, identifier.span, compiler)),
        }
    }
}
impl<'source> Compile<'source> for ast::ArithmeticOperator {
    type Output = ArithmeticOperator;

    fn compile(self, _compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::ArithmeticOperator::Add => ArithmeticOperator::Add,
            ast::ArithmeticOperator::Subtract => ArithmeticOperator::Subtract,
            ast::ArithmeticOperator::Multiply => ArithmeticOperator::Multiply,
            ast::ArithmeticOperator::Divide => ArithmeticOperator::Divide,
        }
    }
}
impl<'source> Compile<'source> for ast::LogicOperator {
    type Output = LogicOperator;

    fn compile(self, _compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::LogicOperator::And => LogicOperator::And,
            ast::LogicOperator::Or => LogicOperator::Or,
        }
    }
}
impl<'source> Compile<'source> for ast::Comparator {
    type Output = Comparator;

    fn compile(self, _compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self {
            ast::Comparator::Equal => Comparator::Equal,
            ast::Comparator::NotEqual => Comparator::NotEqual,
            ast::Comparator::LessOrEqual => Comparator::LessOrEqual,
            ast::Comparator::Less => Comparator::Less,
            ast::Comparator::GreaterOrEqual => Comparator::GreaterOrEqual,
            ast::Comparator::Greater => Comparator::Greater,
        }
    }
}

pub(crate) trait CompileInto: Sized {
    // TODO seems like this should be generic over span providers to avoid eagerly generating spans?
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self>;
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self>;
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self>;
    fn compile_literal<'source>(
        literal: Spanned<ast::Literal<'source>>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        Self::coerce_literal(literal.data.compile(compiler)?, literal.span, compiler)
    }
}
impl CompileInto for CommandBoolean {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::Boolean(value) => Ok(CommandBoolean::Constant { value }),
            Literal::UberIdentifier(UberStateAlias {
                uber_identifier,
                value,
            }) => match value {
                None => match compiler.uber_state_type(uber_identifier, &span)? {
                    UberStateType::Boolean => Ok(CommandBoolean::FetchBoolean { uber_identifier }),
                    other => Err(uber_state_type_error(other, Type::Boolean, span)),
                },
                Some(value) => Ok(create_quest_command(uber_identifier, value)),
            },
            other => Err(type_error(other.literal_type(), Type::Boolean, span)),
        };
        compiler.consume_result(result)
    }
    // TODO a lot of compile_action implementations are really similar
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let result = match action {
            ast::Action::Function(function) => {
                let span = function.span();
                match function.compile(compiler)? {
                    Command::Boolean(function) => Ok(function),
                    _ => Err(return_type_error(Type::Boolean, span)),
                }
            }
            _ => Err(return_type_error(Type::Boolean, action.span())),
        };
        compiler.consume_result(result)
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        match operation.operator.data {
            Operator::Logic(operator) => {
                let left = operation.left.compile_into(compiler);
                let operator = operator.compile(compiler);
                let right = operation.right.compile_into(compiler);
                Some(match (left?, right?) {
                    (
                        CommandBoolean::Constant { value: left },
                        CommandBoolean::Constant { value: right },
                    ) => match operator {
                        LogicOperator::And => CommandBoolean::Constant {
                            value: left && right,
                        },
                        LogicOperator::Or => CommandBoolean::Constant {
                            value: left || right,
                        },
                    },
                    (left, right) => CommandBoolean::LogicOperation {
                        operation: Box::new(Operation {
                            left,
                            operator,
                            right,
                        }),
                    },
                })
            }
            Operator::Comparator(operator) => {
                let left = operation.left.infer_type(compiler)?;
                let operator = operator.compile(compiler);
                let right = operation.right.infer_type(compiler)?;
                let target = common_type(left, right);
                if target.is_none() {
                    compiler.errors.push(Error::custom(
                        format!("Cannot compare {left} and {right}"),
                        operation.span(),
                    ))
                }
                let target = target?;

                let expression = match target {
                    // TODO you may want to compare a lot more than these, especially at compile time, comparing config values or such
                    Type::Boolean | Type::String | Type::Zone => {
                        let operator = match operator {
                            Comparator::Equal => EqualityComparator::Equal,
                            Comparator::NotEqual => EqualityComparator::NotEqual,
                            other => {
                                compiler.errors.push(Error::custom(
                                    format!("Cannot use `{other}` on {target}"),
                                    operation.span(),
                                ));
                                return None;
                            }
                        };
                        match target {
                            // TODO code repetition much
                            Type::Boolean => {
                                let left = operation.left.compile_into(compiler);
                                let right = operation.right.compile_into(compiler);
                                match (left?, right?) {
                                    (
                                        CommandBoolean::Constant { value: left },
                                        CommandBoolean::Constant { value: right },
                                    ) => match operator {
                                        EqualityComparator::Equal => CommandBoolean::Constant {
                                            value: left == right,
                                        },
                                        EqualityComparator::NotEqual => CommandBoolean::Constant {
                                            value: left != right,
                                        },
                                    },
                                    (left, right) => CommandBoolean::CompareBoolean {
                                        operation: Box::new(Operation {
                                            left,
                                            operator,
                                            right,
                                        }),
                                    },
                                }
                            }
                            Type::String => {
                                let left = operation.left.compile_into(compiler);
                                let right = operation.right.compile_into(compiler);
                                match (left?, right?) {
                                    (
                                        CommandString::Constant { value: left },
                                        CommandString::Constant { value: right },
                                    ) => match operator {
                                        EqualityComparator::Equal => CommandBoolean::Constant {
                                            value: left == right,
                                        },
                                        EqualityComparator::NotEqual => CommandBoolean::Constant {
                                            value: left != right,
                                        },
                                    },
                                    (left, right) => CommandBoolean::CompareString {
                                        operation: Box::new(Operation {
                                            left,
                                            operator,
                                            right,
                                        }),
                                    },
                                }
                            }
                            Type::Zone => {
                                let left = operation.left.compile_into(compiler);
                                let right = operation.right.compile_into(compiler);
                                match (left?, right?) {
                                    (
                                        CommandZone::Constant { value: left },
                                        CommandZone::Constant { value: right },
                                    ) => match operator {
                                        EqualityComparator::Equal => CommandBoolean::Constant {
                                            value: left == right,
                                        },
                                        EqualityComparator::NotEqual => CommandBoolean::Constant {
                                            value: left != right,
                                        },
                                    },
                                    (left, right) => CommandBoolean::CompareZone {
                                        operation: Box::new(Operation {
                                            left,
                                            operator,
                                            right,
                                        }),
                                    },
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                    Type::Integer => {
                        let left = operation.left.compile_into(compiler);
                        let right = operation.right.compile_into(compiler);
                        match (left?, right?) {
                            (
                                CommandInteger::Constant { value: left },
                                CommandInteger::Constant { value: right },
                            ) => match operator {
                                Comparator::Equal => CommandBoolean::Constant {
                                    value: left == right,
                                },
                                Comparator::NotEqual => CommandBoolean::Constant {
                                    value: left != right,
                                },
                                Comparator::Less => CommandBoolean::Constant {
                                    value: left < right,
                                },
                                Comparator::LessOrEqual => CommandBoolean::Constant {
                                    value: left <= right,
                                },
                                Comparator::Greater => CommandBoolean::Constant {
                                    value: left > right,
                                },
                                Comparator::GreaterOrEqual => CommandBoolean::Constant {
                                    value: left >= right,
                                },
                            },
                            (left, right) => CommandBoolean::CompareInteger {
                                operation: Box::new(Operation {
                                    left,
                                    operator,
                                    right,
                                }),
                            },
                        }
                    }
                    Type::Float => {
                        let left = operation.left.compile_into(compiler);
                        let right = operation.right.compile_into(compiler);
                        match (left?, right?) {
                            (
                                CommandFloat::Constant { value: left },
                                CommandFloat::Constant { value: right },
                            ) => match operator {
                                Comparator::Equal => CommandBoolean::Constant {
                                    value: left == right,
                                },
                                Comparator::NotEqual => CommandBoolean::Constant {
                                    value: left != right,
                                },
                                Comparator::Less => CommandBoolean::Constant {
                                    value: left < right,
                                },
                                Comparator::LessOrEqual => CommandBoolean::Constant {
                                    value: left <= right,
                                },
                                Comparator::Greater => CommandBoolean::Constant {
                                    value: left > right,
                                },
                                Comparator::GreaterOrEqual => CommandBoolean::Constant {
                                    value: left >= right,
                                },
                            },
                            (left, right) => CommandBoolean::CompareFloat {
                                operation: Box::new(Operation {
                                    left,
                                    operator,
                                    right,
                                }),
                            },
                        }
                    }
                    other => {
                        compiler.errors.push(Error::custom(
                            format!("Cannot compare {other} values"),
                            operation.span(),
                        ));
                        return None;
                    }
                };
                Some(expression)
            }
            _ => {
                let found = operation.infer_type(compiler)?;
                compiler
                    .errors
                    .push(type_error(found, Type::Boolean, operation.span()));
                None
            }
        }
    }
}
impl CompileInto for CommandInteger {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::Integer(value) => Ok(CommandInteger::Constant { value }),
            Literal::UberIdentifier(UberStateAlias {
                uber_identifier,
                value,
            }) => match value {
                None => {
                    let inferred = compiler.uber_state_type(uber_identifier, &span)?;
                    match inferred {
                        UberStateType::Integer => {
                            Ok(CommandInteger::FetchInteger { uber_identifier })
                        }
                        _ => Err(uber_state_type_error(inferred, Type::Integer, span)),
                    }
                }
                Some(_) => Err(alias_type_error(
                    Type::Integer,
                    span,
                    uber_identifier,
                    compiler,
                )),
            },
            other => Err(type_error(other.literal_type(), Type::Integer, span)),
        };
        compiler.consume_result(result)
    }
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let result = match action {
            ast::Action::Function(function) => {
                let span = function.span();
                match function.compile(compiler)? {
                    Command::Integer(function) => Ok(function),
                    _ => Err(return_type_error(Type::Integer, span)),
                }
            }
            _ => Err(return_type_error(Type::Integer, action.span())),
        };
        compiler.consume_result(result)
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        match operation.operator.data {
            Operator::Arithmetic(operator) => {
                let left = operation.left.compile_into(compiler);
                let operator = operator.compile(compiler);
                let right = operation.right.compile_into(compiler);
                let command = match (left?, right?) {
                    (
                        CommandInteger::Constant { value: left },
                        CommandInteger::Constant { value: right },
                    ) => match operator {
                        ArithmeticOperator::Add => CommandInteger::Constant {
                            value: left + right,
                        },
                        ArithmeticOperator::Subtract => CommandInteger::Constant {
                            value: left - right,
                        },
                        ArithmeticOperator::Multiply => CommandInteger::Constant {
                            value: left * right,
                        },
                        ArithmeticOperator::Divide => CommandInteger::Constant {
                            value: left / right,
                        },
                    },
                    (left, right) => CommandInteger::Arithmetic {
                        operation: Box::new(Operation {
                            left,
                            operator,
                            right,
                        }),
                    },
                };
                Some(command)
            }
            _ => {
                let found = operation.infer_type(compiler)?;
                compiler
                    .errors
                    .push(type_error(found, Type::Integer, operation.span()));
                None
            }
        }
    }
}
impl CompileInto for CommandFloat {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::Float(value) => Ok(CommandFloat::Constant { value }),
            Literal::Integer(value) => Ok(CommandFloat::Constant {
                value: (value as f32).into(),
            }),
            Literal::UberIdentifier(UberStateAlias {
                uber_identifier,
                value,
            }) => match value {
                None => {
                    let inferred = compiler.uber_state_type(uber_identifier, &span)?;
                    match inferred {
                        UberStateType::Float => Ok(CommandFloat::FetchFloat { uber_identifier }),
                        UberStateType::Integer => Ok(CommandFloat::FromInteger {
                            integer: Box::new(CommandInteger::FetchInteger { uber_identifier }),
                        }),
                        _ => Err(uber_state_type_error(inferred, Type::Float, span)),
                    }
                }
                Some(_) => Err(alias_type_error(
                    Type::Float,
                    span,
                    uber_identifier,
                    compiler,
                )),
            },
            other => Err(type_error(other.literal_type(), Type::Float, span)),
        };
        compiler.consume_result(result)
    }
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let result = match action {
            ast::Action::Function(function) => {
                let span = function.span();
                match function.compile(compiler)? {
                    Command::Float(function) => Ok(function),
                    _ => Err(return_type_error(Type::Float, span)),
                }
            }
            _ => Err(return_type_error(Type::Float, action.span())),
        };
        compiler.consume_result(result)
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        match operation.operator.data {
            Operator::Arithmetic(operator) => {
                let left = operation.left.compile_into(compiler);
                let operator = operator.compile(compiler);
                let right = operation.right.compile_into(compiler);
                let command = match (left?, right?) {
                    (
                        CommandFloat::Constant { value: left },
                        CommandFloat::Constant { value: right },
                    ) => match operator {
                        ArithmeticOperator::Add => CommandFloat::Constant {
                            value: left + right,
                        },
                        ArithmeticOperator::Subtract => CommandFloat::Constant {
                            value: left - right,
                        },
                        ArithmeticOperator::Multiply => CommandFloat::Constant {
                            value: left * right,
                        },
                        ArithmeticOperator::Divide => CommandFloat::Constant {
                            value: left / right,
                        },
                    },
                    (left, right) => CommandFloat::Arithmetic {
                        operation: Box::new(Operation {
                            left,
                            operator,
                            right,
                        }),
                    },
                };
                Some(command)
            }
            _ => {
                let found = operation.infer_type(compiler)?;
                compiler
                    .errors
                    .push(type_error(found, Type::Float, operation.span()));
                None
            }
        }
    }
}
impl CompileInto for CommandString {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::UberIdentifier(UberStateAlias {
                uber_identifier,
                value,
            }) => match value {
                None => match compiler.uber_state_type(uber_identifier, &span)? {
                    UberStateType::Boolean => Ok(CommandString::FromBoolean {
                        boolean: Box::new(CommandBoolean::FetchBoolean { uber_identifier }),
                    }),
                    UberStateType::Integer => Ok(CommandString::FromInteger {
                        integer: Box::new(CommandInteger::FetchInteger { uber_identifier }),
                    }),
                    UberStateType::Float => Ok(CommandString::FromFloat {
                        float: Box::new(CommandFloat::FetchFloat { uber_identifier }),
                    }),
                },
                Some(value) => Ok(CommandString::FromBoolean {
                    boolean: Box::new(create_quest_command(uber_identifier, value)),
                }),
            },
            Literal::Boolean(value) => Ok(CommandString::Constant {
                value: value.to_string().into(),
            }),
            Literal::Integer(value) => Ok(CommandString::Constant {
                value: value.to_string().into(),
            }),
            Literal::Float(value) => Ok(CommandString::Constant {
                value: value.to_string().into(),
            }),
            Literal::String(value) => Ok(CommandString::Constant { value }),
            _ => Err(Error::custom("cannot convert to String".to_string(), span)),
        };
        compiler.consume_result(result)
    }
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let result = match action {
            ast::Action::Function(function) => {
                let span = function.span();
                match function.compile(compiler)? {
                    Command::Boolean(command) => match command {
                        CommandBoolean::Constant { value } => Ok(CommandString::Constant {
                            value: value.to_string().into(),
                        }),
                        _ => Ok(CommandString::FromBoolean {
                            boolean: Box::new(command),
                        }),
                    },
                    Command::Integer(command) => match command {
                        CommandInteger::Constant { value } => Ok(CommandString::Constant {
                            value: value.to_string().into(),
                        }),
                        _ => Ok(CommandString::FromInteger {
                            integer: Box::new(command),
                        }),
                    },
                    Command::Float(command) => match command {
                        CommandFloat::Constant { value } => Ok(CommandString::Constant {
                            value: value.to_string().into(),
                        }),
                        _ => Ok(CommandString::FromFloat {
                            float: Box::new(command),
                        }),
                    },
                    Command::String(command) => Ok(command),
                    _ => Err(return_type_error(Type::String, span)),
                }
            }
            _ => Err(return_type_error(Type::String, action.span())),
        };
        compiler.consume_result(result)
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        match operation.operator.data {
            Operator::Arithmetic(ast::ArithmeticOperator::Add) => {
                let left = operation.left.compile_into(compiler);
                let right = operation.right.compile_into(compiler);
                let command = match (left?, right?) {
                    (
                        CommandString::Constant {
                            value: StringOrPlaceholder::Value(left),
                        },
                        CommandString::Constant {
                            value: StringOrPlaceholder::Value(right),
                        },
                    ) => CommandString::Constant {
                        value: (left + &right).into(),
                    },
                    (left, right) => CommandString::Concatenate {
                        left: Box::new(left),
                        right: Box::new(right),
                    },
                };
                Some(command)
            }
            _ => {
                let found = operation.infer_type(compiler)?;
                compiler
                    .errors
                    .push(type_error(found, Type::String, operation.span()));
                None
            }
        }
    }
}
impl CompileInto for CommandZone {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::Constant(Constant::Zone(value)) => Ok(CommandZone::Constant { value }),
            other => Err(type_error(other.literal_type(), Type::Zone, span)),
        };
        compiler.consume_result(result)
    }
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let result = match action {
            ast::Action::Function(function) => {
                let span = function.span();
                match function.compile(compiler)? {
                    Command::Zone(function) => Ok(function),
                    _ => Err(return_type_error(Type::Zone, span)),
                }
            }
            _ => Err(return_type_error(Type::Zone, action.span())),
        };
        compiler.consume_result(result)
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let found = operation.infer_type(compiler)?;
        compiler
            .errors
            .push(type_error(found, Type::Zone, operation.span()));
        None
    }
}
impl CompileInto for Command {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let command = match literal {
            Literal::UberIdentifier(UberStateAlias {
                uber_identifier,
                value,
            }) => match value {
                None => {
                    let inferred = compiler.uber_state_type(uber_identifier, &span)?;
                    match inferred {
                        UberStateType::Boolean => {
                            Command::Boolean(CommandBoolean::FetchBoolean { uber_identifier })
                        }
                        UberStateType::Integer => {
                            Command::Integer(CommandInteger::FetchInteger { uber_identifier })
                        }
                        UberStateType::Float => {
                            Command::Float(CommandFloat::FetchFloat { uber_identifier })
                        }
                    }
                }
                Some(value) => Command::Boolean(create_quest_command(uber_identifier, value)),
            },
            Literal::Boolean(value) => Command::Boolean(CommandBoolean::Constant { value }),
            Literal::Integer(value) => Command::Integer(CommandInteger::Constant { value }),
            Literal::Float(value) => Command::Float(CommandFloat::Constant { value }),
            Literal::String(value) => Command::String(CommandString::Constant { value }),
            _ => todo!(),
        };
        Some(command)
    }
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        action.compile(compiler)
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        match operation.operator.data {
            Operator::Arithmetic(_) => {
                let left = operation.left.infer_type(compiler)?;
                let right = operation.right.infer_type(compiler)?;
                let target = common_type(left, right);
                if target.is_none() {
                    compiler.errors.push(Error::custom(
                        format!("Cannot perform operation on {left} and {right}"),
                        operation.span(),
                    ));
                };
                match target? {
                    Type::Boolean => {
                        CommandBoolean::compile_operation(operation, compiler).map(Self::Boolean)
                    }
                    Type::Integer => {
                        CommandInteger::compile_operation(operation, compiler).map(Self::Integer)
                    }
                    Type::Float => {
                        CommandFloat::compile_operation(operation, compiler).map(Self::Float)
                    }
                    Type::String => {
                        CommandString::compile_operation(operation, compiler).map(Self::String)
                    }
                    Type::Zone => {
                        CommandZone::compile_operation(operation, compiler).map(Self::Zone)
                    }
                    _ => {
                        compiler.errors.push(Error::custom(
                            format!("Cannot perform operation on {left} and {right}"),
                            operation.span(),
                        ));
                        None
                    }
                }
            }
            Operator::Logic(_) | Operator::Comparator(_) => {
                CommandBoolean::compile_operation(operation, compiler).map(Self::Boolean)
            }
        }
    }
}
impl CompileInto for usize {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        compiler
            .errors
            .push(type_error(literal.literal_type(), Type::Action, span));
        None
    }
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let span = action.span();
        let command = action.compile(compiler)?.expect_void(compiler, span)?;
        let index = compiler.global.output.command_lookup.len();
        compiler.global.output.command_lookup.push(command);
        Some(index)
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        let found = operation.infer_type(compiler)?;
        compiler
            .errors
            .push(type_error(found, Type::Action, operation.span()));
        None
    }
}

trait CompileIntoLiteral: Sized {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self>;
}
impl<T: CompileIntoLiteral> CompileInto for T {
    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        compiler
            .errors
            .push(Error::custom("expected literal".to_string(), action.span()));
        None
    }
    fn compile_operation<'source>(
        operation: ast::Operation<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        compiler.errors.push(Error::custom(
            "expected literal".to_string(),
            operation.span(),
        ));
        None
    }
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        T::coerce_literal(literal, span, compiler)
    }
}

impl CompileIntoLiteral for bool {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        match literal {
            Literal::Boolean(value) => Some(value),
            other => {
                compiler
                    .errors
                    .push(type_error(other.literal_type(), Type::Boolean, span));
                None
            }
        }
    }
}
impl CompileIntoLiteral for i32 {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        match literal {
            Literal::Integer(value) => Some(value),
            other => {
                compiler
                    .errors
                    .push(type_error(other.literal_type(), Type::Integer, span));
                None
            }
        }
    }
}
impl CompileIntoLiteral for OrderedFloat<f32> {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        match literal {
            Literal::Integer(value) => Some((value as f32).into()),
            Literal::Float(value) => Some(value),
            other => {
                compiler
                    .errors
                    .push(type_error(other.literal_type(), Type::Float, span));
                None
            }
        }
    }
}
impl CompileIntoLiteral for Icon {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let icon = match literal {
            Literal::Constant(Constant::Shard(value)) => Icon::Shard(value),
            Literal::Constant(Constant::Equipment(value)) => Icon::Equipment(value),
            Literal::Constant(Constant::OpherIcon(value)) => Icon::Opher(value),
            Literal::Constant(Constant::LupoIcon(value)) => Icon::Lupo(value),
            Literal::Constant(Constant::GromIcon(value)) => Icon::Grom(value),
            Literal::Constant(Constant::TuleyIcon(value)) => Icon::Tuley(value),
            Literal::IconAsset(path) => Icon::File(path),
            Literal::CustomIcon(path) => Icon::Bundle(path),
            other => {
                compiler
                    .errors
                    .push(type_error(other.literal_type(), Type::Icon, span));
                return None;
            }
        };
        Some(icon)
    }
}
macro_rules! impl_constants_coerce_from {
    ($ident: ident) => {
        impl CompileIntoLiteral for wotw_seedgen_data::$ident {
            fn coerce_literal(literal: Literal, span: Range<usize>, compiler: &mut SnippetCompiler) -> Option<Self> {
                match literal {
                    Literal::Constant(Constant::$ident(value)) => Some(value),
                    other => {
                        compiler.errors.push(type_error(other.literal_type(), Type::$ident, span));
                        None
                    },
                }
            }
        }
    };
    ($ident: ident, $($more: ident),+ $(,)?) => {
        impl_constants_coerce_from!($ident);
        impl_constants_coerce_from!($($more),+);
    };
}
impl_constants_coerce_from!(
    Skill,
    Shard,
    Teleporter,
    WeaponUpgrade,
    Zone,
    Equipment,
    EquipSlot,
    WheelItemPosition,
    WheelBind,
    OpherIcon,
    LupoIcon,
    GromIcon,
    TuleyIcon,
    MapIcon,
    Alignment,
    ScreenPosition,
);
impl CompileIntoLiteral for String {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::String(value) => match value {
                StringOrPlaceholder::Value(value) => Ok(value),
                _ => Err(Error::custom("expected string literal".to_string(), span)),
            },
            other => Err(type_error(other.literal_type(), Type::String, span)),
        };
        compiler.consume_result(result)
    }
}
impl CompileIntoLiteral for StringOrPlaceholder {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        match literal {
            Literal::String(value) => Some(value),
            other => {
                compiler
                    .errors
                    .push(type_error(other.literal_type(), Type::String, span));
                None
            }
        }
    }
}
impl CompileIntoLiteral for UberIdentifier {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::UberIdentifier(UberStateAlias {
                uber_identifier,
                value,
            }) => match value {
                None => Ok(uber_identifier),
                Some(_) => Err(alias_type_error(
                    Type::UberIdentifier,
                    span,
                    uber_identifier,
                    compiler,
                )),
            },
            other => Err(type_error(other.literal_type(), Type::UberIdentifier, span)),
        };
        compiler.consume_result(result)
    }
}

fn create_quest_command(uber_identifier: UberIdentifier, value: u8) -> CommandBoolean {
    CommandBoolean::CompareInteger {
        operation: Box::new(Operation {
            left: CommandInteger::FetchInteger { uber_identifier },
            operator: Comparator::GreaterOrEqual,
            right: CommandInteger::Constant {
                value: value as i32,
            },
        }),
    }
}

// TODO this could accept Option<Type> as found to still provide an error message if type inference fails
#[inline]
fn type_error(found: Type, expected: Type, span: Range<usize>) -> Error {
    Error::custom(format!("Expected {expected}, but found {found}"), span)
}
#[inline]
fn alias_type_error(
    expected: Type,
    span: Range<usize>,
    uber_identifier: UberIdentifier,
    compiler: &SnippetCompiler,
) -> Error {
    match compiler
        .global
        .uber_state_data
        .id_lookup
        .get(&uber_identifier)
    {
        None => Error::custom(
            "alias doesn't resolve to a valid UberIdentifier".to_string(),
            span,
        )
        .with_help("check the loc_data or state_data entry that defines this alias".to_string()),
        Some(uber_state) => type_error(Type::Boolean, expected, span).with_help(format!(
            "did you intend to use the underlying UberIdentifier {}?",
            uber_state.name
        )),
    }
}
#[inline]
fn return_type_error(expected: Type, span: Range<usize>) -> Error {
    Error::custom(format!("expected function returning {expected}"), span)
}
#[inline]
fn uber_state_type_error(found: UberStateType, expected: Type, span: Range<usize>) -> Error {
    let mut error = Error::custom(format!("cannot use {found} UberState as {expected}"), span);
    if matches!(expected, Type::Boolean) {
        error.help = Some(
            "if you want to trigger on every change of the state, use \"on change <UberIdentifier>\""
                .to_string(),
        )
    }
    error
}
