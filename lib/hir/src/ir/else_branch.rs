use super::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ElseBranch {
    type_: Option<Type>,
    expression: Rc<Expression>,
    position: Position,
}

impl ElseBranch {
    pub fn new(type_: Option<Type>, expression: impl Into<Expression>, position: Position) -> Self {
        Self {
            type_,
            expression: expression.into().into(),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn expression(&self) -> &Expression {
        &self.expression
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
