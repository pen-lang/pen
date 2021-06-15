use super::expression::Expression;
use crate::{position::*, types::Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    name: String,
    expression: Expression,
    type_: Option<Type>,
    position: Position,
}

impl Statement {
    pub fn new(
        name: impl Into<String>,
        expression: impl Into<Expression>,
        type_: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            expression: expression.into(),
            type_,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
