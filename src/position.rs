#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, PartialOrd, Ord)]
pub struct Position {
    pub row_index: usize,
    pub column_index: usize,
}

impl Position {
    pub fn new(row_index: usize, column_index: usize) -> Self {
        Self {
            row_index,
            column_index,
        }
    }
}
