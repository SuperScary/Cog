use arboard::Clipboard as OsClipboard;

pub struct Clipboard {
    entries: Vec<String>,
    os_clipboard: Option<OsClipboard>,
}

impl Clipboard {
    pub fn new() -> Self {
        let os_clipboard = OsClipboard::new().ok();
        Self {
            entries: Vec::new(),
            os_clipboard,
        }
    }

    pub fn store(&mut self, text: String) {
        if text.is_empty() {
            return;
        }
        if let Some(os_clipboard) = &mut self.os_clipboard {
            let _ = os_clipboard.set_text(&text);
        }
        self.entries.push(text);
    }

    /// Reads from the OS clipboard first (so paste works cross-application),
    /// falling back to the internal history if the OS clipboard is unavailable.
    pub fn latest(&mut self) -> Option<String> {
        if let Some(os_clipboard) = &mut self.os_clipboard {
            if let Ok(text) = os_clipboard.get_text() {
                if !text.is_empty() {
                    return Some(text);
                }
            }
        }
        self.entries.last().cloned()
    }
}
