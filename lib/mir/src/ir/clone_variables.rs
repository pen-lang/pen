use super::expression::Expression;
use crate::types::Type;
use fnv::FnvHashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct CloneVariables {
    variables: FnvHashMap<String, Type>,
    expression: Arc<Expression>,
}

impl CloneVariables {
    pub fn new(variables: FnvHashMap<String, Type>, expression: impl Into<Expression>) -> Self {
        Self {
            variables,
            expression: expression.into().into(),
        }
    }

    pub fn variables(&self) -> &FnvHashMap<String, Type> {
        &self.variables
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
