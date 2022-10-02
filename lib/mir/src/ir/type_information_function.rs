use super::expression::Expression;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct TypeInformationFunction {
    index: usize,
    variant: Arc<Expression>,
}

impl TypeInformationFunction {
    pub fn new(index: usize, variant: impl Into<Expression>) -> Self {
        Self {
            index,
            variant: variant.into().into(),
        }
    }

    pub fn index(&self) -> usize {
        self.index
    }

    pub fn variant(&self) -> &Expression {
        &self.variant
    }
}
