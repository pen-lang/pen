use super::{binary_operator::BinaryOperator, expression::Expression};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct BinaryOperation {
    operator: BinaryOperator,
    lhs: Rc<Expression>,
    rhs: Rc<Expression>,
    position: Position,
}

impl BinaryOperation {
    pub fn new(
        operator: BinaryOperator,
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

    pub fn operator(&self) -> BinaryOperator {
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
