use super::expression::Expression;
use crate::types::Type;
use std::rc::Rc;

// A try operation matches an operand with a type and returns it from a function
// if it matches.
#[derive(Clone, Debug, PartialEq)]
pub struct TryOperation {
    operand: Rc<Expression>,
    name: String,
    type_: Type,
    then: Rc<Expression>,
}

impl TryOperation {
    pub fn new(
        operand: impl Into<Expression>,
        name: impl Into<String>,
        type_: impl Into<Type>,
        then: impl Into<Expression>,
    ) -> Self {
        Self {
            operand: operand.into().into(),
            name: name.into(),
            type_: type_.into(),
            then: then.into().into(),
        }
    }

    pub fn operand(&self) -> &Expression {
        &self.operand
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn then(&self) -> &Expression {
        &self.then
    }
}
