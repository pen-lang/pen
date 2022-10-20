use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehension {
    type_: Type,
    element: Rc<Expression>,
    element_name: String,
    list: Rc<Expression>,
    position: Position,
}

impl ListComprehension {
    pub fn new(
        type_: impl Into<Type>,
        element: impl Into<Expression>,
        element_name: impl Into<String>,
        list: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            element: element.into().into(),
            element_name: element_name.into(),
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

    pub fn element_name(&self) -> &str {
        &self.element_name
    }

    pub fn list(&self) -> &Expression {
        &self.list
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
