use super::expression::Expression;
use crate::types::Type;
use fnv::FnvHashMap;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct CloneVariables(Rc<CloneVariablesInner>);

#[derive(Debug, PartialEq)]
struct CloneVariablesInner {
    variables: FnvHashMap<String, Type>,
    expression: Expression,
}

impl CloneVariables {
    pub fn new(variables: FnvHashMap<String, Type>, expression: impl Into<Expression>) -> Self {
        Self(
            CloneVariablesInner {
                variables,
                expression: expression.into(),
            }
            .into(),
        )
    }

    pub fn variables(&self) -> &FnvHashMap<String, Type> {
        &self.0.variables
    }

    pub fn expression(&self) -> &Expression {
        &self.0.expression
    }
}
