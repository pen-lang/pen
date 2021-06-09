use super::expression::Expression;
use crate::types::Type;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    name: String,
    type_: Type,
    bound_expression: Arc<Expression>,
    expression: Arc<Expression>,
}

impl Let {
    pub fn new(
        name: impl Into<String>,
        type_: impl Into<Type>,
        bound_expression: impl Into<Expression>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self {
            name: name.into(),
            type_: type_.into(),
            bound_expression: bound_expression.into().into(),
            expression: expression.into().into(),
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn bound_expression(&self) -> &Expression {
        &self.bound_expression
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
