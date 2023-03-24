use std::fmt::Display;

pub(crate) struct HandleErrors<T, E, I: Iterator<Item = std::result::Result<T, E>>, F: FnMut(E)> {
    iter: I,
    handler: F,
    pub errors: usize,
    printed_error_count: bool,
}
impl<T, E, I: Iterator<Item = std::result::Result<T, E>>, F: FnMut(E)> HandleErrors<T, E, I, F> {
    pub(crate) fn new(iter: I, handler: F) -> Self {
        Self {
            iter,
            handler,
            errors: 0,
            printed_error_count: false,
        }
    }
}
impl<T, E: Display, I: Iterator<Item = std::result::Result<T, E>>> HandleErrors<T, E, I, fn(E)> {
    pub(crate) fn new_print_errors(iter: I) -> Self {
        Self {
            iter,
            handler: |err| eprintln!("{err}"),
            errors: 0,
            printed_error_count: false,
        }
    }
}
impl<T, E, I: Iterator<Item = std::result::Result<T, E>>, F: FnMut(E)> Iterator
    for HandleErrors<T, E, I, F>
{
    type Item = T;

    fn next(&mut self) -> Option<Self::Item> {
        loop {
            match self.iter.next() {
                None => {
                    if !self.printed_error_count && self.errors > 10 {
                        let more = self.errors - 10;
                        eprintln!(
                            "...{} more error{} omitted",
                            more,
                            if more == 1 { "" } else { "s" }
                        );
                        self.printed_error_count = true;
                    }
                    return None;
                }
                Some(result) => match result {
                    Ok(item) => return Some(item),
                    Err(err) => {
                        if self.errors < 10 {
                            (self.handler)(err);
                        }
                        self.errors += 1;
                    }
                },
            }
        }
    }
}
