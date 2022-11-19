use super::expression::Expression;
use crate::types::Type;
use position::Position;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct ListComprehension {
    output_type: Type,
    primary_input_type: Option<Type>,
    secondary_input_type: Option<Type>,
    element: Rc<Expression>,
    primary_name: String,
    secondary_name: Option<String>,
    iteratee: Rc<Expression>,
    position: Position,
}

impl ListComprehension {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        output_type: impl Into<Type>,
        primary_input_type: Option<Type>,
        secondary_input_type: Option<Type>,
        element: impl Into<Expression>,
        primary_name: impl Into<String>,
        secondary_name: Option<String>,
        iteratee: impl Into<Expression>,
        position: Position,
    ) -> Self {
        Self {
            output_type: output_type.into(),
            primary_input_type,
            secondary_input_type,
            element: element.into().into(),
            primary_name: primary_name.into(),
            secondary_name,
            iteratee: iteratee.into().into(),
            position,
        }
    }

    pub fn output_type(&self) -> &Type {
        &self.output_type
    }

    pub fn primary_input_type(&self) -> Option<&Type> {
        self.primary_input_type.as_ref()
    }

    pub fn secondary_input_type(&self) -> Option<&Type> {
        self.secondary_input_type.as_ref()
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
