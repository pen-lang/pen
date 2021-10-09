use super::expression::Expression;
use crate::types;

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    type_: types::Record,
    fields: Vec<Expression>,
}

impl Record {
    pub fn new(type_: types::Record, fields: Vec<Expression>) -> Self {
        Self { type_, fields }
    }

    pub fn type_(&self) -> &types::Record {
        &self.type_
    }

    pub fn fields(&self) -> &[Expression] {
        &self.fields
    }
}
