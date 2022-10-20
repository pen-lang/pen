use super::{arithmetic_operator::ArithmeticOperator, expression::Expression};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct ArithmeticOperation(Arc<ArithmeticOperationInner>);

#[derive(Clone, Debug, PartialEq)]
struct ArithmeticOperationInner {
    operator: ArithmeticOperator,
    lhs: Expression,
    rhs: Expression,
}

impl ArithmeticOperation {
    pub fn new(
        operator: ArithmeticOperator,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
    ) -> Self {
        Self(
            ArithmeticOperationInner {
                operator,
                lhs: (lhs.into()),
                rhs: (rhs.into()),
            }
            .into(),
        )
    }

    pub fn operator(&self) -> ArithmeticOperator {
        self.0.operator
    }

    pub fn lhs(&self) -> &Expression {
        &self.0.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.0.rhs
    }
}
