use super::{
    alternative::Alternative, default_alternative::DefaultAlternative, expression::Expression,
};
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Case {
    argument: Rc<Expression>,
    alternatives: Vec<Alternative>,
    default_alternative: Option<DefaultAlternative>,
}

impl Case {
    pub fn new(
        argument: impl Into<Expression>,
        alternatives: Vec<Alternative>,
        default_alternative: Option<DefaultAlternative>,
    ) -> Self {
        Self {
            argument: Rc::new(argument.into()),
            alternatives,
            default_alternative,
        }
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn alternatives(&self) -> &[Alternative] {
        &self.alternatives
    }

    pub fn default_alternative(&self) -> Option<&DefaultAlternative> {
        self.default_alternative.as_ref()
    }
}
