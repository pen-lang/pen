use super::{Argument, Expression};
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Lambda {
    arguments: Vec<Argument>,
    result_type: Type,
    body: Rc<Expression>,
    position: Position,
}

impl Lambda {
    pub fn new(
        arguments: Vec<Argument>,
        result_type: impl Into<Type>,
        body: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            arguments,
            result_type: result_type.into(),
            body: body.into().into(),
            position,
        }
    }

    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn result_type(&self) -> &Type {
        &self.result_type
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
