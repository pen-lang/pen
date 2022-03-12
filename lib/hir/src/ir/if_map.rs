use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::sync::Arc;

#[derive(Clone, Debug, PartialEq)]
pub struct IfMap {
    key_type: Option<Type>,
    value_type: Option<Type>,
    name: String,
    map: Arc<Expression>,
    key: Arc<Expression>,
    then: Arc<Expression>,
    else_: Arc<Expression>,
    position: Position,
}

impl IfMap {
    pub fn new(
        key_type: Option<Type>,
        value_type: Option<Type>,
        name: impl Into<String>,
        map: impl Into<Expression>,
        key: impl Into<Expression>,
        then: impl Into<Expression>,
        else_: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            key_type,
            value_type,
            name: name.into(),
            map: map.into().into(),
            key: key.into().into(),
            then: then.into().into(),
            else_: else_.into().into(),
            position,
        }
    }

    pub fn key_type(&self) -> Option<&Type> {
        self.key_type.as_ref()
    }

    pub fn value_type(&self) -> Option<&Type> {
        self.value_type.as_ref()
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn map(&self) -> &Expression {
        &self.map
    }

    pub fn key(&self) -> &Expression {
        &self.key
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
