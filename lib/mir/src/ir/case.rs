use super::{
    alternative::Alternative, default_alternative::DefaultAlternative, expression::Expression,
};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Case(Arc<CaseInner>);

#[derive(Debug, PartialEq)]
struct CaseInner {
    argument: Expression,
    alternatives: Vec<Alternative>,
    default_alternative: Option<DefaultAlternative>,
}

impl Case {
    pub fn new(
        argument: impl Into<Expression>,
        alternatives: Vec<Alternative>,
        default_alternative: Option<DefaultAlternative>,
    ) -> Self {
        Self(
            CaseInner {
                argument: argument.into(),
                alternatives,
                default_alternative,
            }
            .into(),
        )
    }

    pub fn argument(&self) -> &Expression {
        &self.0.argument
    }

    pub fn alternatives(&self) -> &[Alternative] {
        &self.0.alternatives
    }

    pub fn default_alternative(&self) -> Option<&DefaultAlternative> {
        self.0.default_alternative.as_ref()
    }
}
