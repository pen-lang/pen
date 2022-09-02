use super::FunctionDefinition;

#[derive(Clone, Debug, PartialEq)]
pub struct GlobalFunctionDefinition {
    definition: FunctionDefinition,
    public: bool,
}

impl GlobalFunctionDefinition {
    pub fn new(definition: FunctionDefinition, public: bool) -> Self {
        Self { definition, public }
    }

    pub fn definition(&self) -> &FunctionDefinition {
        &self.definition
    }

    pub fn is_public(&self) -> bool {
        self.public
    }
}
