use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct None {
    position: Position,
}

impl None {
    pub fn new(position: Position) -> Self {
        Self { position }
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
