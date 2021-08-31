use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct TryOperation {
    type_: Option<Type>,
    expression: Arc<Expression>,
    position: Position,
}

impl TryOperation {
    pub fn new(type_: Option<Type>, expression: impl Into<Expression>, position: Position) -> Self {
        Self {
            type_,
            expression: expression.into().into(),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
