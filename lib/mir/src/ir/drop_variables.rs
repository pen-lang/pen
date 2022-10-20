use super::expression::Expression;
use crate::types::Type;
use fnv::FnvHashMap;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct DropVariables(Arc<DropVariablesInner>);

#[derive(Debug, PartialEq)]
struct DropVariablesInner {
    variables: FnvHashMap<String, Type>,
    expression: Expression,
}

impl DropVariables {
    pub fn new(variables: FnvHashMap<String, Type>, expression: impl Into<Expression>) -> Self {
        Self(
            DropVariablesInner {
                variables,
                expression: expression.into().into(),
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
