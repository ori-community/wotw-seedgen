use wotw_seedgen_data::UberIdentifier;

use crate::{
    ast::ArithmeticOperator,
    output::{CommandBoolean, CommandFloat, CommandInteger, CommandVoid, Operation},
};

// TODO search for `Box::new(Operation` and similar, these helpers are often ignored
// TODO move modify_health etc. helpers into here?
pub fn store_boolean<T>(uber_identifier: UberIdentifier, value: T) -> CommandVoid
where
    T: Into<CommandBoolean>,
{
    CommandVoid::StoreBoolean {
        uber_identifier,
        value: value.into(),
        trigger_events: true,
    }
}

pub fn store_integer<T>(uber_identifier: UberIdentifier, value: T) -> CommandVoid
where
    T: Into<CommandInteger>,
{
    CommandVoid::StoreInteger {
        uber_identifier,
        value: value.into(),
        trigger_events: true,
    }
}

pub fn add_integer<T>(uber_identifier: UberIdentifier, add: T) -> CommandVoid
where
    T: Into<CommandInteger>,
{
    CommandVoid::StoreInteger {
        uber_identifier,
        value: CommandInteger::Arithmetic {
            operation: Box::new(Operation {
                left: CommandInteger::FetchInteger { uber_identifier },
                operator: ArithmeticOperator::Add,
                right: add.into(),
            }),
        },
        trigger_events: true,
    }
}

pub fn store_float<T>(uber_identifier: UberIdentifier, value: T) -> CommandVoid
where
    T: Into<CommandFloat>,
{
    CommandVoid::StoreFloat {
        uber_identifier,
        value: value.into(),
        trigger_events: true,
    }
}

pub fn add_float<T>(uber_identifier: UberIdentifier, add: T) -> CommandVoid
where
    T: Into<CommandFloat>,
{
    CommandVoid::StoreFloat {
        uber_identifier,
        value: CommandFloat::Arithmetic {
            operation: Box::new(Operation {
                left: CommandFloat::FetchFloat { uber_identifier },
                operator: ArithmeticOperator::Add,
                right: add.into(),
            }),
        },
        trigger_events: true,
    }
}
