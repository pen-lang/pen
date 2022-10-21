use super::expression::Expression;
use position::Position;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum BooleanOperator {
    And,
    Or,
}

#[derive(Clone, Debug, PartialEq)]
pub struct BooleanOperation {
    operator: BooleanOperator,
    lhs: Rc<Expression>,
    rhs: Rc<Expression>,
    position: Position,
}

impl BooleanOperation {
    pub fn new(
        operator: BooleanOperator,
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

    pub fn operator(&self) -> BooleanOperator {
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
