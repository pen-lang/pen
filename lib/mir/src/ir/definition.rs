use super::{argument::Argument, expression::Expression};
use crate::types::{self, Type};

#[derive(Clone, Debug, PartialEq)]
pub struct Definition {
    name: String,
    // Environment is inferred on module creation and this field is used just
    // as its cache.  So it must be safe to clone definitions inside a
    // module and use it on creation of another module.
    environment: Vec<Argument>,
    arguments: Vec<Argument>,
    body: Expression,
    result_type: Type,
    type_: types::Function,
    is_thunk: bool,
}

impl Definition {
    pub fn new(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(name, vec![], arguments, body, result_type, false)
    }

    pub fn thunk(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(name, vec![], arguments, body, result_type, true)
    }

    #[cfg(test)]
    pub(crate) fn with_environment(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
    ) -> Self {
        Self::with_options(name, environment, arguments, body, result_type, false)
    }

    pub(crate) fn with_options(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type> + Clone,
        is_thunk: bool,
    ) -> Self {
        Self {
            type_: arguments.iter().rev().skip(1).fold(
                types::Function::new(
                    arguments.iter().last().unwrap().type_().clone(),
                    result_type.clone(),
                ),
                |result, argument| types::Function::new(argument.type_().clone(), result),
            ),
            name: name.into(),
            environment,
            arguments,
            body: body.into(),
            result_type: result_type.into(),
            is_thunk,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn environment(&self) -> &[Argument] {
        &self.environment
    }

    pub fn arguments(&self) -> &[Argument] {
        &self.arguments
    }

    pub fn body(&self) -> &Expression {
        &self.body
    }

    pub fn result_type(&self) -> &Type {
        &self.result_type
    }

    pub fn type_(&self) -> &types::Function {
        &self.type_
    }

    pub fn is_thunk(&self) -> bool {
        self.is_thunk
    }
}
