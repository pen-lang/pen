use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Number {
    value: f64,
    position: Position,
}

impl Number {
    pub fn new(value: f64, position: Position) -> Self {
        Self { value, position }
    }

    pub fn value(&self) -> f64 {
        self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
