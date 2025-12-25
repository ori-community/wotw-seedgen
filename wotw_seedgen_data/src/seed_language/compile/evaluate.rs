use super::{expression::CompileInto, SnippetCompiler};
use crate::{
    seed_language::{
        ast,
        output::{
            AsConstant, Command, CommandBoolean, CommandFloat, CommandInteger, CommandString,
            CommandZone, Constant, Literal, StringOrPlaceholder,
        },
    },
    Zone,
};
use ordered_float::OrderedFloat;
use wotw_seedgen_parse::{Error, Span};

pub(crate) trait EvaluateFrom: Sized {
    type From: CompileInto;

    fn evaluate(from: Self::From) -> Option<Self>;
}

impl<'source> ast::Expression<'source> {
    pub(crate) fn evaluate<T: EvaluateFrom>(
        self,
        compiler: &mut SnippetCompiler<'_, 'source, '_, '_>,
    ) -> Option<T> {
        let span = self.span();
        let value = T::evaluate(self.compile_into(compiler)?);

        if value.is_none() {
            compiler.errors.push(Error::error(
                "Cannot be statically evaluated".to_string(),
                span,
            ));
        }

        value
    }
}

impl EvaluateFrom for bool {
    type From = CommandBoolean;

    fn evaluate(from: Self::From) -> Option<Self> {
        from.as_constant().copied()
    }
}

impl EvaluateFrom for i32 {
    type From = CommandInteger;

    fn evaluate(from: Self::From) -> Option<Self> {
        from.as_constant().copied()
    }
}

impl EvaluateFrom for OrderedFloat<f32> {
    type From = CommandFloat;

    fn evaluate(from: Self::From) -> Option<Self> {
        from.as_constant().copied()
    }
}

impl EvaluateFrom for String {
    type From = CommandString;

    fn evaluate(from: Self::From) -> Option<Self> {
        from.as_constant().cloned()
    }
}

impl EvaluateFrom for StringOrPlaceholder {
    type From = CommandString;

    fn evaluate(from: Self::From) -> Option<Self> {
        match from {
            Self::From::Constant { value } => Some(value),
            _ => None,
        }
    }
}

impl EvaluateFrom for Zone {
    type From = CommandZone;

    fn evaluate(from: Self::From) -> Option<Self> {
        from.as_constant().copied()
    }
}

impl EvaluateFrom for Literal {
    type From = Command;

    fn evaluate(from: Self::From) -> Option<Self> {
        match from {
            Command::Boolean(CommandBoolean::Constant { value }) => Some(Literal::Boolean(value)),
            Command::Integer(CommandInteger::Constant { value }) => Some(Literal::Integer(value)),
            Command::Float(CommandFloat::Constant { value }) => Some(Literal::Float(value)),
            Command::String(CommandString::Constant { value }) => Some(Literal::String(value)),
            Command::Zone(CommandZone::Constant { value }) => {
                Some(Literal::Constant(Constant::Zone(value)))
            }
            _ => None,
        }
    }
}
