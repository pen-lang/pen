use super::{arithmetic_operator::ArithmeticOperator, expression::Expression};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ArithmeticOperation {
    operator: ArithmeticOperator,
    lhs: Rc<Expression>,
    rhs: Rc<Expression>,
}

impl ArithmeticOperation {
    pub fn new(
        operator: ArithmeticOperator,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
    ) -> Self {
        Self {
            operator,
            lhs: Rc::new(lhs.into()),
            rhs: Rc::new(rhs.into()),
        }
    }

    pub fn operator(&self) -> ArithmeticOperator {
        self.operator
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
}
