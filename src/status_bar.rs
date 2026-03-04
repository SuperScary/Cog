pub struct BottomStatusBar {
    height: usize,
    width: usize,
    text: String,
}

impl BottomStatusBar {
    pub fn new(height: usize, width: usize, text: &str) -> BottomStatusBar {
        BottomStatusBar { height, width, text: text.to_string() }
    }
    
    pub fn get_text(&self) -> &str {
        &self.text
    }
    
    pub fn set_text(&mut self, text: &str) {
        self.text = text.to_string();   
    }
    
    pub fn get_height(&self) -> usize {
        self.height
    }
    
    pub fn get_width(&self) -> usize {
        self.width
    }   
    
}

// Currently unused. Possible to use in the future.
pub struct TopStatusBar {
    height: usize,
    width: usize,
    text: String,   
}