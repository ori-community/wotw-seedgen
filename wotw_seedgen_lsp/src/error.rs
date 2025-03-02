use std::borrow::Cow;

use tower_lsp::{
    jsonrpc::{Error, ErrorCode},
    lsp_types::{Position, Url},
};

#[repr(i64)]
enum ErrorKind {
    UnknownTextDocument = 1,
    PositionOutOfBounds = 2,
}

pub fn unknown_text_document(url: &Url) -> Error {
    server_error(
        ErrorKind::UnknownTextDocument,
        Cow::Owned(format!("unknown text document \"{url}\"")),
    )
}

pub fn position_out_of_bounds(position: Position, document: &str) -> Error {
    let lines = document.split_inclusive('\n').count();

    server_error(
        ErrorKind::PositionOutOfBounds,
        Cow::Owned(format!(
            "position \"{line}:{character}\" out of bounds, document only has {lines} lines",
            line = position.line,
            character = position.character
        )),
    )
}

fn server_error(kind: ErrorKind, message: Cow<'static, str>) -> Error {
    Error {
        code: ErrorCode::ServerError(kind as i64),
        message,
        data: None,
    }
}
