use super::{alternative::Alternative, expression::Expression, Block};
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfType {
    type_: Type,
    name: String,
    argument: Arc<Expression>,
    alternatives: Vec<Alternative>,
    default_alternative: Option<Block>,
    position: Position,
}

impl IfType {
    pub fn new(
        type_: impl Into<Type>,
        name: impl Into<String>,
        argument: impl Into<Expression>,
        alternatives: Vec<Alternative>,
        default_alternative: Block,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            name: name.into(),
            argument: argument.into().into(),
            alternatives,
            default_alternative: default_alternative.into(),
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn type_(&self) -> &Type {
        &self.type_
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
