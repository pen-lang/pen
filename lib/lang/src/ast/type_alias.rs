use crate::{position::Position, types::Type};

#[derive(Clone, Debug, Hash, PartialEq)]
pub struct TypeAlias {
    name: String,
    type_: Type,
    position: Position,
}

impl TypeAlias {
    pub fn new(name: impl Into<String>, type_: impl Into<Type>, position: Position) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
