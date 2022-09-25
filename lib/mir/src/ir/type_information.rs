use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct TypeInformation {
    types: Vec<Type>,
    index: usize,
    variant: Arc<Expression>,
}

impl TypeInformation {
    pub fn new(types: Vec<Type>, index: usize, variant: impl Into<Expression>) -> Self {
        Self {
            types,
            index,
            variant: variant.into().into(),
        }
    }

    pub fn type_(&self) -> &Type {
        &self.types[self.index]
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn variant(&self) -> &Expression {
        &self.variant
    }
}
