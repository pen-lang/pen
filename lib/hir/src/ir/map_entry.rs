use super::expression::Expression;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct MapEntry {
    key: Rc<Expression>,
    value: Rc<Expression>,
    position: Position,
}

impl MapEntry {
    pub fn new(
        key: impl Into<Expression>,
        value: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            key: key.into().into(),
            value: value.into().into(),
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
