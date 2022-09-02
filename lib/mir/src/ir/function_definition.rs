use super::{argument::Argument, expression::Expression};
use crate::types::{self, Type};

// TODO Consider splitting function and thunk definitions.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    name: String,
    // Environment is inferred on module creation and this field is used just
    // as its cache.  So it must be safe to clone definitions inside a
    // module and use it on creation of another module.
    environment: Vec<Argument>,
    arguments: Vec<Argument>,
    result_type: Type,
    body: Expression,
    type_: types::Function,
    thunk: bool,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        result_type: impl Into<Type>,
        body: impl Into<Expression>,
    ) -> Self {
        Self::with_options(name, vec![], arguments, result_type, body, false)
    }

    pub fn thunk(
        name: impl Into<String>,
        result_type: impl Into<Type>,
        body: impl Into<Expression>,
    ) -> Self {
        Self::with_options(name, vec![], vec![], result_type, body, true)
    }

    pub(crate) fn with_options(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        result_type: impl Into<Type>,
        body: impl Into<Expression>,
        is_thunk: bool,
    ) -> Self {
        let result_type = result_type.into();

        Self {
            type_: types::Function::new(
                arguments
                    .iter()
                    .map(|argument| argument.type_())
                    .cloned()
                    .collect(),
                result_type.clone(),
            ),
            name: name.into(),
            environment,
            arguments,
            result_type,
            body: body.into(),
            thunk: is_thunk,
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
        self.thunk
    }
}
