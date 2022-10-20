use super::expression::Expression;
use crate::types::Type;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Synchronize {
    type_: Type,
    expression: Rc<Expression>,
}

impl Synchronize {
    pub fn new(type_: impl Into<Type>, expression: impl Into<Expression>) -> Self {
        Self {
            type_: type_.into(),
            expression: expression.into().into(),
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
