const DEFAULT_TAB_SIZE: usize = 4;

/// Returns the configured tab size, or the default (4) if the config is unavailable.
pub fn tab_size() -> usize {
    let config = crate::config::configs::load_editor_settings();
    config
        .get_from(Some("editor"), "tab_size")
        .and_then(|v| v.parse::<usize>().ok())
        .unwrap_or(DEFAULT_TAB_SIZE)
}

/// Computes the display column for a given byte offset in a line, accounting
/// for tab characters that expand to the next tab stop.
pub fn display_column(line: &str, byte_offset: usize, tab_size: usize) -> usize {
    let mut column = 0;
    for (index, character) in line.char_indices() {
        if index >= byte_offset {
            break;
        }
        if character == '\t' {
            column = column + tab_size - (column % tab_size);
        } else {
            column += 1;
        }
    }
    column
}

/// Expands tab characters in a line to spaces aligned to tab stops.
/// Returns the expanded string and a mapping from each byte in the original
/// line to its starting display column.
pub fn expand_tabs(line: &str, tab_size: usize) -> (String, Vec<usize>) {
    let mut expanded = String::with_capacity(line.len());
    let mut byte_to_display = Vec::with_capacity(line.len());
    let mut column = 0;

    for character in line.chars() {
        let char_byte_len = character.len_utf8();
        let start_column = column;

        if character == '\t' {
            let spaces = tab_size - (column % tab_size);
            for _ in 0..spaces {
                expanded.push(' ');
            }
            column += spaces;
        } else {
            expanded.push(character);
            column += 1;
        }

        for _ in 0..char_byte_len {
            byte_to_display.push(start_column);
        }
    }

    (expanded, byte_to_display)
}

/// Given a byte offset into the original line, returns the corresponding
/// byte offset in the expanded (tab-free) line.
pub fn original_byte_to_expanded_byte(
    byte_to_display: &[usize],
    original_byte: usize,
    expanded_len: usize,
) -> usize {
    if original_byte >= byte_to_display.len() {
        return expanded_len;
    }
    byte_to_display[original_byte]
}
