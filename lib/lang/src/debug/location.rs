use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, Deserialize, Serialize)]
pub struct Location {
    line_number: usize,
    column_number: usize,
}

impl Location {
    pub fn new(line_number: usize, column_number: usize) -> Self {
        Self {
            line_number,
            column_number,
        }
    }

    pub fn line_number(&self) -> usize {
        self.line_number
    }

    pub fn column_number(&self) -> usize {
        self.column_number
    }
}
