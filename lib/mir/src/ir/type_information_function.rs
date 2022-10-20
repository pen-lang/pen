use super::expression::Expression;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct TypeInformationFunction {
    variant: Rc<Expression>,
}

impl TypeInformationFunction {
    pub fn new(variant: impl Into<Expression>) -> Self {
        Self {
            variant: variant.into().into(),
        }
    }

    pub fn variant(&self) -> &Expression {
        &self.variant
    }
}
