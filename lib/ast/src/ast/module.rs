use super::{
    ForeignImport, Import, function_definition::FunctionDefinition, type_definition::TypeDefinition,
};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    imports: Vec<Import>,
    foreign_imports: Vec<ForeignImport>,
    type_definitions: Vec<TypeDefinition>,
    function_definitions: Vec<FunctionDefinition>,
    position: Position,
}

impl Module {
    pub fn new(
        imports: Vec<Import>,
        foreign_imports: Vec<ForeignImport>,
        type_definitions: Vec<TypeDefinition>,
        function_definitions: Vec<FunctionDefinition>,
        position: Position,
    ) -> Self {
        Self {
            imports,
            foreign_imports,
            type_definitions,
            function_definitions,
            position,
        }
    }

    pub fn imports(&self) -> &[Import] {
        &self.imports
    }

    pub fn foreign_imports(&self) -> &[ForeignImport] {
        &self.foreign_imports
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn function_definitions(&self) -> &[FunctionDefinition] {
        &self.function_definitions
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
