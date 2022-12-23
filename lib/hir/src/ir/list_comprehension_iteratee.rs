use super::expression::Expression;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehensionIteratee {
    type_: Option<Type>,
    expression: Expression,
    position: Position,
}

impl ListComprehensionIteratee {
    pub fn new(type_: Option<Type>, expression: impl Into<Expression>, position: Position) -> Self {
        Self {
            type_,
            expression: expression.into(),
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
