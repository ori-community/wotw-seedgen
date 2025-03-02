use std::ops::Range;

use tower_lsp::{jsonrpc::Result, lsp_types};

use crate::error;

pub fn range_from_lsp(range: lsp_types::Range, document: &str) -> Result<Range<usize>> {
    // TODO optimization: end > start, we are wasting time going through all the lines up to start twice
    // similar for range_to_lsp
    Ok(position_from_lsp(range.start, document)?..position_from_lsp(range.end, document)?)
}

pub fn position_from_lsp(position: lsp_types::Position, document: &str) -> Result<usize> {
    // If the document ends with a newline and the position is at the end of the document,
    // position.line will refer to the empty "line" after, which would be out of bounds
    // when using lines() or split_inclusive('\n').
    let line_position = if position.line == 0 {
        0
    } else {
        let previous_line_end = document
            .match_indices('\n')
            .nth((position.line - 1) as usize)
            .ok_or_else(|| error::position_out_of_bounds(position, document))?
            .0;
        previous_line_end + 1
    };
    // We'll return once we reached position.character, so as long as we receive a
    // well formed position, we won't go past the target line.
    let line = &document[line_position..];

    let mut utf16_offset = position.character as usize;
    for (index, char) in line.char_indices() {
        let len = char.len_utf16();
        if utf16_offset < len {
            return Ok(line_position + index);
        }
        utf16_offset -= len;
    }

    Ok(line_position + line.len())
}

pub fn range_to_lsp(range: Range<usize>, document: &str) -> lsp_types::Range {
    lsp_types::Range::new(
        position_to_lsp(range.start, document),
        position_to_lsp(range.end, document),
    )
}

pub fn position_to_lsp(position: usize, document: &str) -> lsp_types::Position {
    let (line, line_start) = last_line(&document[..position]);
    let character = document[line_start..position].encode_utf16().count();
    lsp_types::Position::new(line as u32, character as u32)
}

pub fn last_line(source: &str) -> (usize, usize) {
    let mut line = 0;
    let mut line_start = 0;
    let mut line_indices = source.rmatch_indices('\n');
    if let Some((index, _)) = line_indices.next() {
        line = 1;
        line_start = index + 1;
    };
    line += line_indices.count();

    (line, line_start)
}

#[cfg(test)]
#[test]
fn convert() {
    let multiline = "aaa\nbbb\nccc";
    assert_eq!(
        position_from_lsp(lsp_types::Position::new(1, 1), multiline),
        Ok(5)
    );
    assert_eq!(
        position_to_lsp(5, multiline),
        lsp_types::Position::new(1, 1)
    );

    let wide_chars = "🦀🦀🦀";
    assert_eq!(
        position_from_lsp(lsp_types::Position::new(0, 4), wide_chars),
        Ok(8)
    );
    assert_eq!(
        position_to_lsp(8, wide_chars),
        lsp_types::Position::new(0, 4)
    );
}
