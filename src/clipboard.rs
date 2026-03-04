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
            match os_clipboard.get_text() {
                Ok(text) => {
                    if !text.is_empty() {
                        return Some(text);
                    }
                }
                Err(_e) => {
                    // Try to re-initialize if the clipboard handle is stale
                    if let Ok(mut new_clipboard) = OsClipboard::new() {
                        if let Ok(text) = new_clipboard.get_text() {
                            self.os_clipboard = Some(new_clipboard);
                            if !text.is_empty() {
                                return Some(text);
                            }
                        }
                    }
                }
            }
        } else {
            // Try to initialize if it was None
            if let Ok(mut new_clipboard) = OsClipboard::new() {
                if let Ok(text) = new_clipboard.get_text() {
                    self.os_clipboard = Some(new_clipboard);
                    if !text.is_empty() {
                        return Some(text);
                    }
                } else {
                    self.os_clipboard = Some(new_clipboard);
                }
            }
        }
        self.entries.last().cloned()
    }
}
