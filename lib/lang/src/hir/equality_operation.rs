use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::sync::Arc;

#[derive(Copy, Clone, Debug, Eq, PartialEq)]
pub enum EqualityOperator {
    Equal,
    NotEqual,
}

#[derive(Clone, Debug, PartialEq)]
pub struct EqualityOperation {
    type_: Option<Type>,
    operator: EqualityOperator,
    lhs: Arc<Expression>,
    rhs: Arc<Expression>,
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
            lhs: Arc::new(lhs.into()),
            rhs: Arc::new(rhs.into()),
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
