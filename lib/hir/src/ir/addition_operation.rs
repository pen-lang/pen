use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct AdditionOperation {
    type_: Option<Type>,
    lhs: Rc<Expression>,
    rhs: Rc<Expression>,
    position: Position,
}

impl AdditionOperation {
    pub fn new(
        type_: Option<Type>,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_,
            lhs: lhs.into().into(),
            rhs: rhs.into().into(),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn lhs(&self) -> &Expression {
        &self.lhs
    }

    pub fn rhs(&self) -> &Expression {
        &self.rhs
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
