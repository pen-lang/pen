use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Synchronize(Arc<SynchronizeInner>);

#[derive(Debug, PartialEq)]
struct SynchronizeInner {
    type_: Type,
    expression: Expression,
}

impl Synchronize {
    pub fn new(type_: impl Into<Type>, expression: impl Into<Expression>) -> Self {
        Self(
            SynchronizeInner {
                type_: type_.into(),
                expression: expression.into().into(),
            }
            .into(),
        )
    }

    pub fn type_(&self) -> &Type {
        &self.0.type_
    }

    pub fn expression(&self) -> &Expression {
        &self.0.expression
    }
}
