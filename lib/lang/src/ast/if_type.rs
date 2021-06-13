use super::{alternative::Alternative, expression::Expression, Block};
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfType {
    name: String,
    argument: Arc<Expression>,
    alternatives: Vec<Alternative>,
    else_: Option<Block>,
    position: Position,
}

impl IfType {
    pub fn new(
        name: impl Into<String>,
        argument: impl Into<Expression>,
        alternatives: Vec<Alternative>,
        else_: Option<Block>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            argument: argument.into().into(),
            alternatives,
            else_,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn alternatives(&self) -> &[Alternative] {
        &self.alternatives
    }

    pub fn else_(&self) -> Option<&Block> {
        self.else_.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
