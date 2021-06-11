use super::expression::Expression;
use crate::{position::*, types::Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    name: String,
    body: Expression,
    type_: Type,
    public: bool,
    position: Position,
}

impl Definition {
    pub fn new(
        name: impl Into<String>,
        body: impl Into<Expression>,
        type_: impl Into<Type>,
        public: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            body: body.into(),
            type_: type_.into(),
            public,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub fn type_(&self) -> &Type {
        &self.type_
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
