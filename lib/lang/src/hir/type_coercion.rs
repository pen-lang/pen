use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct TypeCoercion {
    from: Type,
    to: Type,
    argument: Arc<Expression>,
    position: Position,
}

impl TypeCoercion {
    pub fn new(
        from: impl Into<Type>,
        to: impl Into<Type>,
        argument: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
            argument: Arc::new(argument.into()),
            position,
        }
    }

    pub fn from(&self) -> &Type {
        &self.from
    }

    pub fn to(&self) -> &Type {
        &self.to
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
