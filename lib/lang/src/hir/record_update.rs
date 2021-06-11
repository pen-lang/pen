use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::{collections::BTreeMap, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct RecordUpdate {
    type_: Type,
    argument: Arc<Expression>,
    elements: BTreeMap<String, Expression>,
    position: Position,
}

impl RecordUpdate {
    pub fn new(
        type_: impl Into<Type>,
        argument: impl Into<Expression>,
        elements: BTreeMap<String, Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            argument: Arc::new(argument.into()),
            elements,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn argument(&self) -> &Expression {
        &self.argument
    }

    pub fn elements(&self) -> &BTreeMap<String, Expression> {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
