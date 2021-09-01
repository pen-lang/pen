use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfList {
    type_: Option<Type>, // element type
    argument: Arc<Expression>,
    first_name: String,
    rest_name: String,
    then: Arc<Expression>,
    else_: Arc<Expression>,
    position: Position,
}

impl IfList {
    pub fn new(
        type_: Option<Type>,
        argument: impl Into<Expression>,
        first_name: impl Into<String>,
        rest_name: impl Into<String>,
        then: impl Into<Expression>,
        else_: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_,
            argument: Arc::new(argument.into()),
            first_name: first_name.into(),
            rest_name: rest_name.into(),
            then: then.into().into(),
            else_: else_.into().into(),
            position,
        }
    }

    pub fn type_(&self) -> Option<&Type> {
        self.type_.as_ref()
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn first_name(&self) -> &str {
        &self.first_name
    }

    pub fn rest_name(&self) -> &str {
        &self.rest_name
    }

    pub fn then(&self) -> &Expression {
        &self.then
    }

    pub fn else_(&self) -> &Expression {
        &self.else_
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
