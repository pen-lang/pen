use super::{comparison_operator::ComparisonOperator, expression::Expression};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonOperation(Arc<ComparisonOperationInner>);

#[derive(Debug, PartialEq)]
struct ComparisonOperationInner {
    operator: ComparisonOperator,
    lhs: Expression,
    rhs: Expression,
}

impl ComparisonOperation {
    pub fn new(
        operator: ComparisonOperator,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
    ) -> Self {
        Self(
            ComparisonOperationInner {
                operator,
                lhs: (lhs.into()),
                rhs: (rhs.into()),
            }
            .into(),
        )
    }

    pub fn operator(&self) -> ComparisonOperator {
        self.0.operator
    }

    pub fn lhs(&self) -> &Expression {
        &self.0.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.0.rhs
    }
}
