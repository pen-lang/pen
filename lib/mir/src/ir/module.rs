use super::{
    GlobalFunctionDefinition, TypeInformation, foreign_declaration::ForeignDeclaration,
    foreign_definition::ForeignDefinition, function_declaration::FunctionDeclaration,
    type_definition::TypeDefinition,
};

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    foreign_declarations: Vec<ForeignDeclaration>,
    foreign_definitions: Vec<ForeignDefinition>,
    function_declarations: Vec<FunctionDeclaration>,
    function_definitions: Vec<GlobalFunctionDefinition>,
    type_information: TypeInformation,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        foreign_declarations: Vec<ForeignDeclaration>,
        foreign_definitions: Vec<ForeignDefinition>,
        function_declarations: Vec<FunctionDeclaration>,
        function_definitions: Vec<GlobalFunctionDefinition>,
        type_information: TypeInformation,
    ) -> Self {
        Self {
            type_definitions,
            foreign_declarations,
            foreign_definitions,
            function_declarations,
            function_definitions,
            type_information,
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

    pub fn type_information(&self) -> &TypeInformation {
        &self.type_information
    }
}
