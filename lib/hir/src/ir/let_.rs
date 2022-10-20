use super::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Let {
    name: Option<String>,
    type_: Option<Type>,
    bound_expression: Rc<Expression>,
    expression: Rc<Expression>,
    position: Position,
}

impl Let {
    pub fn new(
        name: Option<String>,
        type_: Option<Type>,
        bound_expression: impl Into<Expression>,
        expression: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            name,
            type_,
            bound_expression: bound_expression.into().into(),
            expression: expression.into().into(),
            position,
        }
    }

    pub fn name(&self) -> Option<&str> {
        self.name.as_deref()
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn bound_expression(&self) -> &Expression {
        &self.bound_expression
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
