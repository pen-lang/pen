use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Boolean {
    value: bool,
    position: Position,
}

impl Boolean {
    pub fn new(value: bool, position: Position) -> Self {
        Self { value, position }
    }

    pub fn value(&self) -> bool {
        self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
