use super::expression::Expression;
use crate::{position::*, types::Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Assignment {
    name: String,
    expression: Expression,
    type_: Type,
    position: Position,
}

impl Assignment {
    pub fn new(
        name: impl Into<String>,
        expression: impl Into<Expression>,
        type_: impl Into<Type>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            expression: expression.into(),
            type_: type_.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
