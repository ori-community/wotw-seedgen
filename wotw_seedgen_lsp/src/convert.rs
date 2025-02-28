use std::ops::Range;

use tower_lsp::lsp_types;

pub fn range_from_lsp(range: lsp_types::Range, document: &str) -> Option<Range<usize>> {
    // TODO optimization: end > start, we are wasting time going through all the lines up to start twice
    // similar for range_to_lsp
    Some(position_from_lsp(range.start, document)?..position_from_lsp(range.end, document)?)
}

pub fn position_from_lsp(position: lsp_types::Position, document: &str) -> Option<usize> {
    let line = document.lines().nth(position.line as usize)?;
    let line_position = line.as_ptr() as usize - document.as_ptr() as usize;

    let mut utf16_offset = position.character as usize;
    for (index, char) in line.char_indices() {
        let len = char.len_utf16();
        if utf16_offset < len {
            return Some(line_position + index);
        }
        utf16_offset -= len;
    }

    Some(line_position + line.len())
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
        Some(5)
    );
    assert_eq!(
        position_to_lsp(5, multiline),
        lsp_types::Position::new(1, 1)
    );

    let wide_chars = "🦀🦀🦀";
    assert_eq!(
        position_from_lsp(lsp_types::Position::new(0, 4), wide_chars),
        Some(8)
    );
    assert_eq!(
        position_to_lsp(8, wide_chars),
        lsp_types::Position::new(0, 4)
    );
}
