use std::env;

use crate::editor::Editor;
use crate::terminal_session::TerminalSession;

mod clipboard;
mod document;
mod edit_history;
mod editor;
mod file;
mod file_detector;
mod gutter;
mod position;
mod selection;
mod syntax_definition;
mod syntax_highlighter;
mod terminal_session;
pub mod config;

fn ensure_configs_exist() {
    config::configs::load_user_settings();
    config::configs::load_editor_settings();
    config::configs::load_color_scheme();
}

fn main() -> std::io::Result<()> {
    let mut terminal_session = TerminalSession::start()?;
    ensure_configs_exist();

    let file_path = env::args().nth(1);

    let mut editor = match file_path {
        Some(path) => Editor::open(path)?,
        None => Editor::new(),
    };

    editor.run(terminal_session.stdout_mut())
}
