use super::{ForeignDefinitionConfiguration, lambda::Lambda};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct FunctionDefinition {
    name: String,
    original_name: String,
    lambda: Lambda,
    foreign_definition_configuration: Option<ForeignDefinitionConfiguration>,
    public: bool,
    position: Position,
}

impl FunctionDefinition {
    pub fn new(
        name: impl Into<String>,
        original_name: impl Into<String>,
        lambda: Lambda,
        foreign_definition_configuration: Option<ForeignDefinitionConfiguration>,
        public: bool,
        position: Position,
    ) -> Self {
        Self {
            name: name.into(),
            original_name: original_name.into(),
            lambda,
            foreign_definition_configuration,
            public,
            position,
        }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn original_name(&self) -> &str {
        &self.original_name
    }

    pub fn lambda(&self) -> &Lambda {
        &self.lambda
    }

    pub fn foreign_definition_configuration(&self) -> Option<&ForeignDefinitionConfiguration> {
        self.foreign_definition_configuration.as_ref()
    }

    pub fn is_public(&self) -> bool {
        self.public
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
