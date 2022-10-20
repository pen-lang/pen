use super::expression::Expression;
use crate::types::Type;
use fnv::FnvHashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct DropVariables {
    variables: FnvHashMap<String, Type>,
    expression: Rc<Expression>,
}

impl DropVariables {
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
