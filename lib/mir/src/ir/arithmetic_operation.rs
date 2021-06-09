use super::{arithmetic_operator::ArithmeticOperator, expression::Expression};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct ArithmeticOperation {
    operator: ArithmeticOperator,
    lhs: Arc<Expression>,
    rhs: Arc<Expression>,
}

impl ArithmeticOperation {
    pub fn new(
        operator: ArithmeticOperator,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
    ) -> Self {
        Self {
            operator,
            lhs: Arc::new(lhs.into()),
            rhs: Arc::new(rhs.into()),
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
