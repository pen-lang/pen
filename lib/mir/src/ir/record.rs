use super::expression::Expression;
use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    type_: types::Record,
    elements: Vec<Expression>,
}

impl Record {
    pub fn new(type_: types::Record, elements: Vec<Expression>) -> Self {
        Self { type_, elements }
    }

    pub fn type_(&self) -> &types::Record {
        &self.type_
    }

    pub fn elements(&self) -> &[Expression] {
        &self.elements
    }
}
