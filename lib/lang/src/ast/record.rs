use super::expression::Expression;
use crate::{position::Position, types::Type};
use std::{collections::HashMap, sync::Arc};

#[derive(Clone, Debug, PartialEq)]
pub struct Record {
    type_: Type,
    record: Arc<Option<Expression>>,
    elements: HashMap<String, Expression>,
    position: Position,
}

impl Record {
    pub fn new(
        type_: impl Into<Type>,
        record: Option<Expression>,
        elements: HashMap<String, Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            record: Arc::new(record.into()),
            elements,
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn record(&self) -> Option<&Expression> {
        self.record.as_ref().as_ref()
    }

    pub fn elements(&self) -> &HashMap<String, Expression> {
        &self.elements
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
