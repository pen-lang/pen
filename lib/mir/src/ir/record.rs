use super::expression::Expression;
use crate::types;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Record(Rc<RecordInner>);

#[derive(Debug, PartialEq)]
struct RecordInner {
    type_: types::Record,
    fields: Vec<Expression>,
}

impl Record {
    pub fn new(type_: types::Record, fields: Vec<Expression>) -> Self {
        Self(RecordInner { type_, fields }.into())
    }

    pub fn type_(&self) -> &types::Record {
        &self.0.type_
    }

    pub fn fields(&self) -> &[Expression] {
        &self.0.fields
    }
}
