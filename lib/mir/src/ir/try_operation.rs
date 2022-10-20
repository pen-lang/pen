use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

// A try operation matches an operand with a type and returns it from a function
// if it matches.
#[derive(Clone, Debug, PartialEq)]
pub struct TryOperation(Arc<TryOperationInner>);

#[derive(Debug, PartialEq)]
struct TryOperationInner {
    operand: Expression,
    name: String,
    type_: Type,
    then: Expression,
}

impl TryOperation {
    pub fn new(
        operand: impl Into<Expression>,
        name: impl Into<String>,
        type_: impl Into<Type>,
        then: impl Into<Expression>,
    ) -> Self {
        Self(
            TryOperationInner {
                operand: operand.into().into(),
                name: name.into(),
                type_: type_.into(),
                then: then.into().into(),
            }
            .into(),
        )
    }

    pub fn operand(&self) -> &Expression {
        &self.0.operand
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn type_(&self) -> &Type {
        &self.0.type_
    }

    pub fn then(&self) -> &Expression {
        &self.0.then
    }
}
