use super::expression::Expression;
use crate::types::Type;
use std::rc::Rc;

#[derive(Clone, Debug, PartialEq)]
pub struct Let(Arc<LetInner>);

#[derive(Clone, Debug, PartialEq)]
struct LetInner {
    name: String,
    type_: Type,
    bound_expression: Expression,
    expression: Expression,
}

impl Let {
    pub fn new(
        name: impl Into<String>,
        type_: impl Into<Type>,
        bound_expression: impl Into<Expression>,
        expression: impl Into<Expression>,
    ) -> Self {
        Self(
            LetInner {
                name: name.into(),
                type_: type_.into(),
                bound_expression: bound_expression.into(),
                expression: expression.into(),
            }
            .into(),
        )
    }

    pub fn name(&self) -> &str {
        &self.0.name
    }

    pub fn type_(&self) -> &Type {
        &self.0.type_
    }

    pub fn bound_expression(&self) -> &Expression {
        &self.0.bound_expression
    }

    pub fn expression(&self) -> &Expression {
        &self.0.expression
    }
}
