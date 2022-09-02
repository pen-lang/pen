use position::Position;

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct UnqualifiedName {
    name: String,
    position: Position,
}

impl UnqualifiedName {
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
