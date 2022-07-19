use super::expression::Expression;
use crate::types::Type;

#[derive(Clone, Debug, PartialEq)]
pub struct Alternative {
    types: Vec<Type>,
    name: String,
    expression: Expression,
}

impl Alternative {
    pub fn new(
        types: Vec<Type>,
        name: impl Into<String>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self {
            types,
            name: name.into(),
            expression: expression.into(),
        }
    }

    pub fn types(&self) -> &[Type] {
        &self.types
    }

    pub fn type_(&self) -> &Type {
        if let [type_] = &self.types[..] {
            type_
        } else {
            &Type::Variant
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }
}
