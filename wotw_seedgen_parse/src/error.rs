use itertools::Itertools;
use std::{
    error,
    fmt::{self, Display},
    io,
    ops::Range,
};
use wotw_seedgen_assets::Source;

// TODO the Error type is pretty big! It seems like Boxing it would reduce type sizes by a lot in the current ASTs.
pub type Result<T> = std::result::Result<T, Error>;

/// An Error returned from an [`Ast`] implementation, including the span the error originated from.
///
/// If you wish, you can reuse this Error type directly as your compilation Error by utilizing [`Error::custom`].
///
/// The [`Display`] implementation of [`Error`] may not provide the best error messages since it will not reference the source file.
/// You may provide the [`Source`] file to [`Error::with_source`] or (with the ariadne feature) [`Error::write_pretty`] for better error displays.
///
/// [`Ast`]: crate::Ast
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Error {
    /// Information about what went wrong
    pub kind: ErrorKind,
    /// The span in the source this `Error` originated from
    pub span: Range<usize>,
    /// An optional help message
    pub help: Option<String>,
}
impl Error {
    /// Returns an [`Error`] with the given values.
    #[inline]
    pub fn new(kind: ErrorKind, span: Range<usize>) -> Self {
        Self {
            kind,
            span,
            help: None,
        }
    }
    /// Returns an [`Error`] with a custom message.
    /// Convenience wrapper for [`Error::new`] with [`ErrorKind::Other`]
    #[inline]
    pub fn custom(message: String, span: Range<usize>) -> Self {
        Self::new(ErrorKind::Other(message), span)
    }
    /// Sets the help message
    #[inline]
    pub fn with_help(self, help: String) -> Self {
        Self {
            help: Some(help),
            ..self
        }
    }
    /// Returns an [`Error`] representing multiple attempted branches have all failed.
    /// `errors` is the list of all [`Error`]s returned by those failures.
    ///
    /// Some of the provided `errors` may be discarded in an attempt to improve the resulting error message
    /// if some branches progressed further before failing than others.
    pub fn all_failed(mut errors: Vec<Self>) -> Self {
        let (earliest, farthest) = errors
            .iter()
            .map(|err| err.span.start)
            .minmax()
            .into_option()
            .unwrap();
        errors.retain(|err| err.span.start == farthest);
        let span = earliest..errors[0].span.end;
        let mut errors: Vec<ErrorKind> = errors
            .into_iter()
            .flat_map(|err| match err.kind {
                ErrorKind::AllFailed(nested) => nested,
                other => vec![other],
            })
            .collect::<Vec<_>>();
        errors.sort_unstable_by_key(ErrorKind::to_string);
        errors.dedup();
        let kind = if errors.len() == 1 {
            errors.pop().unwrap()
        } else {
            ErrorKind::AllFailed(errors)
        };
        Self::new(kind, span)
    }

    /// Returns a type implementing [`Display`] that uses the [`Source`] information for a more useful error display.
    pub fn with_source<'a, 'b>(&'a self, source: &'b Source) -> ErrorWithSource<'a, 'b> {
        ErrorWithSource {
            error: self,
            source,
        }
    }
    // TODO is there a use for print and eprint functions here like ariadne itself has?
    // I suppose we could lock stderr then to avoid issues when having multiple threads printing errors
    // TODO try some other options like codespan-reporting, there's some rough edges on ariadne
    /// Write this [`Error`] to an implementor of [`Write`] using the [`ariadne`] crate.
    ///
    /// [`Write`]: io::Write
    pub fn write_pretty<W: io::Write>(&self, source: &Source, w: W) -> io::Result<()> {
        use ariadne::{Color, Config, Fmt, Label, Report, ReportKind, Source};
        let id = source.id.as_str();
        if source.content.is_empty() {
            // the error printing library (ariadne) seems to panic in this case
            Report::<(&str, _)>::build(ReportKind::Error, id, 0)
                .with_message("Empty Input")
                .finish()
                .write((id, Source::from("")), w)
        } else {
            let mut report = Report::build(ReportKind::Error, id, self.span.start)
                .with_config(Config::default().with_index_type(ariadne::IndexType::Byte))
                .with_label(
                    Label::new((id, self.span.clone()))
                        .with_message(self.kind.to_string().fg(Color::Red))
                        .with_color(Color::Red),
                );
            if let Some(help) = &self.help {
                report.set_help(help);
            }
            report
                .finish()
                .write((id, Source::from(&source.content)), w)
        }
    }
}
impl Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.kind.fmt(f)
    }
}
impl error::Error for Error {}

/// Errors that may occur in [`Ast`] implementations, or a custom error message
///
/// [`Ast`]: crate::Ast
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorKind {
    UnexpectedToken,
    UnexpectedEnd,
    InvalidNumber(String),
    ExpectedToken(String),
    AllFailed(Vec<ErrorKind>),
    Other(String),
}
impl Display for ErrorKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorKind::UnexpectedToken => write!(f, "invalid input"), // TODO this is not a great error message so at least the documentation should warn about this
            ErrorKind::UnexpectedEnd => write!(f, "unexpected end of input"),
            ErrorKind::InvalidNumber(message) => {
                write!(f, "number does not fit in target type: {message}")
            }
            ErrorKind::ExpectedToken(message) => write!(f, "expected {message}"),
            ErrorKind::AllFailed(errors) => {
                let all_expected = errors
                    .iter()
                    .map(|err| match err {
                        ErrorKind::ExpectedToken(message) => Some(message),
                        _ => None,
                    })
                    .collect::<Option<Vec<_>>>();
                match all_expected {
                    None => write!(
                        f,
                        "multiple possibilities failed to parse: {}",
                        errors.iter().format(" / ")
                    ),
                    Some(messages) => write!(f, "expected {}", messages.iter().format(" or ")),
                }
            }
            ErrorKind::Other(message) => message.fmt(f),
        }
    }
}
impl error::Error for ErrorKind {}

/// [`Display`] implementation returned by [`Error::with_source`]
pub struct ErrorWithSource<'a, 'b> {
    error: &'a Error,
    source: &'b Source,
}
impl Display for ErrorWithSource<'_, '_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let id = &self.source.id;
        let line_start = self.source.content[..self.error.span.start]
            .rfind('\n')
            .unwrap_or(0);
        let line_number = 1 + self.source.content[..=line_start].matches('\n').count();
        let position = self.error.span.start - line_start;
        let slice = &self.source.content[self.error.span.clone()];
        let error = &self.error.kind;
        write!(f, "[{id}:{line_number}:{position}] at \"{slice}\": {error}")
    }
}
