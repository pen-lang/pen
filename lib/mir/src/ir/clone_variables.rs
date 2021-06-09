use super::expression::Expression;
use crate::types::Type;
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct CloneVariables {
    variables: HashMap<String, Type>,
    expression: Arc<Expression>,
}

impl CloneVariables {
    pub fn new(variables: HashMap<String, Type>, expression: impl Into<Expression>) -> Self {
        Self {
            variables,
            expression: expression.into().into(),
        }
    }

    pub fn variables(&self) -> &HashMap<String, Type> {
        &self.variables
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
