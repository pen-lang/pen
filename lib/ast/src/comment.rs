use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Comment {
    line: String,
    position: Position,
}

impl Comment {
    pub fn new(line: impl Into<String>, position: Position) -> Self {
        Self {
            line: line.into(),
            position,
        }
    }

    pub fn line(&self) -> &str {
        &self.line
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
