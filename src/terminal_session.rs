use std::io;

use crossterm::{
    cursor, execute,
    terminal::{self, EnterAlternateScreen, LeaveAlternateScreen},
    event::{EnableBracketedPaste, DisableBracketedPaste},
};

pub struct TerminalSession {
    stdout: io::Stdout,
}

impl TerminalSession {
    pub fn start() -> io::Result<Self> {
        terminal::enable_raw_mode()?;

        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, cursor::Hide, EnableBracketedPaste)?;

        Ok(Self { stdout })
    }

    pub fn stdout_mut(&mut self) -> &mut io::Stdout {
        &mut self.stdout
    }
}

impl Drop for TerminalSession {
    fn drop(&mut self) {
        let _ = execute!(self.stdout, cursor::Show, LeaveAlternateScreen, DisableBracketedPaste);
        let _ = terminal::disable_raw_mode();
    }
}
