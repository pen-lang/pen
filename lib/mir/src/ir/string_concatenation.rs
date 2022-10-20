use super::expression::Expression;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct StringConcatenation {
    operands: Rc<Vec<Expression>>,
}

impl StringConcatenation {
    pub fn new(operands: Vec<Expression>) -> Self {
        Self {
            operands: operands.into(),
        }
    }

    pub fn operands(&self) -> &[Expression] {
        &self.operands
    }
}
