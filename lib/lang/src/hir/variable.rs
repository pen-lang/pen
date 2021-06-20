use crate::position::Position;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Variable {
    name: String,
    type_: Option<Type>,
    position: Position,
}

impl Variable {
    pub fn new(name: impl Into<String>, type_: Option<Type>, position: Position) -> Self {
        Self {
            name: name.into(),
            type_,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
