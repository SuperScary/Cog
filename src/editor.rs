use std::io::{self, Write};
use std::path::Path;
use std::time::Duration;

use crate::clipboard::Clipboard;
use crate::document::Document;
use crate::edit_history::{EditAction, EditHistory};
use crate::file_detector::detect_language_from_path;
use crate::gutter::Gutter;
use crate::position::Position;
use crate::selection::Selection;
use crate::syntax_definition::SyntaxDefinition;
use crate::syntax_highlighter::{self, HighlightSpan};
use crate::tab_handler;
use crate::status_bar::BottomStatusBar;
use crossterm::event::KeyEventKind;
use crossterm::style::{Attribute, Color, SetAttribute, SetForegroundColor};
use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEvent, KeyModifiers},
    queue,
    style::Print,
    terminal::{self, ClearType},
};

fn modifier_key_name() -> &'static str {
    #[cfg(target_os = "macos")]
    { "Cmd" }
    #[cfg(target_os = "windows")]
    { "Ctrl" }
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    { "Ctrl" }
}

/// Represents a text editor structure with various fields to manage its state.
///
/// # Fields
///
/// * `document` - The `Document` being edited. It holds the content and structure of the text.
/// * `caret_position` - The current position of the caret (text cursor) within the document.
/// * `selection` - An optional `Selection` representing a range of text selected by the user.
///                 This is `None` if no text is selected.
/// * `clipboard` - The `Clipboard` used for storing copied or cut text data.
/// * `edit_history` - The `EditHistory` object tracking changes made to the document, allowing undo/redo actions.
/// * `vertical_scroll_offset` - The vertical scroll offset in lines, representing how much of the document
///                              is scrolled in the vertical direction.
/// * `horizontal_scroll_offset` - The horizontal scroll offset in columns, representing how much of the document
///                                is scrolled in the horizontal direction.
/// * `help_message` - A string containing a help message or instructions for the user.
pub struct Editor {
    document: Document,
    caret_position: Position,
    selection: Option<Selection>,
    clipboard: Clipboard,
    edit_history: EditHistory,
    syntax_definition: Option<SyntaxDefinition>,
    tab_size: usize,
    vertical_scroll_offset: usize,
    horizontal_scroll_offset: usize,
    help_message: String,
    bottom_status_bar: BottomStatusBar,
}

impl Editor {
    /// Creates a new instance of the struct with default values.
    ///
    /// # Returns
    /// A new instance of the struct with the following default configuration:
    /// - `document`: An empty `Document`.
    /// - `caret_position`: The default `Position` (typically representing the start of the document).
    /// - `selection`: None, indicating there is no active text selection.
    /// - `clipboard`: A newly initialized `Clipboard` object.
    /// - `edit_history`: An empty `EditHistory` for undo/redo operations.
    /// - `vertical_scroll_offset`: 0, meaning there is no vertical scrolling applied.
    /// - `horizontal_scroll_offset`: 0, meaning there is no horizontal scrolling applied.
    /// - `help_message`: A default help message string, as defined by `DEFAULT_HELP_MESSAGE`.
    pub fn new() -> Self {
        Self {
            document: Document::empty(),
            caret_position: Position::default(),
            selection: None,
            clipboard: Clipboard::new(),
            edit_history: EditHistory::new(),
            syntax_definition: None,
            tab_size: tab_handler::tab_size(),
            vertical_scroll_offset: 0,
            horizontal_scroll_offset: 0,
            help_message: format!("{}+S Save | {}+F Find | {}+Q Quit", modifier_key_name(), modifier_key_name(), modifier_key_name()),
            bottom_status_bar: BottomStatusBar::new(1, 0, "")
        }
    }

    /// Opens a document from the specified file path or creates a new document if the file does not exist.
    ///
    /// # Arguments
    /// * `path` - A type that can be converted into a `Path`. This specifies the file path to open or create.
    ///
    /// # Returns
    /// Returns an `io::Result<Self>` which is:
    /// - `Ok(Self)`: If the operation is successful, the method returns a new instance of the containing type, initialized with:
    ///     - The document loaded from the specified file if it exists.
    ///     - An empty document with the provided path if the file does not exist.
    /// - `Err(io::Error)`: If there is an error while attempting to open the file.
    ///
    /// # Fields Initialized
    /// - `document`: Initialized with either the content of the file or an empty document tied to the specified path.
    /// - `caret_position`: Defaults to the starting position (`Position::default()`).
    /// - `selection`: Initializes as `None` (no active text selection).
    /// - `clipboard`: A new, empty clipboard instance (`Clipboard::new()`).
    /// - `edit_history`: A new, empty edit history instance (`EditHistory::new()`).
    /// - `vertical_scroll_offset`: Defaults to `0` (no vertical scrolling).
    /// - `horizontal_scroll_offset`: Defaults to `0` (no horizontal scrolling).
    /// - `help_message`: Sets to the default help message (`DEFAULT_HELP_MESSAGE.to_string()`).
    /// - `bottom_status_bar`: A new, empty bottom status bar instance (`BottomStatusBar::new(1, 0, "")`).
    ///
    /// # Errors
    /// This function will return an error as an `io::Error` in case of:
    /// - I/O errors that occur while opening or reading the file.
    ///
    /// # Example
    /// ```
    /// use crate::YourStruct;
    /// use std::path::Path;
    ///
    /// // Example: Open an existing document or create a new one
    /// let editor = YourStruct::open(Path::new("example.txt"));
    ///
    /// match editor {
    ///     Ok(instance) => {
    ///         // Work with the editor instance
    ///     }
    ///     Err(error) => {
    ///         // Handle the error
    ///         eprintln!("Error: {}", error);
    ///     }
    /// }
    /// ```
    pub fn open(path: impl AsRef<Path>) -> io::Result<Self> {
        let path = path.as_ref();

        let document = if path.exists() {
            Document::open_from_file(path)?
        } else {
            Document::empty_with_path(path)
        };

        let syntax_definition = document
            .file_extension()
            .and_then(|ext| SyntaxDefinition::find_for_extension(&ext));

        Ok(Self {
            document,
            caret_position: Position::default(),
            selection: None,
            clipboard: Clipboard::new(),
            edit_history: EditHistory::new(),
            syntax_definition,
            tab_size: tab_handler::tab_size(),
            vertical_scroll_offset: 0,
            horizontal_scroll_offset: 0,
            help_message: format!("{}+S Save | {}+F Find | {}+Q Quit", modifier_key_name(), modifier_key_name(), modifier_key_name()),
            bottom_status_bar: BottomStatusBar::new(1, 0, "")
        })
    }

    /// Executes the main application loop, handling rendering, input, and window events.
    ///
    /// # Parameters
    /// - `stdout`: A mutable reference to the standard output, used for rendering the application's UI.
    ///
    /// # Returns
    /// - `Result<()>`: Returns `Ok(())` if the loop exits without errors, or an `io::Error` if an error occurs.
    ///
    /// # Behavior
    /// - Continuously runs the application's main loop until a quit event is triggered.
    /// - Redraws the UI when necessary, determined by `needs_redraw`.
    /// - Listens for user input and handles key events and window resize events:
    ///   - **Key Events:**
    ///     - If a key press is detected, it calls `self.handle_key_event`. If this method returns `true`,
    ///       the loop exits (indicating the application should quit).
    ///   - **Resize Events:**
    ///     - If a resize event is detected, it calls `self.ensure_caret_is_visible` to adjust the UI as necessary.
    /// - Polls for events with a timeout of 250 milliseconds to control the application's responsiveness.
    ///
    /// # Errors
    /// - Returns an `io::Error` if there is a failure during I/O operations such as rendering, reading user input,
    ///   or handling window events.
    ///
    /// # Example Usage
    /// ```rust
    /// // Assuming `app` is an instance of a struct implementing the `run` function:
    /// let mut stdout = std::io::stdout();
    /// app.run(&mut stdout)?;
    /// ```
    pub fn run(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        let mut needs_redraw = true;

        loop {
            if needs_redraw {
                self.render(stdout)?;
                needs_redraw = false;
            }

            if event::poll(Duration::from_millis(250))? {
                needs_redraw = true;

                match event::read()? {
                    Event::Key(key_event) => {
                        if key_event.kind == KeyEventKind::Press {
                            let should_quit = self.handle_key_event(key_event, stdout)?;
                            if should_quit {
                                break;
                            }
                        }
                    }
                    Event::Resize(_, _) => {
                        self.ensure_caret_is_visible()?;
                    }
                    _ => {}
                }
            }
        }

        Ok(())
    }

    /// Handles a keyboard event and performs the corresponding actions based on the given input.
    ///
    /// # Parameters
    /// - `key_event`: The `KeyEvent` instance representing the key input along with any modifiers.
    /// - `stdout`: A mutable reference to the `Stdout` used for rendering the output.
    ///
    /// # Returns
    /// - `Ok(true)` if the operation results in the program exiting (e.g., user quits).
    /// - `Ok(false)` if the program should continue running.
    /// - `Err(io::Error)` if an I/O-related error occurs during handling.
    ///
    /// # Behavior
    /// - Detects key combinations involving control (`Ctrl`) and shift (`Shift`) modifiers.
    /// - Handles application quit requests when `Ctrl+Q` is pressed:
    ///     - If no unsaved changes exist, the program exits.
    ///     - If unsaved changes are present, the user is prompted with a message:
    ///         - Pressing `y` saves changes and exits.
    ///         - Pressing `n` exits without saving.
    ///         - Pressing `Esc` cancels quitting.
    /// - Processes navigation keys (`Arrow`, `Home`, `End`) via `handle_navigation`.
    /// - Handles text-editing keys:
    ///     - `Backspace`: Deletes the character before the caret.
    ///     - `Delete`: Deletes the character at the caret position.
    ///     - `Enter`: Inserts a new line.
    /// - Inserts characters into the document for non-modified inputs (e.g., regular typing).
    /// - Special handling for control-character shortcuts facilitated by `handle_control_key`.
    /// - Ensures the caret's position is visible by updating document rendering.
    ///
    /// # Key Bindings
    /// - **Quit**: `Ctrl+Q`
    ///     - Prompts for saving unsaved changes if any.
    ///     - Accepts `y` to save and exit, `n` to exit without saving, and `Esc` to cancel.
    /// - **Navigation**: Arrow keys, `Home`, `End`.
    /// - **Text Editing**: `Backspace`, `Delete`, `Enter`, or any printable character.
    /// - **Special Control Commands**: Handled via `handle_control_key`.
    ///
    /// # Errors
    /// - Returns an `io::Error` if reading input events or rendering the output fails.
    fn handle_key_event(&mut self, key_event: KeyEvent, stdout: &mut io::Stdout, ) -> io::Result<bool> {
        let is_control = key_event.modifiers.contains(KeyModifiers::CONTROL);
        let is_shift = key_event.modifiers.contains(KeyModifiers::SHIFT);
        let (_terminal_width, terminal_height) = terminal::size()?;
        let page_rows = (terminal_height as usize).saturating_sub(self.status_bar_height() + 2); // Status bar and prompt bar
        let max_row = self.document.number_of_lines().saturating_sub(1);

        if is_control && key_event.code == KeyCode::Char('q') {
            if !self.document.is_modified() {
                return Ok(true);
            }

            self.help_message = "Unsaved changes! Save before quitting? (y/n, Esc to cancel)".to_string();
            self.render(stdout)?;

            loop {
                if let Event::Key(confirm_event) = event::read()? {
                    if confirm_event.kind != KeyEventKind::Press { continue; }

                    match confirm_event.code {
                        KeyCode::Char('y') => match self.document.save_to_file(None::<&str>) {
                            Ok(()) => return Ok(true),
                            Err(error) => {
                                self.help_message = format!("Save failed: {error}");
                                break;
                            }
                        },
                        KeyCode::Char('n') => return Ok(true),
                        KeyCode::Esc => {
                            self.help_message = format!("{}+S Save | {}+F Find | {}+Q Quit", modifier_key_name(), modifier_key_name(), modifier_key_name());
                            break;
                        }
                        _ => {}
                    }
                }
            }
        } else if is_control && key_event.code == KeyCode::PageUp {
            self.caret_position.row_index = 0;
            self.caret_position.column_index = 0;
            self.ensure_caret_is_visible()?;
            return Ok(false);
        } else if is_control && key_event.code == KeyCode::PageDown {
            self.caret_position.row_index = self.document.number_of_lines() - 1;
            self.caret_position.column_index = self.document.line(self.caret_position.row_index).len();
            self.ensure_caret_is_visible()?;
            return Ok(false);
        } else if !is_control && key_event.code == KeyCode::PageUp {
            self.caret_position.row_index = self.caret_position.row_index.saturating_sub(page_rows).min(max_row);
            self.caret_position.column_index = self.caret_position.column_index.min(self.document.line(self.caret_position.row_index).len());
            self.ensure_caret_is_visible()?;
            return Ok(false);
        } else if !is_control && key_event.code == KeyCode::PageDown {
            self.caret_position.row_index = self.caret_position.row_index.saturating_add(page_rows).min(max_row);
            self.caret_position.column_index = self.caret_position.column_index.min(self.document.line(self.caret_position.row_index).len());
            self.ensure_caret_is_visible()?;
            return Ok(false);
        }

        match key_event.code {
            KeyCode::Left
            | KeyCode::Right
            | KeyCode::Up
            | KeyCode::Down
            | KeyCode::Home
            | KeyCode::End => {
                self.handle_navigation(key_event.code, is_shift);
            }

            KeyCode::Backspace => self.perform_backspace(),
            KeyCode::Delete => self.perform_delete(),
            KeyCode::Enter => self.perform_insert("\n", false),

            KeyCode::Char(character) => {
                if is_control {
                    self.handle_control_key(character);
                } else if !key_event.modifiers.contains(KeyModifiers::ALT) {
                    self.perform_insert(&character.to_string(), true);
                }
            }

            _ => {}
        }

        self.ensure_caret_is_visible()?;
        Ok(false)
    }

    /// Handles navigation and text selection based on the provided key input and modifier state.
    ///
    /// # Parameters
    /// - `key_code`: The `KeyCode` indicating the navigation operation (e.g., left, right, up, down).
    /// - `is_shift_held`: A boolean indicating if the Shift key is held. When true, text selection is initiated or modified.
    ///
    /// # Behavior
    /// 1. **With `Shift` Key Held**:
    ///    - If there is no existing selection, a new selection is created from the current caret position.
    ///    - Updates the end of the selection to match the new caret position after navigation.
    ///    - If the selection becomes empty after navigation, it is cleared.
    ///
    /// 2. **Without `Shift` Key Held**:
    ///    - If a selection exists, it is cleared, and the caret is repositioned based on the navigation key:
    ///      - Moves the caret to the start of the selection for `KeyCode::Left`, `KeyCode::Up`, or `KeyCode::Home`.
    ///      - Moves the caret to the end of the selection for other navigation keys.
    ///    - Performs the requested navigation.
    ///    - If no selection exists, only the caret position is updated without affecting selection.
    ///
    /// # Navigation Keys
    /// - `KeyCode::Left`: Moves the caret one position to the left.
    /// - `KeyCode::Right`: Moves the caret one position to the right.
    /// - `KeyCode::Up`: Moves the caret one line up.
    /// - `KeyCode::Down`: Moves the caret one line down.
    /// - `KeyCode::Home`: Moves the caret to the beginning of the current line.
    /// - `KeyCode::End`: Moves the caret to the end of the current line.
    ///
    /// # Selection Behavior
    /// - Modifies an existing selection's `cursor` position unless the selection becomes empty.
    /// - Clears the selection if it becomes empty (start and end positions converge).
    fn handle_navigation(&mut self, key_code: KeyCode, is_shift_held: bool) {
        if is_shift_held {
            if self.selection.is_none() {
                self.selection = Some(Selection::new(self.caret_position, self.caret_position));
            }
        } else if let Some(selection) = self.selection.take() {
            let (start, end) = selection.ordered_range();
            match key_code {
                KeyCode::Left | KeyCode::Up | KeyCode::Home => {
                    self.caret_position = start;
                }
                _ => {
                    self.caret_position = end;
                }
            }
            return;
        }

        match key_code {
            KeyCode::Left => self.move_caret_left(),
            KeyCode::Right => self.move_caret_right(),
            KeyCode::Up => self.move_caret_up(),
            KeyCode::Down => self.move_caret_down(),
            KeyCode::Home => self.caret_position.column_index = 0,
            KeyCode::End => {
                self.caret_position.column_index =
                    self.document.line(self.caret_position.row_index).len();
            }
            _ => {}
        }

        if let Some(selection) = &mut self.selection {
            selection.cursor = self.caret_position;
            if selection.is_empty() {
                self.selection = None;
            }
        }
    }

    /// Handles control key inputs and maps specific characters to corresponding editor actions.
    ///
    /// # Arguments
    /// * `character` - A `char` representing the control key pressed by the user.
    ///
    /// The method executes different actions based on the value of `character`:
    ///
    /// - `'s'`: Attempts to save the current document to a file.
    ///   - If the save operation succeeds, the help message is updated to indicate success ("File saved successfully.").
    ///   - If the save operation fails, the help message displays the encountered error ("Save failed: {error}").
    ///
    /// - `'c'`: Executes the copy operation by calling `perform_copy()`.
    ///
    /// - `'x'`: Executes the cut operation by calling `perform_cut()`.
    ///
    /// - `'v'`: Executes the paste operation by calling `perform_paste()`.
    ///
    /// - `'z'`: Executes the undo operation by calling `perform_undo()`.
    ///
    /// - `'y'`: Executes the redo operation by calling `perform_redo()`.
    ///
    /// - `'o'`: Updates the help message to indicate that the "Open" functionality is not implemented yet ("Open: not implemented yet").
    ///
    /// - `'f'`: Updates the help message to indicate that the "Find" functionality is not implemented yet ("Find: not implemented yet").
    ///
    /// - For any other key, no action is performed.
    ///
    /// # Notes
    /// This function modifies the editor's internal state, including `help_message`, to provide feedback
    /// to the user about the executed command or unimplemented features.
    fn handle_control_key(&mut self, character: char) {
        match character {
            's' => match self.document.save_to_file(None::<&str>) {
                Ok(()) => self.help_message = "File saved successfully.".to_string(),
                Err(error) => self.help_message = format!("Save failed: {error}"),
            },
            'c' => self.perform_copy(),
            'x' => self.perform_cut(),
            'v' => self.perform_paste(),
            'z' => self.perform_undo(),
            'y' => self.perform_redo(),
            'o' => {
                self.help_message = "Open: not implemented yet".to_string();
            }
            'f' => {
                self.help_message = "Find: not implemented yet".to_string();
            }
            _ => {}
        }
    }

    /// Moves the caret one position to the left within the text editor.
    ///
    /// If the caret is not at the beginning of a line, this function decreases the
    /// `column_index` by one, moving the caret one step to the left on the same line.
    ///
    /// If the caret is at the beginning of a line (`column_index` equals 0) and
    /// there are lines above (`row_index` > 0), the caret moves to the end of the
    /// previous line by updating `row_index` to the previous line and setting
    /// `column_index` to the length of that line.
    ///
    /// # Behavior
    /// - Does nothing if the caret is already at the beginning of the document
    ///   (i.e., `row_index` and `column_index` are both 0).
    ///
    /// # Assumptions
    /// - `self.document.line(row_index)` returns the content of a line in the document
    ///   as a string or a type that supports `.len()`.
    /// - `self.caret_position` keeps track of the current caret position with
    ///   both `row_index` for the line number and `column_index` for the character
    ///   position within the line.
    ///
    /// # Example
    /// ```rust
    /// let mut editor = TextEditor::new();
    /// editor.caret_position = CaretPosition { row_index: 1, column_index: 0 };
    /// editor.move_caret_left();
    /// assert_eq!(editor.caret_position, CaretPosition {
    ///     row_index: 0,
    ///     column_index: editor.document.line(0).len()
    /// });
    /// ```
    fn move_caret_left(&mut self) {
        if self.caret_position.column_index > 0 {
            self.caret_position.column_index -= 1;
            return;
        }

        if self.caret_position.row_index > 0 {
            self.caret_position.row_index -= 1;
            self.caret_position.column_index =
                self.document.line(self.caret_position.row_index).len();
        }
    }

    /// Moves the caret (cursor) one position to the right within the document.
    ///
    /// If the caret's current position is not at the end of the current line,
    /// it is moved one column to the right. If the caret is at the end of the
    /// line and there is a next line available, the caret is moved to the
    /// beginning of the next line.
    ///
    /// # Behavior
    /// - If the caret is within the bounds of the current line, the `column_index`
    ///   is incremented by 1.
    /// - If the caret is at the end of the current line and another line exists,
    ///   the `row_index` is incremented by 1, and the `column_index` is reset to 0.
    /// - If the caret is at the end of the document (last line and last column),
    ///   no action is taken.
    ///
    /// # Preconditions
    /// - The `self.document` must have valid lines.
    /// - `self.caret_position` must refer to a valid position within the document.
    ///
    /// # Fields
    /// - `self.caret_position.row_index`: The current row index of the caret.
    /// - `self.caret_position.column_index`: The current column index of the caret.
    /// - `self.document.line(row_index)`: Returns the content of the line at a given row index.
    /// - `self.document.number_of_lines()`: Returns the total number of lines in the document.
    fn move_caret_right(&mut self) {
        let current_line_length = self.document.line(self.caret_position.row_index).len();

        if self.caret_position.column_index < current_line_length {
            self.caret_position.column_index += 1;
            return;
        }

        if self.caret_position.row_index + 1 < self.document.number_of_lines() {
            self.caret_position.row_index += 1;
            self.caret_position.column_index = 0;
        }
    }

    /// Moves the caret (cursor) up by one row in the text document.
    ///
    /// If the caret is already at the top row (row index 0), this function does nothing.
    /// Otherwise, it decreases the caret's row index by 1 and adjusts the column index
    /// to ensure it remains valid for the new row. If the current column index exceeds
    /// the length of the new row, it is clamped to the end of the row.
    ///
    /// # Assumptions
    /// - `self.caret_position` represents the current position of the caret in the document,
    ///   containing `row_index` (the row number) and `column_index` (the column number).
    /// - `self.document` provides access to the text content of the document, and the method
    ///   `line(row_index)` returns the text content of the specified row.
    ///
    /// # Example
    /// ```rust
    /// // Assume `caret_position` starts at row 2, column 5.
    /// editor.move_caret_up();
    /// // After calling, `caret_position` is moved to row 1 at column 5, or adjusted
    /// // to match the length of row 1 if column 5 exceeds the new row's length.
    /// ```
    fn move_caret_up(&mut self) {
        if self.caret_position.row_index == 0 {
            return;
        }

        self.caret_position.row_index -= 1;
        self.caret_position.column_index = self
            .caret_position
            .column_index
            .min(self.document.line(self.caret_position.row_index).len());
    }

    /// Moves the caret one row down in the document, if possible.
    ///
    /// This function updates the caret's `row_index` to the next row, ensuring it does not exceed
    /// the total number of lines in the document. The column position is adjusted to stay within
    /// the bounds of the new row's length.
    ///
    /// # Behavior
    /// - If the caret is already on the last row of the document, the function exits without
    ///   making any changes.
    /// - The column position of the caret is constrained to the length of the new row to prevent
    ///   it from exceeding the row's bounds.
    ///
    /// # Assumptions
    /// - It is assumed that `self.document` provides the methods:
    ///   - `number_of_lines()`: Returns the total number of lines in the document.
    ///   - `line(row_index)`: Returns the content of the line at the specified row index as
    ///     a `String` or `&str`, allowing the line's length to be determined.
    ///
    /// # Example
    /// ```rust
    /// // Assuming `self` is properly initialized with a `document` and valid `caret_position`:
    /// self.move_caret_down();
    /// ```
    fn move_caret_down(&mut self) {
        if self.caret_position.row_index + 1 >= self.document.number_of_lines() {
            return;
        }

        self.caret_position.row_index += 1;
        self.caret_position.column_index = self
            .caret_position
            .column_index
            .min(self.document.line(self.caret_position.row_index).len());
    }

    // ── Edit operations ──────────────────────────────────────────────

    /// Inserts the given text at the current caret position in the document.
    ///
    /// If there is a current text selection, the selected text is deleted before performing the insertion.
    /// The insertion of text also updates the document state and records the action in the edit history.
    ///
    /// # Arguments
    ///
    /// * `text` - A string slice containing the text to be inserted.
    /// * `merge_with_previous` - A boolean flag indicating whether this insertion should be merged
    ///   with the previous edit action in the edit history. If `true`, the current insertion
    ///   is combined with the last action for undo/redo purposes; otherwise, it is recorded as a separate action.
    ///
    /// # Behavior
    ///
    /// 1. Deletes any currently selected text (if a selection exists).
    /// 2. Inserts the provided `text` at the current `caret_position` in the document.
    /// 3. Records the insertion operation in the edit history, specifying:
    ///    - The starting insertion position.
    ///    - The content of the inserted text.
    ///    - The ending position after insertion.
    /// 4. Updates the `caret_position` to the end position after the inserted text.
    ///
    /// # Panics
    ///
    /// This function assumes that the caret position is valid and that the document handles
    /// text insertion correctly. Any issues with insertion or caret invalidity must be handled
    /// by the caller or the `document` object itself.
    fn perform_insert(&mut self, text: &str, merge_with_previous: bool) {
        if let Some(selection) = self.selection.take() {
            self.delete_selected_text(selection);
        }

        let end_position = self.document.insert_text(self.caret_position, text);
        self.edit_history.record(
            EditAction::Insert {
                position: self.caret_position,
                text: text.to_string(),
                end_position,
            },
            merge_with_previous,
        );
        self.caret_position = end_position;
    }

    /// Performs a backspace operation in the text editor.
    ///
    /// This method handles removing text based on the current caret position and selection,
    /// if any. It updates the document and edit history to reflect the changes and adjusts
    /// the caret position appropriately.
    ///
    /// Behavior:
    /// 1. If there is a selected text region (`self.selection`), it deletes the selected text
    ///    and exits the method.
    /// 2. If there is no selected text, it determines the range of text to delete:
    ///    - If the caret is not at the start of a line, it deletes the character immediately
    ///      before the caret.
    ///    - If the caret is at the start of a line, it merges the current line with the previous
    ///      one.
    /// 3. Updates the document by deleting the determined range of text.
    /// 4. Records the deletion action in the edit history.
    /// 5. Moves the caret to the new position after the deletion.
    ///
    /// # Notes
    /// - If the caret is at the very beginning of the document (position (0, 0)), no action is taken.
    /// - The edit history recording ensures that this operation can be undone/redone appropriately.
    ///
    /// # Fields Involved
    /// - `self.selection`: Represents the currently selected text range, if any.
    /// - `self.caret_position`: Tracks the current position of the caret in the document.
    /// - `self.document`: Represents the document being edited, providing access to lines
    ///   and allowing for modification.
    /// - `self.edit_history`: Keeps track of edit actions to enable undo/redo functionality.
    ///
    /// # Example Behavior
    /// - Caret inside a line: Deletes the character immediately to the left of the caret.
    /// - Caret at the start of a line: Merges the current line with the previous line.
    /// - Selected text: Deletes the selected region and exits.
    ///
    /// # Implementation Details
    /// - If there is no selection (`self.selection.take()` == `None`), the method determines the
    ///   start of the range to delete (`delete_start`) by checking the caret's position:
    ///   - If the caret is not at column `0`, it moves one column back.
    ///   - If the caret is at column `0`, it moves to the end of the previous line (if there is one).
    ///
    /// # See Also
    /// - [`Position::new`](#) for constructing positions.
    /// - [`self.document.delete_range`](#) for deleting a range of text in the document.
    /// - [`self.edit_history.record`](#) for recording edit actions.
    fn perform_backspace(&mut self) {
        if let Some(selection) = self.selection.take() {
            self.delete_selected_text(selection);
            return;
        }

        let delete_start = if self.caret_position.column_index > 0 {
            Position::new(
                self.caret_position.row_index,
                self.caret_position.column_index - 1,
            )
        } else if self.caret_position.row_index > 0 {
            let previous_row = self.caret_position.row_index - 1;
            Position::new(previous_row, self.document.line(previous_row).len())
        } else {
            return;
        };

        let deleted_text = self
            .document
            .delete_range(delete_start, self.caret_position);
        self.edit_history.record(
            EditAction::Delete {
                start: delete_start,
                end: self.caret_position,
                deleted_text,
            },
            true,
        );
        self.caret_position = delete_start;
    }

    /// Performs a delete operation within the editor.
    ///
    /// This function handles two main cases for deletion:
    /// 1. If there is a selected range of text (`self.selection`), it deletes the selected text.
    /// 2. If no text is selected, it performs a single-character deletion (or joins lines if at the end of a line).
    ///
    /// ### Behavior:
    /// - **Case 1: Selection Exists:**
    ///   - Removes the selected text range by calling `self.delete_selected_text(selection)`.
    ///
    /// - **Case 2: No Selection:**
    ///   - Deletes the character at the current caret position.
    ///   - If the caret is at the end of a line, the next line content is merged into the current line, effectively deleting the newline character.
    ///
    /// ### Details:
    /// - The function determines the range of text to be deleted by checking the caret's position:
    ///   - If the caret is within the bounds of the current line, a single character to the right of the caret is deleted.
    ///   - If the caret is at the end of the line and there is another line below, the newline character is deleted, merging the lines.
    ///   - If the caret is at the very end of the document, the function does nothing.
    /// - The deleted range (`delete_end`) is passed to the `self.document.delete_range` method, which performs the deletion in the document.
    /// - The deletion is recorded in the `edit_history` for undo/redo functionality.
    ///
    /// ### Arguments:
    /// - No arguments are explicitly required. It operates on the mutable state of the current editor instance (`self`).
    ///
    /// ### Effects:
    /// - Modifies the document by removing text.
    /// - Updates the edit history with an `EditAction::Delete` entry.
    ///
    /// ### Example Scenario:
    /// - Caret positioned at the beginning of the line: Deletes the first character.
    /// - Caret positioned mid-line: Deletes the character to the right of the caret.
    /// - Caret positioned at the end of the line: Merges the next line into the current one.
    ///
    /// ### Preconditions:
    /// - `self.caret_position` must point to a valid position in the document.
    /// - `self.document` must be properly initialized and hold the text content.
    /// - `self.edit_history` must be enabled to record actions.
    fn perform_delete(&mut self) {
        if let Some(selection) = self.selection.take() {
            self.delete_selected_text(selection);
            return;
        }

        let line_length = self.document.line(self.caret_position.row_index).len();

        let delete_end = if self.caret_position.column_index < line_length {
            Position::new(
                self.caret_position.row_index,
                self.caret_position.column_index + 1,
            )
        } else if self.caret_position.row_index + 1 < self.document.number_of_lines() {
            Position::new(self.caret_position.row_index + 1, 0)
        } else {
            return;
        };

        let deleted_text = self.document.delete_range(self.caret_position, delete_end);
        self.edit_history.record(
            EditAction::Delete {
                start: self.caret_position,
                end: delete_end,
                deleted_text,
            },
            true,
        );
    }

    /// Deletes the text within the specified selection range from the document,
    /// records the deletion in the edit history, and updates the caret position
    /// to the start of the selection.
    ///
    /// # Parameters
    /// - `selection`: A `Selection` object specifying the range of text to delete.
    ///   The range is determined by the `ordered_range()` method, which returns
    ///   the start and end positions in the correct order.
    ///
    /// # Side Effects
    /// - Removes the text within the specified selection range in the document.
    /// - Records the deletion operation in the edit history, encapsulated in an
    ///   `EditAction::Delete` variant, along with the range and the deleted text.
    /// - Updates the caret position to the start of the deleted range.
    ///
    /// # Example
    /// ```rust
    /// let selection = Selection::new(5, 10);
    /// editor.delete_selected_text(selection);
    /// ```
    ///
    /// # Notes
    /// - This method assumes that `self.document.delete_range` modifies the
    ///   document's internal state by removing the specified range of text.
    /// - The edit history uses `record` to log the action, with the `false` argument
    ///   indicating whether the action should be marked as user-initiated.
    /// - After the text is deleted, the caret position is reset to the start of the
    ///   deleted range, allowing further edits or navigation from that position.
    fn delete_selected_text(&mut self, selection: Selection) {
        let (start, end) = selection.ordered_range();
        let deleted_text = self.document.delete_range(start, end);
        self.edit_history.record(
            EditAction::Delete {
                start,
                end,
                deleted_text,
            },
            false,
        );
        self.caret_position = start;
    }

    // ── Clipboard operations ─────────────────────────────────────────

    /// Copies the selected text from the document to the clipboard.
    ///
    /// This method checks if there is a valid text selection and, if so,
    /// extracts the text from the document within the selected range and
    /// stores it in the clipboard. Additionally, it updates the help message
    /// to inform the user that the text has been copied.
    ///
    /// # Behavior
    /// - If a selection exists:
    ///   - The method retrieves the start and end of the selection range,
    ///     ensuring the range is ordered.
    ///   - Extracts the corresponding text from the document based on the range.
    ///   - Stores the extracted text in the clipboard.
    ///   - Updates the help message to `"Copied to clipboard."`.
    /// - If no selection exists, the method does nothing.
    ///
    /// # Examples
    ///
    /// ```rust
    /// // Assuming `editor` is an instance of a struct with this method:
    /// editor.selection = Some(Selection::new(start_position, end_position));
    /// editor.perform_copy();
    /// // The selected text is now copied to the clipboard,
    /// // and the help message reflects this action.
    /// ```
    ///
    /// # Notes
    /// - Ensure that the `selection` is set before calling this method.
    /// - The `clipboard` and `document` should be properly initialized.
    /// - The `help_message` string is replaced, so its previous content will
    ///   be overwritten.
    fn perform_copy(&mut self) {
        if let Some(selection) = &self.selection {
            let (start, end) = selection.ordered_range();
            let text = self.document.extract_text(start, end);
            self.clipboard.store(text);
            self.help_message = "Copied to clipboard.".to_string();
        }
    }

    /// Performs a "cut" operation on the currently selected text in the editor.
    ///
    /// If there is a selection, the following steps are executed:
    /// 1. The selected text is removed from the document.
    /// 2. The removed text is stored in the clipboard for later use.
    /// 3. The operation is recorded in the edit history as a `Delete` action.
    /// 4. The caret position is moved to the start of the selection.
    /// 5. An informational help message is updated to notify the user of the action.
    ///
    /// # Behavior:
    /// - If no selection exists (`self.selection` is `None`), the method does nothing.
    ///
    /// # Side Effects:
    /// - Modifies `self.document` by deleting the selected text.
    /// - Updates `self.clipboard` with the deleted text.
    /// - Records the action in `self.edit_history`.
    /// - Resets `self.caret_position` to the start of the selection.
    /// - Sets `self.help_message` to `"Cut to clipboard."`.
    ///
    /// # Examples:
    /// ```rust
    /// let mut editor = TextEditor::new();
    /// editor.select_range(0, 5); // Select text from index 0 to 5.
    /// editor.perform_cut();      // Cuts the selected text and stores it in the clipboard.
    /// assert_eq!(editor.clipboard.get(), "Hello"); // Assuming the text "Hello" was selected.
    /// assert_eq!(editor.document.text(), " world!"); // Remaining text after the cut.
    /// ```
    fn perform_cut(&mut self) {
        if let Some(selection) = self.selection.take() {
            let (start, end) = selection.ordered_range();
            let deleted_text = self.document.delete_range(start, end);
            self.clipboard.store(deleted_text.clone());
            self.edit_history.record(
                EditAction::Delete {
                    start,
                    end,
                    deleted_text,
                },
                false,
            );
            self.caret_position = start;
            self.help_message = "Cut to clipboard.".to_string();
        }
    }

    /// Performs a paste operation by retrieving the latest text from the clipboard
    /// and inserting it into the current context.
    ///
    /// This method checks if there is any available text in the clipboard. If text is
    /// found, it invokes the `perform_insert` method to insert the text into the
    /// desired location.
    ///
    /// # Behavior
    /// - If the clipboard contains text, it is inserted as-is.
    /// - If the clipboard is empty, the method does nothing.
    ///
    /// # Parameters
    /// This method does not take any parameters directly but operates on the current
    /// state of the instance it is called on.
    ///
    /// # Example
    /// ```rust
    /// // Assuming `self` implements the required methods and clipboard functionality.
    /// self.perform_paste();
    /// ```
    ///
    /// # Preconditions
    /// - Ensure that the clipboard is properly set up and contains accessible text data.
    ///
    /// # Errors
    /// This function does not explicitly handle errors. If `self.clipboard.latest()`
    /// fails or returns `None`, no text is inserted and the method silently exits.
    ///
    /// # Side Effects
    /// - Modifies the current state by inserting the text retrieved from the clipboard.
    fn perform_paste(&mut self) {
        if let Some(text) = self.clipboard.latest() {
            self.perform_insert(&text, false);
        }
    }

    // ── Undo / Redo ──────────────────────────────────────────────────

    /// Performs the undo operation for the most recent action in the edit history.
    ///
    /// This method checks the edit history for the most recent action and undoes it,
    /// if available. The caret position is updated based on the result of applying
    /// the action in reverse. Additionally, any current text selection is cleared.
    ///
    /// # Functionality
    /// - Retrieves the latest action from the `edit_history` by calling `undo`.
    /// - Applies the inverse of the action to the document using `apply_action(&action)`.
    /// - Updates the `caret_position` based on the results of the undo operation.
    /// - Clears the `selection` attribute by setting it to `None`.
    ///
    /// # Behavior
    /// - If no undoable action exists in the `edit_history`, the method exits early
    ///   without making any changes.
    /// - The document's state and caret position are reverted to their state before
    ///   the last action.
    ///
    /// # Requirements
    /// - The `self.edit_history.undo()` method must return an `Option<Action>`, where
    ///   `Action` represents a reversible change made to the document.
    /// - The `self.document.apply_action(&action)` must return the resulting caret
    ///   position after the action is undone.
    ///
    /// # Example
    /// ```rust
    /// editor.perform_undo();
    /// ```
    ///
    /// Here, `perform_undo` reverts the last action in the edit history, updates the
    /// caret position accordingly, and clears any current selection.
    ///
    /// # Side Effects
    /// - Modifies the caret position (`self.caret_position`).
    /// - Modifies the current selection (`self.selection`).
    fn perform_undo(&mut self) {
        if let Some(action) = self.edit_history.undo() {
            self.caret_position = self.document.apply_action(&action);
            self.selection = None;
        }
    }

    /// Redoes the most recently undone action in the edit history.
    ///
    /// This function attempts to retrieve the next action from the edit history's redo stack.
    /// If an action is available, it applies the action to the document and updates the caret
    /// position accordingly. Any existing text selection is cleared as part of this process.
    ///
    /// # Behavior
    /// - If there is no action to redo, the function does nothing.
    /// - If an action is successfully redone, the caret position is updated to the result of
    ///   the action's application, and the text selection is reset to `None`.
    ///
    /// # Preconditions
    /// - The `edit_history` must contain a valid sequence of undo/redo states.
    /// - The `document` must support the application of the action.
    ///
    /// # Side Effects
    /// - Modifies `self.caret_position` to reflect the new caret position after applying the action.
    /// - Clears the current text selection by setting `self.selection` to `None`.
    ///
    /// # Example
    /// ```rust
    /// editor.perform_redo();
    /// ```
    fn perform_redo(&mut self) {
        if let Some(action) = self.edit_history.redo() {
            self.caret_position = self.document.apply_action(&action);
            self.selection = None;
        }
    }

    // ── Scrolling ────────────────────────────────────────────────────

    /// Adjusts the current scroll offsets to ensure that the caret is visible within the terminal's
    /// visible text area. This method recalculates the vertical and horizontal scroll offsets
    /// based on the caret's position and the terminal's dimensions.
    ///
    /// # Details
    /// - If the caret is positioned above the current vertical scroll offset, the vertical scroll
    ///   offset is updated to bring the caret into view.
    /// - If the caret is positioned below the visible text area's height (excluding any reserved buffer),
    ///   the vertical scroll offset is adjusted to include the caret, ensuring it's visible without
    ///   scrolling out of bounds.
    /// - Similarly, this method ensures the caret's horizontal position is within view by adjusting
    ///   the horizontal scroll offset if the caret is outside the visible text area's width.
    ///
    /// The visible text area's width and height are determined by subtracting margins, such as the
    /// gutter width and reserved rows (e.g., for UI elements), from the terminal's dimensions.
    ///
    /// # Errors
    /// This method may return an [`io::Result`] error if there is an issue fetching the terminal's
    /// dimensions using the `terminal::size` function.
    ///
    /// # Returns
    /// Returns `Ok(())` if the scroll adjustments are applied successfully.
    ///
    /// # Usage
    /// This method is typically called during cursor movement operations to ensure the caret
    /// remains visible within the rendered text editor.
    ///
    /// # Example
    /// ```rust
    /// let mut editor = Editor::new();
    /// // Move caret and ensure it's visible.
    /// editor.caret_position = Position { row_index: 10, column_index: 5 };
    /// editor.ensure_caret_is_visible()?;
    /// ```
    fn ensure_caret_is_visible(&mut self) -> io::Result<()> {
        let (terminal_width, terminal_height) = terminal::size()?;

        let visible_text_area_height = terminal_height.saturating_sub(2) as usize;
        let gutter = Gutter::for_line_count(self.document.number_of_lines());
        let visible_text_area_width = (terminal_width as usize).saturating_sub(gutter.width());

        if self.caret_position.row_index < self.vertical_scroll_offset {
            self.vertical_scroll_offset = self.caret_position.row_index;
        } else if self.caret_position.row_index
            >= self.vertical_scroll_offset + visible_text_area_height
        {
            self.vertical_scroll_offset = self
                .caret_position
                .row_index
                .saturating_sub(visible_text_area_height - 1);
        }

        let line = self.document.line(self.caret_position.row_index);
        let caret_display_column =
            tab_handler::display_column(line, self.caret_position.column_index, self.tab_size);
        let scroll_display_column =
            tab_handler::display_column(line, self.horizontal_scroll_offset, self.tab_size);

        if caret_display_column < scroll_display_column {
            self.horizontal_scroll_offset = self.caret_position.column_index;
        } else if caret_display_column >= scroll_display_column + visible_text_area_width {
            let target_display = caret_display_column.saturating_sub(visible_text_area_width - 1);
            let mut byte_offset = 0;
            let mut col = 0;
            for ch in line.chars() {
                if col >= target_display {
                    break;
                }
                if ch == '\t' {
                    col = col + self.tab_size - (col % self.tab_size);
                } else {
                    col += 1;
                }
                byte_offset += ch.len_utf8();
            }
            self.horizontal_scroll_offset = byte_offset;
        }

        Ok(())
    }

    // ── Rendering ────────────────────────────────────────────────────

    /// Renders the text editor interface to the terminal.
    ///
    /// This function handles rendering the visible text, gutters, status bar, and the
    /// caret to the terminal screen. It determines the visible dimensions of the terminal,
    /// calculates the appropriate layout for the text area, and renders each component of
    /// the interface accordingly.
    ///
    /// # Parameters
    /// - `stdout`: A mutable reference to the terminal's stdout handle, which is used for
    ///   writing output directly to the screen.
    ///
    /// # Returns
    /// - `io::Result<()>`: Returns an `Ok(())` result on successful rendering, or an
    ///   `Err` with an I/O error if rendering fails.
    ///
    /// # Behavior
    /// - Determines the terminal's width and height using `terminal::size()`.
    /// - Calculates the visible text area height and width by factoring in the gutter width
    ///   and terminal dimensions.
    /// - Uses `Gutter` for rendering line numbers, which align with the currently visible
    ///   text lines.
    /// - Iteratively renders each visible line of the document within the calculated text area.
    ///   It clears any remaining content at the end of each line and ensures proper positioning
    ///   for the next line.
    /// - Renders additional interface components such as:
    ///   - Status bar at the bottom of the terminal.
    ///   - Prompt bar, if applicable, below the status bar.
    ///   - Caret (cursor) in the appropriate place based on the current editor state.
    /// - Hides the terminal cursor during rendering and ensures it is flushed after all rendering
    ///   is completed.
    ///
    /// # Errors
    /// This function will return an error if any of the terminal I/O operations fail, such as:
    /// - Fetching terminal size.
    /// - Writing to the terminal via the `stdout` handle.
    /// - Flushing the output buffer.
    ///
    /// # Usage
    /// This function is typically called as part of an editor application rendering loop
    /// to continuously update the terminal UI based on user interactions.
    ///
    /// # Example
    /// ```rust
    /// let mut editor = Editor::new();
    /// let mut stdout = io::stdout();
    /// if let Err(e) = editor.render(&mut stdout) {
    ///     eprintln!("Failed to render the editor: {}", e);
    /// }
    /// ```
    fn render(&mut self, stdout: &mut io::Stdout) -> io::Result<()> {
        let (terminal_width, terminal_height) = terminal::size()?;
        let terminal_width = terminal_width as usize;
        let terminal_height = terminal_height as usize;

        let visible_text_area_height = terminal_height.saturating_sub(2);

        let gutter = Gutter::for_line_count(self.document.number_of_lines());
        let gutter_width = gutter.width();
        let text_area_width = terminal_width.saturating_sub(gutter_width);

        queue!(stdout, cursor::Hide, cursor::MoveTo(0, 0))?;

        let mut active_span_state: Option<usize> = None;
        if let Some(definition) = &self.syntax_definition {
            for row in 0..self.vertical_scroll_offset {
                if row < self.document.number_of_lines() {
                    let line = self.document.line(row);
                    let (_, next_state) =
                        syntax_highlighter::highlight_line(definition, line, active_span_state);
                    active_span_state = next_state;
                }
            }
        }

        for screen_row_index in 0..visible_text_area_height {
            let document_row_index = self.vertical_scroll_offset + screen_row_index;
            let is_document_line = document_row_index < self.document.number_of_lines();

            if is_document_line {
                gutter.render_line_number(stdout, Some(document_row_index + 1))?;
            } else {
                gutter.render_line_number(stdout, None)?;
            }

            let highlight_spans = if is_document_line {
                if let Some(definition) = &self.syntax_definition {
                    let line = self.document.line(document_row_index);
                    let (spans, next_state) =
                        syntax_highlighter::highlight_line(definition, line, active_span_state);
                    active_span_state = next_state;
                    spans
                } else {
                    Vec::new()
                }
            } else {
                Vec::new()
            };

            self.render_text_line(stdout, document_row_index, text_area_width, &highlight_spans)?;

            queue!(
                stdout,
                terminal::Clear(ClearType::UntilNewLine),
                cursor::MoveTo(0, (screen_row_index + 1) as u16)
            )?;
        }

        self.render_status_bar(stdout, terminal_width, terminal_height)?;
        self.render_prompt_bar(stdout, terminal_height)?;

        self.render_caret(stdout, gutter_width)?;

        stdout.flush()?;
        Ok(())
    }

    /// Renders a single line of text from the document to the provided standard output stream,
    /// accounting for horizontal scrolling and text selection.
    ///
    /// # Arguments
    ///
    /// * `stdout` - A mutable reference to the standard output stream where the line will be rendered.
    /// * `document_row_index` - The index of the line in the document (0-based) to be rendered.
    /// * `text_area_width` - The width of the rendering area (typically the width of the terminal).
    ///
    /// # Behavior
    ///
    /// - The method supports horizontal scrolling by only rendering the visible portion of the line
    ///   based on the `horizontal_scroll_offset`.
    /// - Handles cases where there is no selection or when there is a selection within the visible range.
    /// - If part of the text is selected, the selected portion is rendered using a reversed attribute
    ///   (or highlighted), while the rest of the text remains normal.
    ///
    /// ## Steps:
    /// 1. Retrieves the line text using its `document_row_index` from the document.
    /// 2. Calculates the visible start and end positions within the line text based on
    ///    the `horizontal_scroll_offset` and `text_area_width`.
    /// 3. Checks if there's a text selection for the specified line:
    ///    - If no selection, the visible part of the line is directly rendered.
    ///    - If a selection exists, the line is split into three segments:
    ///      - Text before the selection.
    ///      - The selected text (highlighted with a reverse attribute).
    ///      - Text after the selection.
    /// 4. Queues the respective segments for rendering to the provided `stdout`.
    ///
    /// # Returns
    ///
    /// * `io::Result<()>` - Returns `Ok(())` if rendering is successful, or an error if
    ///   there is an issue with the I/O operations.
    ///
    /// # Panics
    ///
    /// This function expects that `document_row_index` points to a valid line in the document.
    /// If the index is out of bounds or if `self.document.line()` or slicing operations result
    /// in an invalid range, this may cause a panic.
    ///
    /// # Side Effects
    ///
    /// The method directly modifies the `stdout` stream by queuing rendering commands.
    ///
    /// # Example
    ///
    /// ```rust
    /// let mut stdout = io::stdout();
    /// let document_row_index = 0; // Render the first line of the document.
    /// let text_area_width = 50;  // Assume a terminal width of 50 characters.
    ///
    /// my_renderer.render_text_line(&mut stdout, document_row_index, text_area_width)?;
    /// ```
    ///
    /// # Notes
    ///
    /// Ensure that `stdout` is flushed after this method, if necessary, to display the queued
    /// output on the terminal.

    fn render_text_line(&self, stdout: &mut io::Stdout, document_row_index: usize, text_area_width: usize, highlight_spans: &[HighlightSpan], ) -> io::Result<()> {
        let line_text = self.document.line(document_row_index);
        if line_text.is_empty() {
            return Ok(());
        }

        let (expanded, byte_to_display) =
            tab_handler::expand_tabs(line_text, self.tab_size);
        let expanded_len = expanded.len();

        let mapped_spans: Vec<HighlightSpan> = highlight_spans
            .iter()
            .map(|span| HighlightSpan {
                byte_start: tab_handler::original_byte_to_expanded_byte(
                    &byte_to_display,
                    span.byte_start,
                    expanded_len,
                ),
                byte_end: tab_handler::original_byte_to_expanded_byte(
                    &byte_to_display,
                    span.byte_end,
                    expanded_len,
                ),
                color: span.color,
            })
            .collect();

        let scroll_column =
            tab_handler::display_column(line_text, self.horizontal_scroll_offset, self.tab_size);
        let visible_start = scroll_column.min(expanded_len);
        let visible_end = (scroll_column + text_area_width).min(expanded_len);

        if visible_start >= visible_end {
            return Ok(());
        }

        let visible_text = &expanded[visible_start..visible_end];
        let original_line_length = line_text.len();

        let selection_range =
            self.selection_columns_for_line(document_row_index, original_line_length);
        let mapped_selection = selection_range.map(|(s, e)| {
            let es = tab_handler::original_byte_to_expanded_byte(
                &byte_to_display,
                s,
                expanded_len,
            );
            let ee = tab_handler::original_byte_to_expanded_byte(
                &byte_to_display,
                e,
                expanded_len,
            );
            (es, ee)
        });

        if mapped_spans.is_empty() && mapped_selection.is_none() {
            queue!(stdout, Print(visible_text))?;
            return Ok(());
        }

        let visible_len = visible_end - visible_start;
        let mut colors: Vec<Color> = vec![Color::Reset; visible_len];

        for span in &mapped_spans {
            if span.byte_end <= visible_start || span.byte_start >= visible_end {
                continue;
            }
            let start = span.byte_start.max(visible_start) - visible_start;
            let end = span.byte_end.min(visible_end) - visible_start;
            for i in start..end {
                colors[i] = span.color;
            }
        }

        let (selection_vis_start, selection_vis_end) = match mapped_selection {
            Some((s, e)) => {
                let vs = s.max(visible_start).min(visible_end) - visible_start;
                let ve = e.max(visible_start).min(visible_end) - visible_start;
                (vs, ve)
            }
            None => (0, 0),
        };

        let mut position = 0;
        while position < visible_len {
            let current_color = colors[position];
            let current_selected = position >= selection_vis_start && position < selection_vis_end;

            let mut segment_end = position + 1;
            while segment_end < visible_len {
                let next_selected =
                    segment_end >= selection_vis_start && segment_end < selection_vis_end;
                if colors[segment_end] != current_color || next_selected != current_selected {
                    break;
                }
                segment_end += 1;
            }

            let segment = &visible_text[position..segment_end];
            let has_styling = current_color != Color::Reset || current_selected;

            if has_styling {
                if current_color != Color::Reset {
                    queue!(stdout, SetForegroundColor(current_color))?;
                }
                if current_selected {
                    queue!(stdout, SetAttribute(Attribute::Reverse))?;
                }
                queue!(stdout, Print(segment), SetAttribute(Attribute::Reset))?;
            } else {
                queue!(stdout, Print(segment))?;
            }

            position = segment_end;
        }

        Ok(())
    }

    /// Determines the column range of a selection for a given line within a text component.
    ///
    /// This function computes the start and end column indices for a specific row within
    /// the selection range. If the row is not part of the selection, it returns `None`.
    ///
    /// # Arguments
    ///
    /// * `row_index` - The index of the row for which the column range is being computed.
    /// * `line_length` - The total number of columns (characters) in the line.
    ///
    /// # Returns
    ///
    /// An `Option<(usize, usize)>` containing a tuple:
    /// - `Some((column_start, column_end))` if the row is within the selection range, where:
    ///   - `column_start` is the starting column index of the selection on the row.
    ///   - `column_end` is the exclusive ending column index of the selection on the row.
    /// - `None` if the row is outside the selection range or if the computed range is invalid.
    ///
    /// # Behavior
    ///
    /// - If the row index falls before the start of the selection or after its end, the function
    ///   returns `None`.
    /// - If the row index corresponds to the start or end of the selection range, the starting
    ///   and ending column indices are adjusted accordingly.
    /// - For rows inside the selection range, but not on the start or end rows, the selection
    ///   spans the entire line from column `0` to `line_length`.
    /// - If the calculated start column index is greater than or equal to the end column index,
    ///   the function returns `None`.
    ///
    /// # Example
    ///
    /// ```rust
    /// let selection = MySelection {
    ///     // Assume MySelection implements `selection` and `ordered_range`
    /// };
    /// let row_index = 5;
    /// let line_length = 80;
    ///
    /// if let Some((start, end)) = selection.selection_columns_for_line(row_index, line_length) {
    ///     println!("Selection range on row {}: {} to {}", row_index, start, end);
    /// } else {
    ///     println!("Row {} is not in the selection range.", row_index);
    /// }
    /// ```
    fn selection_columns_for_line(&self, row_index: usize, line_length: usize, ) -> Option<(usize, usize)> {
        let selection = self.selection.as_ref()?;
        let (start, end) = selection.ordered_range();

        if row_index < start.row_index || row_index > end.row_index {
            return None;
        }

        let column_start = if row_index == start.row_index {
            start.column_index
        } else {
            0
        };

        let column_end = if row_index == end.row_index {
            end.column_index
        } else {
            line_length
        };

        if column_start >= column_end {
            return None;
        }

        Some((column_start, column_end))
    }

    /// Renders the status bar for the text editor.
    ///
    /// This function calculates and displays the status bar content, including the file's name,
    /// modification status, help message, and the caret's position (row and column). The status bar
    /// is displayed in reverse color for visibility and adjusted to fit within the terminal's
    /// width constraints. If the status bar content exceeds the terminal width, it is truncated.
    ///
    /// # Arguments
    /// * `stdout` - A mutable reference to the terminal's standard output, used for rendering the status bar.
    /// * `terminal_width` - The width of the terminal in characters.
    /// * `terminal_height` - The height of the terminal in characters, used to determine the status bar's row position.
    ///
    /// # Returns
    /// Returns `io::Result<()>` indicating the success or failure of the status bar rendering operation.
    ///
    /// # Behavior
    /// - The status bar is placed two rows above the terminal's bottom row.
    /// - It includes the file name and a " •" marker if the file has been modified.
    /// - The caret position (`Ln` and `Col`) shows the 1-based row and column indices of the caret.
    /// - A help message is displayed alongside the file name and caret position.
    /// - The content is either padded with spaces or truncated to match the terminal's width.
    ///
    /// # Errors
    /// An error is returned if there are issues with writing to the terminal's `stdout` or queueing the commands.
    ///
    /// # Example
    /// ```rust
    /// use std::io::{self, Write};
    /// use crossterm::{queue, cursor, style::{SetAttribute, Attribute, Print}};
    ///
    /// let mut stdout = io::stdout();
    /// let terminal_width = 80;
    /// let terminal_height = 24;
    ///
    /// // Assuming `self` is a mutable reference to an instance of `Editor`
    /// self.render_status_bar(&mut stdout, terminal_width, terminal_height)?;
    /// ```
    fn render_status_bar(&mut self, stdout: &mut io::Stdout, terminal_width: usize, terminal_height: usize, ) -> io::Result<()> {
        let status_bar_row = (terminal_height - 2) as u16;

        let file_name = self.document.file_name_display();
        let file_name_with_status = if self.document.is_modified() {
            format!("[{}]", file_name)
        } else {
            file_name.clone()
        };
        let language_name = detect_language_from_path(&file_name);

        let caret_display = format!(
            "{} | {} | {}:{} ",
            self.document.encoding_name(),
            language_name,
            self.caret_position.row_index + 1,
            self.caret_position.column_index + 1
        );

        let left_side = format!("{file_name_with_status} | {}", self.help_message);

        let mut status_text = left_side;
        if status_text.len() + caret_display.len() + 1 < terminal_width {
            let padding = terminal_width - status_text.len() - caret_display.len();
            status_text.push_str(&" ".repeat(padding));
            status_text.push_str(&caret_display);
        }

        if status_text.len() > terminal_width {
            status_text.truncate(terminal_width);
        }

        self.bottom_status_bar.set_text(&status_text);

        queue!(
            stdout,
            cursor::MoveTo(0, status_bar_row),
            SetAttribute(Attribute::Reverse),
            Print(self.bottom_status_bar.get_text()),
            SetAttribute(Attribute::Reset),
        )?;

        Ok(())
    }

    fn render_prompt_bar(&self, stdout: &mut io::Stdout, terminal_height: usize) -> io::Result<()> {
        let prompt_bar_row = (terminal_height - 1) as u16;

        queue!(
            stdout,
            cursor::MoveTo(0, prompt_bar_row),
            terminal::Clear(ClearType::CurrentLine),
            Print("")
        )?;

        Ok(())
    }

    fn render_caret(&self, stdout: &mut io::Stdout, gutter_width: usize) -> io::Result<()> {
        let line = self.document.line(self.caret_position.row_index);
        let caret_display_column =
            tab_handler::display_column(line, self.caret_position.column_index, self.tab_size);
        let scroll_display_column =
            tab_handler::display_column(line, self.horizontal_scroll_offset, self.tab_size);

        let caret_screen_column =
            (caret_display_column.saturating_sub(scroll_display_column) + gutter_width) as u16;

        let caret_screen_row = self
            .caret_position
            .row_index
            .saturating_sub(self.vertical_scroll_offset) as u16;

        queue!(
            stdout,
            cursor::MoveTo(caret_screen_column, caret_screen_row),
            cursor::Show
        )?;
        Ok(())
    }

    pub fn status_bar_height(&self) -> usize {
        self.bottom_status_bar.get_height()
    }

    pub fn set_status_bar_text(&mut self, text: &str) {
        self.bottom_status_bar.set_text(text);
    }
}
