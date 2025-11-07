use std::ops::ControlFlow;

use crate::{Error, Result};

pub trait Mode {
    type Output<T>;
    type Error;

    fn output<T>(ctr: ControlFlow<Self::Error, T>) -> Self::Output<T>;

    fn err<E: FnOnce() -> Error>(err: E) -> Self::Error;

    fn map_err<E, O: FnOnce(E) -> Error>(err: E, op: O) -> Self::Error;

    fn combine_errors(errors: Vec<Self::Error>) -> Self::Error;

    fn opt<T, E>(opt: Option<T>, err: E) -> ControlFlow<Self::Error, T>
    where
        E: FnOnce() -> Error,
    {
        match opt {
            None => ControlFlow::Break(Self::err(err)),
            Some(t) => ControlFlow::Continue(t),
        }
    }

    fn res<T, E, O>(res: std::result::Result<T, E>, op: O) -> ControlFlow<Self::Error, T>
    where
        O: FnOnce(E) -> Error,
    {
        match res {
            Err(err) => ControlFlow::Break(Self::map_err(err, op)),
            Ok(t) => ControlFlow::Continue(t),
        }
    }
}

pub struct ResultMode;

impl Mode for ResultMode {
    type Output<T> = Result<T>;
    type Error = Error;

    fn output<T>(ctr: ControlFlow<Self::Error, T>) -> Self::Output<T> {
        match ctr {
            ControlFlow::Continue(t) => Ok(t),
            ControlFlow::Break(err) => Err(err),
        }
    }

    fn err<E: FnOnce() -> Error>(err: E) -> Self::Error {
        err()
    }

    fn map_err<E, O: FnOnce(E) -> Error>(err: E, op: O) -> Self::Error {
        op(err)
    }

    fn combine_errors(errors: Vec<Self::Error>) -> Self::Error {
        Error::all_failed(errors)
    }
}

pub struct OptionMode;

impl Mode for OptionMode {
    type Output<T> = Option<T>;
    type Error = ();

    fn output<T>(ctr: ControlFlow<Self::Error, T>) -> Self::Output<T> {
        ctr.continue_value()
    }

    fn err<E: FnOnce() -> Error>(_err: E) -> Self::Error {}

    fn map_err<E, O: FnOnce(E) -> Error>(_err: E, _op: O) -> Self::Error {}

    fn combine_errors(_errors: Vec<Self::Error>) -> Self::Error {}
}
