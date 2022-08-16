use super::{argument::Argument, expression::Expression};
use crate::types::{self, Type};

// TODO Consider splitting function and thunk definitions.
// TODO Consider splitting global and local function definitions.
#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    name: String,
    // Environment is inferred on module creation and this field is used just
    // as its cache.  So it must be safe to clone definitions inside a
    // module and use it on creation of another module.
    environment: Vec<Argument>,
    arguments: Vec<Argument>,
    body: Expression,
    result_type: Type,
    type_: types::Function,
    public: bool,
    thunk: bool,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
        public: bool,
    ) -> Self {
        Self::with_options(name, vec![], arguments, body, result_type, public, false)
    }

    pub fn thunk(
        name: impl Into<String>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
        public: bool,
    ) -> Self {
        Self::with_options(name, vec![], vec![], body, result_type, public, true)
    }

    pub(crate) fn with_options(
        name: impl Into<String>,
        environment: Vec<Argument>,
        arguments: Vec<Argument>,
        body: impl Into<Expression>,
        result_type: impl Into<Type>,
        public: bool,
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
            body: body.into(),
            result_type,
            public,
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

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn is_thunk(&self) -> bool {
        self.thunk
    }
}
