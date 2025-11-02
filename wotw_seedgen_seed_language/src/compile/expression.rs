use super::{Compile, SnippetCompiler};
use crate::{
    ast::{self, UberStateType},
    output::{
        ArithmeticOperator, AsConstant, Command, CommandBoolean, CommandFloat, CommandInteger,
        CommandString, CommandVoid, CommandZone, Comparator, Concatenator, Constant,
        EqualityComparator, ExecuteOperator, Literal, LogicOperator, Operation,
        StringOrPlaceholder,
    },
    types::Type,
};
use ordered_float::OrderedFloat;
use std::{borrow::Cow, ops::Range};
use wotw_seedgen_assets::UberStateAlias;
use wotw_seedgen_data::{
    Alignment, CoordinateSystem, EquipSlot, Equipment, GromIcon, HorizontalAnchor, Icon, LupoIcon,
    MapIcon, OpherIcon, ScreenPosition, Shard, Skill, Teleporter, TuleyIcon, UberIdentifier,
    VerticalAnchor, WeaponUpgrade, WheelBind, WheelItemPosition, Zone,
};
use wotw_seedgen_parse::{Error, Span, Spanned};

impl Command {
    // TODO unidiomatic naming
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
            ast::Expression::Operation(operation) => T::compile_command(*operation, compiler),
        }
    }
}

impl<'source> ast::ExpressionValue<'source> {
    pub(crate) fn compile_into<T: CompileInto>(
        self,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<T> {
        match self {
            ast::ExpressionValue::Group(group) => {
                compiler.consume_delimited(group)?.0.compile_into(compiler)
            }
            ast::ExpressionValue::Action(action) => T::compile_action(action, compiler),
            ast::ExpressionValue::Literal(literal) => T::compile_literal(literal, compiler),
            ast::ExpressionValue::Identifier(identifier) => compiler
                .resolve(&identifier)
                .cloned()
                .and_then(|literal| T::coerce_literal(literal, identifier.span, compiler)),
        }
    }
}

impl<'source> Compile<'source> for ast::Operation<'source> {
    type Output = Option<Command>;

    fn compile(self, compiler: &mut SnippetCompiler<'_, 'source, '_, '_>) -> Self::Output {
        match self.operator.data {
            ast::Operator::Arithmetic(operator) => {
                let operator = operator.compile(compiler);
                let target = compiler.common_type(&self.left, &self.right)?;

                match target {
                    Type::Integer => self
                        .compile_operation(operator, compiler)
                        .map(Command::Integer),
                    Type::Float => self
                        .compile_operation(operator, compiler)
                        .map(Command::Float),
                    Type::String => match Concatenator::try_from(operator) {
                        Ok(operator) => self
                            .compile_operation(operator, compiler)
                            .map(Command::String),
                        Err(()) => {
                            compiler.errors.push(operation_error(target, self.span()));
                            None
                        }
                    },
                    _ => {
                        compiler.errors.push(operation_error(target, self.span()));
                        None
                    }
                }
            }
            ast::Operator::Logic(operator) => {
                let operator = operator.compile(compiler);

                self.compile_operation(operator, compiler)
                    .map(Command::Boolean)
            }
            ast::Operator::Comparator(operator) => {
                let operator = operator.compile(compiler);
                let target = compiler.common_type(&self.left, &self.right)?;

                match target {
                    Type::Boolean => match EqualityComparator::try_from(operator) {
                        Ok(operator) => self
                            .compile_operation::<CommandBoolean, _, _>(operator, compiler)
                            .map(Command::Boolean),
                        Err(()) => {
                            compiler.errors.push(operation_error(target, self.span()));
                            None
                        }
                    },
                    Type::Integer => self
                        .compile_operation::<CommandInteger, _, _>(operator, compiler)
                        .map(Command::Boolean),
                    Type::Float => self
                        .compile_operation::<CommandFloat, _, _>(operator, compiler)
                        .map(Command::Boolean),
                    Type::String => match operator.try_into() {
                        Ok(operator) => self
                            .compile_operation::<CommandString, _, _>(operator, compiler)
                            .map(Command::Boolean),
                        Err(()) => {
                            compiler.errors.push(operation_error(target, self.span()));
                            None
                        }
                    },
                    Type::Zone => match operator.try_into() {
                        Ok(operator) => self
                            .compile_operation::<CommandZone, _, _>(operator, compiler)
                            .map(Command::Boolean),
                        Err(()) => {
                            compiler.errors.push(operation_error(target, self.span()));
                            None
                        }
                    },
                    _ => {
                        compiler.errors.push(operation_error(target, self.span()));
                        None
                    }
                }
            }
        }
    }
}

impl<'source> ast::Operation<'source> {
    fn compile_operation<Item, Operator, Output>(
        self,
        operator: Operator,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Output>
    where
        Item: CompileInto + AsConstant,
        Item::Output: Clone,
        Operator: ExecuteOperator<Item::Output>,
        Operator::Output: Into<Output>,
        Operation<Item, Operator>: Into<Output>,
    {
        let left = self.left.compile_into::<Item>(compiler);
        let right = self.right.compile_into::<Item>(compiler);

        let (left, right) = (left?, right?);

        match (left.as_constant(), right.as_constant()) {
            (Some(left), Some(right)) => Some(operator.execute(left.clone(), right.clone()).into()),
            _ => Some(
                Operation {
                    left,
                    operator,
                    right,
                }
                .into(),
            ),
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
    fn coerce_command(command: Command) -> Result<Self, String>;

    // TODO seems like this should be generic over span providers to avoid eagerly generating spans?
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self>;

    fn compile_action<'source>(
        action: ast::Action<'source>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        Self::compile_command(action, compiler)
    }

    fn compile_command<'source, T>(
        ast: T,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self>
    where
        T: Compile<'source, Output = Option<Command>> + Span,
    {
        let span = ast.span();
        let command = ast.compile(compiler)?;

        Self::coerce_command(command)
            .map_err(|message| compiler.errors.push(Error::custom(message, span)))
            .ok()
    }

    fn compile_literal<'source>(
        literal: Spanned<ast::Literal<'source>>,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<Self> {
        Self::coerce_literal(literal.data.compile(compiler)?, literal.span, compiler)
    }
}

impl CompileInto for CommandBoolean {
    fn coerce_command(command: Command) -> Result<Self, String> {
        match command {
            Command::Boolean(command) => Ok(command),
            other => Err(type_error_message(other.command_type(), Type::Boolean)),
        }
    }

    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::Boolean(value) => Ok(value.into()),
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
}

impl CompileInto for CommandInteger {
    fn coerce_command(command: Command) -> Result<Self, String> {
        match command {
            Command::Integer(command) => Ok(command),
            other => Err(type_error_message(other.command_type(), Type::Integer)),
        }
    }

    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::Integer(value) => Ok(value.into()),
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
}

impl CompileInto for CommandFloat {
    fn coerce_command(command: Command) -> Result<Self, String> {
        match command {
            Command::Integer(command) => match command.as_constant() {
                Some(value) => Ok((*value as f32).into()),
                None => Ok(CommandFloat::FromInteger {
                    integer: Box::new(command),
                }),
            },
            Command::Float(command) => Ok(command),
            other => Err(type_error_message(other.command_type(), Type::Float)),
        }
    }

    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = match literal {
            Literal::Float(value) => Ok(value.into()),
            Literal::Integer(value) => Ok((value as f32).into()),
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
}

impl CompileInto for CommandString {
    fn coerce_command(command: Command) -> Result<Self, String> {
        match command {
            Command::Boolean(command) => match command.as_constant() {
                Some(value) => Ok(value.to_string().into()),
                None => Ok(CommandString::FromBoolean {
                    boolean: Box::new(command),
                }),
            },
            Command::Integer(command) => match command.as_constant() {
                Some(value) => Ok(value.to_string().into()),
                None => Ok(CommandString::FromInteger {
                    integer: Box::new(command),
                }),
            },
            Command::Float(command) => match command.as_constant() {
                Some(value) => Ok(value.to_string().into()),
                None => Ok(CommandString::FromFloat {
                    float: Box::new(command),
                }),
            },
            Command::String(command) => Ok(command),
            other => Err(type_error_message(other.command_type(), Type::String)),
        }
    }

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
            Literal::Boolean(value) => Ok(value.to_string().into()),
            Literal::Integer(value) => Ok(value.to_string().into()),
            Literal::Float(value) => Ok(value.to_string().into()),
            Literal::String(value) => Ok(value.into()),
            _ => Err(Error::custom("cannot convert to String".to_string(), span)),
        };

        compiler.consume_result(result)
    }
}

impl CompileInto for CommandZone {
    fn coerce_command(command: Command) -> Result<Self, String> {
        match command {
            Command::Zone(command) => Ok(command),
            other => Err(type_error_message(other.command_type(), Type::Zone)),
        }
    }

    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        <Zone as CompileInto>::coerce_literal(literal, span, compiler).map(Self::from)
    }
}

impl CompileInto for Command {
    fn coerce_command(command: Command) -> Result<Self, String> {
        Ok(command)
    }

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
            Literal::Boolean(value) => Command::Boolean(value.into()),
            Literal::Integer(value) => Command::Integer(value.into()),
            Literal::Float(value) => Command::Float(value.into()),
            Literal::String(value) => Command::String(value.into()),
            _ => todo!(),
        };

        Some(command)
    }
}

impl CompileInto for usize {
    fn coerce_command(_command: Command) -> Result<Self, String> {
        unimplemented!()
    }

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
}

trait CompileIntoLiteral: Sized {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error>;
}

impl<T: CompileIntoLiteral> CompileInto for T {
    fn coerce_command(_command: Command) -> Result<Self, String> {
        Err("expected literal".to_string())
    }

    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Option<Self> {
        let result = T::coerce_literal(literal, span, compiler);
        compiler.consume_result(result)
    }
}

impl CompileIntoLiteral for bool {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        match literal {
            Literal::Boolean(value) => Ok(value),
            other => Err(type_error(other.literal_type(), Type::Boolean, span)),
        }
    }
}

impl CompileIntoLiteral for i32 {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        match literal {
            Literal::Integer(value) => Ok(value),
            other => Err(type_error(other.literal_type(), Type::Integer, span)),
        }
    }
}

impl CompileIntoLiteral for OrderedFloat<f32> {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        match literal {
            Literal::Integer(value) => Ok((value as f32).into()),
            Literal::Float(value) => Ok(value),
            other => Err(type_error(other.literal_type(), Type::Float, span)),
        }
    }
}

impl CompileIntoLiteral for Icon {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        match literal {
            Literal::Constant(Constant::Shard(value)) => Ok(Icon::Shard(value)),
            Literal::Constant(Constant::LupoIcon(value)) => Ok(Icon::Lupo(value)),
            Literal::Constant(Constant::GromIcon(value)) => Ok(Icon::Grom(value)),
            Literal::Constant(Constant::TuleyIcon(value)) => Ok(Icon::Tuley(value)),
            Literal::Constant(constant) => Equipment::coerce_constant(constant)
                .map(Icon::Equipment)
                .or_else(|| OpherIcon::coerce_constant(constant).map(Icon::Opher))
                .ok_or_else(|| type_error(constant.literal_type(), Type::Icon, span)),
            Literal::IconAsset(path) => Ok(Icon::File(Cow::Owned(path))),
            Literal::CustomIcon(path) => Ok(Icon::Bundle(path)),
            other => Err(type_error(other.literal_type(), Type::Icon, span)),
        }
    }
}

impl CompileIntoLiteral for String {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        match literal {
            Literal::String(value) => match value {
                StringOrPlaceholder::Value(value) => Ok(value),
                _ => Err(Error::custom("expected string literal".to_string(), span)),
            },
            other => Err(type_error(other.literal_type(), Type::String, span)),
        }
    }
}

impl CompileIntoLiteral for StringOrPlaceholder {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        match literal {
            Literal::String(value) => Ok(value),
            other => Err(type_error(other.literal_type(), Type::String, span)),
        }
    }
}

impl CompileIntoLiteral for UberIdentifier {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        match literal {
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
        }
    }
}

trait CompileIntoConstant: Sized {
    const TYPE: Type;

    fn coerce_constant(constant: Constant) -> Option<Self>;
}

impl<T: CompileIntoConstant> CompileIntoLiteral for T {
    fn coerce_literal(
        literal: Literal,
        span: Range<usize>,
        _compiler: &mut SnippetCompiler,
    ) -> Result<Self, Error> {
        let t = match &literal {
            Literal::Constant(constant) => T::coerce_constant(*constant),
            _ => None,
        };

        t.ok_or_else(|| type_error(literal.literal_type(), T::TYPE, span))
    }
}

impl CompileIntoConstant for Skill {
    const TYPE: Type = Type::Skill;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::Skill(skill) => Some(skill),
            _ => None,
        }
    }
}

impl CompileIntoConstant for Shard {
    const TYPE: Type = Type::Shard;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::Shard(shard) => Some(shard),
            _ => None,
        }
    }
}

impl CompileIntoConstant for Teleporter {
    const TYPE: Type = Type::Teleporter;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::Teleporter(teleporter) => Some(teleporter),
            _ => None,
        }
    }
}

impl CompileIntoConstant for WeaponUpgrade {
    const TYPE: Type = Type::WeaponUpgrade;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::WeaponUpgrade(weapon_upgrade) => Some(weapon_upgrade),
            _ => None,
        }
    }
}

impl CompileIntoConstant for Equipment {
    const TYPE: Type = Type::Equipment;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::Skill(skill) => skill.equipment(),
            Constant::Equipment(equipment) => Some(equipment),
            _ => None,
        }
    }
}

impl CompileIntoConstant for Zone {
    const TYPE: Type = Type::Zone;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::Teleporter(teleporter) => match teleporter {
                Teleporter::Marsh => Some(Zone::Marsh),
                Teleporter::Hollow => Some(Zone::Hollow),
                Teleporter::Glades => Some(Zone::Glades),
                Teleporter::Wellspring => Some(Zone::Wellspring),
                Teleporter::Burrows => Some(Zone::Burrows),
                Teleporter::Reach => Some(Zone::Reach),
                Teleporter::Depths => Some(Zone::Depths),
                Teleporter::Willow => Some(Zone::Willow),
                Teleporter::Den
                | Teleporter::WoodsEntrance
                | Teleporter::WoodsExit
                | Teleporter::CentralPools
                | Teleporter::PoolsBoss
                | Teleporter::FeedingGrounds
                | Teleporter::CentralWastes
                | Teleporter::OuterRuins
                | Teleporter::InnerRuins
                | Teleporter::Shriek => None,
            },
            Constant::Zone(zone) => Some(zone),
            _ => None,
        }
    }
}

impl CompileIntoConstant for EquipSlot {
    const TYPE: Type = Type::EquipSlot;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::EquipSlot(equip_slot) => Some(equip_slot),
            _ => None,
        }
    }
}

impl CompileIntoConstant for WheelItemPosition {
    const TYPE: Type = Type::WheelItemPosition;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::WheelItemPosition(wheel_item_position) => Some(wheel_item_position),
            _ => None,
        }
    }
}

impl CompileIntoConstant for WheelBind {
    const TYPE: Type = Type::WheelBind;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::EquipSlot(equip_slot) => Some(match equip_slot {
                EquipSlot::Ability1 => WheelBind::Ability1,
                EquipSlot::Ability2 => WheelBind::Ability2,
                EquipSlot::Ability3 => WheelBind::Ability3,
            }),
            Constant::WheelBind(wheel_bind) => Some(wheel_bind),
            _ => None,
        }
    }
}

impl CompileIntoConstant for OpherIcon {
    const TYPE: Type = Type::OpherIcon;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::Skill(skill) => skill.opher_icon(),
            Constant::WeaponUpgrade(weapon_upgrade) => Some(weapon_upgrade.opher_icon()),
            Constant::OpherIcon(opher_icon) => Some(opher_icon),
            _ => None,
        }
    }
}

impl CompileIntoConstant for LupoIcon {
    const TYPE: Type = Type::LupoIcon;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::LupoIcon(lupo_icon) => Some(lupo_icon),
            _ => None,
        }
    }
}

impl CompileIntoConstant for GromIcon {
    const TYPE: Type = Type::GromIcon;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::GromIcon(grom_icon) => Some(grom_icon),
            _ => None,
        }
    }
}

impl CompileIntoConstant for TuleyIcon {
    const TYPE: Type = Type::TuleyIcon;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::TuleyIcon(tuley_icon) => Some(tuley_icon),
            _ => None,
        }
    }
}

impl CompileIntoConstant for MapIcon {
    const TYPE: Type = Type::MapIcon;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::MapIcon(map_icon) => Some(map_icon),
            _ => None,
        }
    }
}

impl CompileIntoConstant for Alignment {
    const TYPE: Type = Type::Alignment;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::WheelItemPosition(wheel_item_position) => match wheel_item_position {
                WheelItemPosition::Right => Some(Alignment::Right),
                WheelItemPosition::Left => Some(Alignment::Left),
                _ => None,
            },
            Constant::Alignment(alignment) => Some(alignment),
            _ => None,
        }
    }
}

impl CompileIntoConstant for HorizontalAnchor {
    const TYPE: Type = Type::HorizontalAnchor;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::WheelItemPosition(wheel_item_position) => match wheel_item_position {
                WheelItemPosition::Right => Some(HorizontalAnchor::Right),
                WheelItemPosition::Left => Some(HorizontalAnchor::Left),
                _ => None,
            },
            Constant::Alignment(alignment) => match alignment {
                Alignment::Left => Some(HorizontalAnchor::Left),
                Alignment::Center => Some(HorizontalAnchor::Center),
                Alignment::Right => Some(HorizontalAnchor::Right),
                _ => None,
            },
            Constant::HorizontalAnchor(horizontal_anchor) => Some(horizontal_anchor),
            _ => None,
        }
    }
}

impl CompileIntoConstant for VerticalAnchor {
    const TYPE: Type = Type::VerticalAnchor;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::WheelItemPosition(wheel_item_position) => match wheel_item_position {
                WheelItemPosition::Top => Some(VerticalAnchor::Top),
                WheelItemPosition::Bottom => Some(VerticalAnchor::Bottom),
                _ => None,
            },
            Constant::VerticalAnchor(vertical_anchor) => Some(vertical_anchor),
            _ => None,
        }
    }
}

impl CompileIntoConstant for ScreenPosition {
    const TYPE: Type = Type::ScreenPosition;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::WheelItemPosition(wheel_item_position) => match wheel_item_position {
                WheelItemPosition::TopRight => Some(ScreenPosition::TopRight),
                WheelItemPosition::BottomRight => Some(ScreenPosition::BottomRight),
                WheelItemPosition::BottomLeft => Some(ScreenPosition::BottomLeft),
                WheelItemPosition::TopLeft => Some(ScreenPosition::TopLeft),
                _ => None,
            },
            Constant::ScreenPosition(screen_position) => Some(screen_position),
            _ => None,
        }
    }
}

impl CompileIntoConstant for CoordinateSystem {
    const TYPE: Type = Type::CoordinateSystem;

    fn coerce_constant(constant: Constant) -> Option<Self> {
        match constant {
            Constant::CoordinateSystem(coordinate_system) => Some(coordinate_system),
            _ => None,
        }
    }
}

fn create_quest_command(uber_identifier: UberIdentifier, value: i32) -> CommandBoolean {
    CommandBoolean::CompareInteger {
        operation: Box::new(Operation {
            left: CommandInteger::FetchInteger { uber_identifier },
            operator: Comparator::GreaterOrEqual,
            right: value.into(),
        }),
    }
}

// TODO this could accept Option<Type> as found to still provide an error message if type inference fails
#[inline]
fn type_error(found: Type, expected: Type, span: Range<usize>) -> Error {
    Error::custom(type_error_message(found, expected), span)
}

#[inline]
fn type_error_message(found: Type, expected: Type) -> String {
    format!("expected {expected}, but found {found}")
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
fn operation_error(target: Type, span: Range<usize>) -> Error {
    Error::custom(operation_error_message(target), span)
}

#[inline]
fn operation_error_message(target: Type) -> String {
    format!("Cannot perform operation on {target}")
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
