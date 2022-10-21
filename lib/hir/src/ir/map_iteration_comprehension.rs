use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct MapIterationComprehension {
    key_type: Option<Type>,
    value_type: Option<Type>,
    element_type: Type,
    element: Rc<Expression>,
    key_name: String,
    value_name: String,
    map: Rc<Expression>,
    position: Position,
}

impl MapIterationComprehension {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        key_type: Option<Type>,
        value_type: Option<Type>,
        element_type: impl Into<Type>,
        element: impl Into<Expression>,
        key_name: impl Into<String>,
        value_name: impl Into<String>,
        map: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            key_type,
            value_type,
            element_type: element_type.into(),
            element: element.into().into(),
            key_name: key_name.into(),
            value_name: value_name.into(),
            map: map.into().into(),
            position,
        }
    }

    pub fn key_type(&self) -> Option<&Type> {
        self.key_type.as_ref()
    }

    pub fn value_type(&self) -> Option<&Type> {
        self.value_type.as_ref()
    }

    pub fn element_type(&self) -> &Type {
        &self.element_type
    }

    pub fn element(&self) -> &Expression {
        &self.element
    }

    pub fn key_name(&self) -> &str {
        &self.key_name
    }

    pub fn value_name(&self) -> &str {
        &self.value_name
    }

    pub fn map(&self) -> &Expression {
        &self.map
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
