use super::{comparison_operator::ComparisonOperator, expression::Expression};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ComparisonOperation {
    operator: ComparisonOperator,
    lhs: Rc<Expression>,
    rhs: Rc<Expression>,
}

impl ComparisonOperation {
    pub fn new(
        operator: ComparisonOperator,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
    ) -> Self {
        Self {
            operator,
            lhs: Rc::new(lhs.into()),
            rhs: Rc::new(rhs.into()),
        }
    }

    pub fn operator(&self) -> ComparisonOperator {
        self.operator
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }
}
