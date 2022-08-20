use super::{
    foreign_declaration::ForeignDeclaration, foreign_definition::ForeignDefinition,
    function_declaration::FunctionDeclaration, type_definition::TypeDefinition,
    GlobalFunctionDefinition,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    foreign_declarations: Vec<ForeignDeclaration>,
    foreign_definitions: Vec<ForeignDefinition>,
    function_declarations: Vec<FunctionDeclaration>,
    function_definitions: Vec<GlobalFunctionDefinition>,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        foreign_declarations: Vec<ForeignDeclaration>,
        foreign_definitions: Vec<ForeignDefinition>,
        function_declarations: Vec<FunctionDeclaration>,
        function_definitions: Vec<GlobalFunctionDefinition>,
    ) -> Self {
        Self {
            type_definitions,
            foreign_declarations,
            foreign_definitions,
            function_declarations,
            function_definitions,
        }
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn foreign_declarations(&self) -> &[ForeignDeclaration] {
        &self.foreign_declarations
    }

    pub fn foreign_definitions(&self) -> &[ForeignDefinition] {
        &self.foreign_definitions
    }

    pub fn function_declarations(&self) -> &[FunctionDeclaration] {
        &self.function_declarations
    }

    pub fn function_definitions(&self) -> &[GlobalFunctionDefinition] {
        &self.function_definitions
    }
}
