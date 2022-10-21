use super::{expression::Expression, unary_operator::UnaryOperator};
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct UnaryOperation {
    operator: UnaryOperator,
    expression: Rc<Expression>,
    position: Position,
}

impl UnaryOperation {
    pub fn new(
        operator: UnaryOperator,
        expression: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            operator,
            expression: expression.into().into(),
            position,
        }
    }

    pub fn operator(&self) -> UnaryOperator {
        self.operator
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
