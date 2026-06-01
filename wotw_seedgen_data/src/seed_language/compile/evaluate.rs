use super::{expression::CompileInto, SnippetCompiler};
use crate::{
    seed_language::{
        ast,
        output::{
            Command, CommandBoolean, CommandFloat, CommandInteger, CommandString, CommandZone,
            Constant, IntoConstant, Literal, StringOrPlaceholder,
        },
    },
    Zone,
};
use wotw_seedgen_parse::{Error, Span};

pub(crate) trait EvaluateFrom: Sized {
    type From;

    fn evaluate(from: Self::From) -> Option<Self>;
}

impl<'source> ast::Expression<'source> {
    pub(crate) fn evaluate<T>(
        self,
        compiler: &mut SnippetCompiler<'source, '_, '_, '_>,
    ) -> Option<T>
    where
        T: EvaluateFrom,
        T::From: CompileInto,
    {
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
macro_rules! evaluate_from_into_constant {
    (($t:ty, $from:ty) $(,)?) => {
        impl EvaluateFrom for $t {
            type From = $from;

            fn evaluate(from: Self::From) -> Option<Self> {
                from.into_constant().ok()
            }
        }
    };

    (($t:ty, $from:ty), $($more:tt)+) => {
        evaluate_from_into_constant!(($t, $from));
        evaluate_from_into_constant!($($more)*);
    };
}

evaluate_from_into_constant!(
    (bool, CommandBoolean),
    (i32, CommandInteger),
    (f32, CommandFloat),
    (String, CommandString),
    (Zone, CommandZone),
);

impl EvaluateFrom for StringOrPlaceholder {
    type From = CommandString;

    fn evaluate(from: Self::From) -> Option<Self> {
        match from {
            Self::From::Constant { value } => Some(value),
            _ => None,
        }
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
