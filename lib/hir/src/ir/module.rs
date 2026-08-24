use super::{
    ForeignDeclaration, FunctionDeclaration, TypeAlias, function_definition::FunctionDefinition,
    type_definition::TypeDefinition,
};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub struct Module {
    type_definitions: Vec<TypeDefinition>,
    type_aliases: Vec<TypeAlias>,
    foreign_declarations: Vec<ForeignDeclaration>,
    function_declarations: Vec<FunctionDeclaration>,
    function_definitions: Vec<FunctionDefinition>,
    position: Position,
}

impl Module {
    pub fn new(
        type_definitions: Vec<TypeDefinition>,
        type_aliases: Vec<TypeAlias>,
        foreign_declarations: Vec<ForeignDeclaration>,
        function_declarations: Vec<FunctionDeclaration>,
        function_definitions: Vec<FunctionDefinition>,
        position: Position,
    ) -> Self {
        Self {
            type_definitions,
            type_aliases,
            foreign_declarations,
            function_declarations,
            function_definitions,
            position,
        }
    }

    pub fn type_definitions(&self) -> &[TypeDefinition] {
        &self.type_definitions
    }

    pub fn type_aliases(&self) -> &[TypeAlias] {
        &self.type_aliases
    }

    pub fn foreign_declarations(&self) -> &[ForeignDeclaration] {
        &self.foreign_declarations
    }

    pub fn function_declarations(&self) -> &[FunctionDeclaration] {
        &self.function_declarations
    }

    pub fn function_definitions(&self) -> &[FunctionDefinition] {
        &self.function_definitions
    }

    pub fn position(&self) -> &Position {
        &self.position
    }
}
