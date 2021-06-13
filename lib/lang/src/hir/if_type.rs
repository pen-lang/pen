use super::{alternative::Alternative, expression::Expression, Block};
use crate::{position::Position, types::Type};
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfType {
    name: String,
    argument: Arc<Expression>,
    argument_type: Option<Type>,
    alternatives: Vec<Alternative>,
    default_alternative: Option<Block>,
    result_type: Option<Type>,
    position: Position,
}

impl IfType {
    pub fn new(
        name: impl Into<String>,
        argument: impl Into<Expression>,
        argument_type: Option<Type>,
        alternatives: Vec<Alternative>,
        default_alternative: Option<Block>,
        result_type: Option<Type>,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            argument: argument.into().into(),
            argument_type,
            alternatives,
            default_alternative,
            result_type,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn argument_type(&self) -> Option<&Type> {
        self.argument_type.as_ref()
    }

    pub fn alternatives(&self) -> &[Alternative] {
        &self.alternatives
    }

    pub fn default_alternative(&self) -> Option<&Block> {
        self.default_alternative.as_ref()
    }

    pub fn result_type(&self) -> Option<&Type> {
        self.result_type.as_ref()
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
