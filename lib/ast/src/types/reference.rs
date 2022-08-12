use position::Position;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Reference {
    name: String,
    position: Position,
}

impl Reference {
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
