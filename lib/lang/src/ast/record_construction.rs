use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
pub struct RecordConstruction {
    type_: Type,
    elements: HashMap<String, Expression>,
    position: Position,
}

impl RecordConstruction {
    pub fn new(
        type_: impl Into<Type>,
        elements: HashMap<String, Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            elements,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn elements(&self) -> &HashMap<String, Expression> {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
