use super::expression::Expression;
use crate::{position::*, types::Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Statement {
    name: Option<String>,
    expression: Expression,
    type_: Option<Type>,
    position: Position,
}

impl Statement {
    pub fn new(
        name: Option<String>,
        expression: impl Into<Expression>,
        type_: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            name,
            expression: expression.into(),
            type_,
            position,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
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
