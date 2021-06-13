use super::{alternative::Alternative, expression::Expression, Block};
use crate::position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfType {
    name: String,
    argument: Arc<Expression>,
    alternatives: Vec<Alternative>,
    default_alternative: Option<Block>,
    position: Position,
}

impl IfType {
    pub fn new(
        name: impl Into<String>,
        argument: impl Into<Expression>,
        alternatives: Vec<Alternative>,
        default_alternative: Option<Block>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            argument: argument.into().into(),
            alternatives,
            default_alternative,
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

    pub fn default_alternative(&self) -> Option<&Block> {
        self.default_alternative.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
