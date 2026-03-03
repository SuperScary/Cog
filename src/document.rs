use std::io;
use std::path::{Path, PathBuf};

use crate::edit_history::EditAction;
use crate::file;
use crate::position::Position;

#[derive(Default)]
pub struct Document {
    lines: Vec<String>,
    file_path: Option<PathBuf>,
    is_modified: bool,
    encoding_name: String,
}

impl Document {
    pub fn empty() -> Self {
        Self {
            lines: vec![String::new()],
            file_path: None,
            is_modified: false,
            encoding_name: "UTF-8".to_string(),
        }
    }

    pub fn empty_with_path(path: impl AsRef<Path>) -> Self {
        Self {
            lines: vec![String::new()],
            file_path: Some(path.as_ref().to_path_buf()),
            is_modified: false,
            encoding_name: "UTF-8".to_string(),
        }
    }

    pub fn open_from_file(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref().to_path_buf();
        let path_str = path.to_string_lossy();

        let decoded = file::read_file_with_encoding_detection(&path_str)?;

        let mut lines: Vec<String> = decoded.text.lines().map(|line| line.to_string()).collect();

        if lines.is_empty() {
            lines.push(String::new());
        }

        Ok(Self {
            lines,
            file_path: Some(path),
            is_modified: false,
            encoding_name: decoded.encoding_name,
        })
    }

    pub fn save_to_file(&mut self, path: Option<impl AsRef<Path>>) -> io::Result<()> {
        if let Some(path) = path {
            self.file_path = Some(path.as_ref().to_path_buf());
        }

        let Some(path) = self.file_path.clone() else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "No file path set",
            ));
        };

        let contents = self.lines.join("\n");
        let path_str = path.to_string_lossy();
        file::save_with_encoding(&path_str, &contents, &self.encoding_name)?;
        self.is_modified = false;
        Ok(())
    }

    // ── Queries ──────────────────────────────────────────────────────

    pub fn file_name_display(&self) -> String {
        match &self.file_path {
            Some(path) => path
                .file_name()
                .and_then(|s| s.to_str())
                .unwrap_or("[Invalid Name]")
                .to_string(),
            None => "[No Name]".to_string(),
        }
    }

    pub fn is_modified(&self) -> bool {
        self.is_modified
    }

    pub fn encoding_name(&self) -> &str {
        &self.encoding_name
    }

    pub fn file_extension(&self) -> Option<String> {
        self.file_path
            .as_ref()
            .and_then(|p| p.extension())
            .and_then(|ext| ext.to_str())
            .map(|s| s.to_string())
    }

    pub fn number_of_lines(&self) -> usize {
        self.lines.len()
    }

    pub fn line(&self, row_index: usize) -> &str {
        self.lines
            .get(row_index)
            .map(|line| line.as_str())
            .unwrap_or("")
    }

    /// Copies text between two positions without modifying the document.
    pub fn extract_text(&self, start: Position, end: Position) -> String {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        if start.row_index >= self.lines.len() {
            return String::new();
        }

        let end_row = end.row_index.min(self.lines.len() - 1);
        let start_byte = start.column_index.min(self.line(start.row_index).len());
        let end_byte = end.column_index.min(self.line(end_row).len());

        if start.row_index == end_row {
            return self.lines[start.row_index][start_byte..end_byte].to_string();
        }

        let mut text = String::new();
        text.push_str(&self.lines[start.row_index][start_byte..]);

        for row in (start.row_index + 1)..end_row {
            text.push('\n');
            text.push_str(&self.lines[row]);
        }

        text.push('\n');
        text.push_str(&self.lines[end_row][..end_byte]);

        text
    }

    // ── Mutations ────────────────────────────────────────────────────

    /// Inserts arbitrary text (single-char, multi-line, etc.) at a position.
    /// Returns the position immediately after the inserted text.
    pub fn insert_text(&mut self, position: Position, text: &str) -> Position {
        if text.is_empty() {
            return position;
        }

        self.ensure_row_exists(position.row_index);

        let insert_byte = position
            .column_index
            .min(self.lines[position.row_index].len());
        let after_cursor = self.lines[position.row_index].split_off(insert_byte);

        let segments: Vec<&str> = text.split('\n').collect();

        self.lines[position.row_index].push_str(segments[0]);

        for (index, segment) in segments.iter().enumerate().skip(1) {
            self.lines
                .insert(position.row_index + index, segment.to_string());
        }

        let last_row = position.row_index + segments.len() - 1;
        let end_column = self.lines[last_row].len();
        self.lines[last_row].push_str(&after_cursor);

        self.is_modified = true;
        Position::new(last_row, end_column)
    }

    /// Deletes all text between two positions. Returns the deleted text.
    pub fn delete_range(&mut self, start: Position, end: Position) -> String {
        let (start, end) = if start <= end {
            (start, end)
        } else {
            (end, start)
        };

        if start.row_index >= self.lines.len() {
            return String::new();
        }

        let end_row = end.row_index.min(self.lines.len() - 1);
        let start_byte = start.column_index.min(self.lines[start.row_index].len());
        let end_byte = end.column_index.min(self.lines[end_row].len());

        if start.row_index == end_row {
            let deleted: String = self.lines[start.row_index]
                .drain(start_byte..end_byte)
                .collect();
            if !deleted.is_empty() {
                self.is_modified = true;
            }
            return deleted;
        }

        let mut deleted = String::new();
        deleted.push_str(&self.lines[start.row_index][start_byte..]);

        for row in (start.row_index + 1)..end_row {
            deleted.push('\n');
            deleted.push_str(&self.lines[row]);
        }

        deleted.push('\n');
        deleted.push_str(&self.lines[end_row][..end_byte]);

        let remaining_after_end = self.lines[end_row][end_byte..].to_string();
        self.lines[start.row_index].truncate(start_byte);
        self.lines[start.row_index].push_str(&remaining_after_end);

        self.lines.drain((start.row_index + 1)..=end_row);

        self.is_modified = true;
        deleted
    }

    /// Applies an undo/redo action to the document. Returns the new caret position.
    pub fn apply_action(&mut self, action: &EditAction) -> Position {
        match action {
            EditAction::Insert { position, text, .. } => self.insert_text(*position, text),
            EditAction::Delete { start, end, .. } => {
                self.delete_range(*start, *end);
                *start
            }
        }
    }

    fn ensure_row_exists(&mut self, row_index: usize) {
        if row_index >= self.lines.len() {
            self.lines.resize(row_index + 1, String::new());
        }
    }
}
