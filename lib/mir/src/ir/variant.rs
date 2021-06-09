use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Variant {
    type_: Type,
    payload: Arc<Expression>,
}

impl Variant {
    pub fn new(type_: impl Into<Type>, payload: impl Into<Expression>) -> Self {
        Self {
            type_: type_.into(),
            payload: payload.into().into(),
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn payload(&self) -> &Expression {
        &self.payload
    }
}
