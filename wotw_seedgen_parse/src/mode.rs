pub trait ErrorMode {
    fn err<O: FnOnce()>(op: O);

    fn map_err<E, O: FnOnce(E)>(err: E, op: O);

    fn none<T, E: FnOnce()>(err: E) -> Option<T> {
        Self::err(err);
        None
    }

    fn consume_result<T, E, O: FnOnce(E)>(res: Result<T, E>, op: O) -> Option<T> {
        match res {
            Ok(t) => Some(t),
            Err(err) => {
                Self::map_err(err, op);
                None
            }
        }
    }
}

pub struct Errors;

impl ErrorMode for Errors {
    #[inline]
    fn err<E: FnOnce()>(err: E) {
        err()
    }

    #[inline]
    fn map_err<E, O: FnOnce(E)>(err: E, op: O) {
        op(err)
    }
}

pub struct NoErrors;

impl ErrorMode for NoErrors {
    #[inline]
    fn err<E: FnOnce()>(_err: E) {}

    #[inline]
    fn map_err<E, O: FnOnce(E)>(_err: E, _op: O) {}
}
