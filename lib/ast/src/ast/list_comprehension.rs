use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehension {
    type_: Type,
    element: Rc<Expression>,
    primary_element_name: String,
    secondary_element_name: Option<String>,
    list: Rc<Expression>,
    position: Position,
}

impl ListComprehension {
    pub fn new(
        type_: impl Into<Type>,
        element: impl Into<Expression>,
        primary_name: impl Into<String>,
        secondary_name: Option<String>,
        list: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            element: element.into().into(),
            primary_element_name: primary_name.into(),
            secondary_element_name: secondary_name,
            list: list.into().into(),
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn element(&self) -> &Expression {
        &self.element
    }

    pub fn primary_name(&self) -> &str {
        &self.primary_element_name
    }

    pub fn secondary_name(&self) -> Option<&str> {
        self.secondary_element_name.as_deref()
    }

    pub fn iteratee(&self) -> &Expression {
        &self.list
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
