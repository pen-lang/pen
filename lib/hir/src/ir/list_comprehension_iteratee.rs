use super::expression::Expression;
use crate::types::Type;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehensionIteratee {
    type_: Option<Type>,
    expression: Expression,
}

impl ListComprehensionIteratee {
    pub fn new(type_: Option<Type>, expression: impl Into<Expression>) -> Self {
        Self {
            type_,
            expression: expression.into(),
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.expression.position()
    }
}
