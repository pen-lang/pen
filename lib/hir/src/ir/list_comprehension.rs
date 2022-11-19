use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehension {
    type_: Type,
    iteratee_type: Option<Type>,
    element: Rc<Expression>,
    primary_name: String,
    secondary_name: Option<String>,
    iteratee: Rc<Expression>,
    position: Position,
}

impl ListComprehension {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        type_: impl Into<Type>,
        iteratee_type: Option<Type>,
        element: impl Into<Expression>,
        primary_name: impl Into<String>,
        secondary_name: Option<String>,
        iteratee: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            type_: type_.into(),
            iteratee_type,
            element: element.into().into(),
            primary_name: primary_name.into(),
            secondary_name,
            iteratee: iteratee.into().into(),
            position,
        }
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn iteratee_type(&self) -> Option<&Type> {
        self.iteratee_type.as_ref()
    }

    pub fn element(&self) -> &Expression {
        &self.element
    }

    pub fn primary_name(&self) -> &str {
        &self.primary_name
    }

    pub fn secondary_name(&self) -> Option<&str> {
        self.secondary_name.as_deref()
    }

    pub fn iteratee(&self) -> &Expression {
        &self.iteratee
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
