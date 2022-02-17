use super::{argument::Argument, expression::Expression};
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Thunk {
    // See also an environment inference pass.
    environment: Vec<Argument>,
    body: Arc<Expression>,
    type_: Type,
}

impl Thunk {
    pub fn new(body: impl Into<Expression>, type_: impl Into<Type>) -> Self {
        Self::with_environment(vec![], body, type_)
    }

    pub(crate) fn with_environment(
        environment: Vec<Argument>,
        body: impl Into<Expression>,
        type_: impl Into<Type>,
    ) -> Self {
        Self {
            environment,
            body: body.into().into(),
            type_: type_.into(),
        }
    }

    pub fn environment(&self) -> &[Argument] {
        &self.environment
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }
}
