use std::ops::Range;

use wotw_seedgen_parse::{Error, SpanEnd, SpanStart};

use crate::{
    seed_language::{
        ast::{Expression, UberStateType},
        compile::SnippetCompiler,
        types::Type,
    },
    UberIdentifier,
};

// TODO this could accept Option<Type> as found to still provide an error message if type inference fails
#[inline]
pub fn type_error(found: Type, expected: Type, span: Range<usize>) -> Error {
    Error::error(type_error_message(found, expected), span)
}

#[inline]
pub fn type_error_message(found: Type, expected: Type) -> String {
    format!("expected {expected}, but found {found}")
}

#[inline]
pub fn alias_type_error(
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
        None => Error::error(
            "alias doesn't resolve to a valid UberIdentifier".to_string(),
            span,
        )
        .with_help("check the loc_data or state_data entry that defines this alias".to_string()),
        Some(uber_state) => type_error(Type::Boolean, expected, span).with_help(format!(
            "this alias resolves to an integer comparison, maybe you can use the underlying UberIdentifier {}?",
            uber_state.preferred_name()
        )),
    }
}

#[inline]
pub fn operation_error(target: Type, span: Range<usize>) -> Error {
    Error::error(operation_error_message(target), span)
}

#[inline]
pub fn operation_error_message(target: Type) -> String {
    format!("Cannot perform operation on {target}")
}

#[inline]
pub fn operand_error(
    left_ty: Type,
    right_ty: Type,
    left: &Expression,
    right: &Expression,
) -> Error {
    Error::error(
        format!("Cannot perform operation on {left_ty} and {right_ty}"),
        left.span_start()..right.span_end(),
    )
}

#[inline]
pub fn uber_state_type_error(found: UberStateType, expected: Type, span: Range<usize>) -> Error {
    let mut error = Error::error(format!("cannot use {found} UberState as {expected}"), span);

    if matches!(expected, Type::Boolean) {
        error.help = Some(
            "if you want to trigger on every change of the state, use \"on change <UberIdentifier>\""
                .to_string(),
        );
    }

    error
}
