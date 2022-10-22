use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EqualityOperator {
    Equal,
    NotEqual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EqualityOperation {
    type_: Option<Type>,
    operator: EqualityOperator,
    lhs: Rc<Expression>,
    rhs: Rc<Expression>,
    position: Position,
}

impl EqualityOperation {
    pub fn new(
        type_: Option<Type>,
        operator: EqualityOperator,
        lhs: impl Into<Expression>,
        rhs: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_,
            operator,
            lhs: Rc::new(lhs.into()),
            rhs: Rc::new(rhs.into()),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn operator(&self) -> EqualityOperator {
        self.operator
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
