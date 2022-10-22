use super::expression::Expression;
use crate::types::Type;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Variant(Arc<VariantInner>);

#[derive(Debug, PartialEq)]
struct VariantInner {
    type_: Type,
    payload: Expression,
}

impl Variant {
    pub fn new(type_: impl Into<Type>, payload: impl Into<Expression>) -> Self {
        Self(
            VariantInner {
                type_: type_.into(),
                payload: payload.into(),
            }
            .into(),
        )
    }

    pub fn type_(&self) -> &Type {
        &self.0.type_
    }

    pub fn payload(&self) -> &Expression {
        &self.0.payload
    }
}
