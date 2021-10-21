use super::expression::Expression;
use crate::types::Type;
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct CloneVariables {
    variables: BTreeMap<String, Type>,
    expression: Arc<Expression>,
}

impl CloneVariables {
    pub fn new(variables: BTreeMap<String, Type>, expression: impl Into<Expression>) -> Self {
        Self {
            variables,
            expression: expression.into().into(),
        }
    }

    pub fn variables(&self) -> &BTreeMap<String, Type> {
        &self.variables
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
