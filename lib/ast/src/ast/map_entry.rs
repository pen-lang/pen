use crate::Expression;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct MapEntry {
    key: Expression,
    value: Expression,
    position: Position,
}

impl MapEntry {
    pub fn new(
        key: impl Into<Expression>,
        value: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            key: key.into(),
            value: value.into(),
            position,
        }
    }

    pub fn key(&self) -> &Expression {
        &self.key
    }

    pub fn value(&self) -> &Expression {
        &self.value
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
