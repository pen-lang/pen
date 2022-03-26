use position::Position;

pub struct Comment {
    lines: Vec<String>,
    position: Position,
}

impl Comment {
    pub fn new(lines: Vec<String>, position: Position) -> Self {
        Self { lines, position }
    }

    pub fn lines(&self) -> &[String] {
        &self.lines
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
