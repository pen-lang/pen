use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Variable {
    name: String,
    position: Position,
}

impl Variable {
    pub fn new(name: impl Into<String>, position: Position) -> Self {
        Self {
            name: name.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
