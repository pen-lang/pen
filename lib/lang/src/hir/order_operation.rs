use super::expression::Expression;
use crate::position::Position;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum OrderOperator {
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct OrderOperation {
    operator: OrderOperator,
    lhs: Arc<Expression>,
    rhs: Arc<Expression>,
    position: Position,
}

impl OrderOperation {
    pub fn new(
        operator: OrderOperator,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            operator,
            lhs: lhs.into().into(),
            rhs: rhs.into().into(),
            position,
        }
    }

    pub fn operator(&self) -> OrderOperator {
        self.operator
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
