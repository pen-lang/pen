use super::{expression::Expression, Block};
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfList {
    argument: Arc<Expression>,
    first_name: String,
    rest_name: String,
    then: Arc<Block>,
    else_: Arc<Block>,
    result_type: Option<Type>,
    position: Position,
}

impl IfList {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        argument: impl Into<Expression>,
        first_name: impl Into<String>,
        rest_name: impl Into<String>,
        then: Block,
        else_: Block,
        result_type: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            argument: Arc::new(argument.into()),
            first_name: first_name.into(),
            rest_name: rest_name.into(),
            then: then.into(),
            else_: else_.into(),
            result_type,
            position,
        }
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

    pub fn then(&self) -> &Block {
        &self.then
    }

    pub fn else_(&self) -> &Block {
        &self.else_
    }

    pub fn result_type(&self) -> Option<&Type> {
        self.result_type.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
