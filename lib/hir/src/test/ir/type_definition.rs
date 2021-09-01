use crate::{ir::*, test::position, types};

pub trait TypeDefinitionFake {
    fn fake(
        name: impl Into<String>,
        elements: Vec<types::RecordElement>,
        open: bool,
        public: bool,
        external: bool,
    ) -> Self;
}

impl TypeDefinitionFake for TypeDefinition {
    fn fake(
        name: impl Into<String>,
        elements: Vec<types::RecordElement>,
        open: bool,
        public: bool,
        external: bool,
    ) -> Self {
        Self::new(name, "", elements, open, public, external, position())
    }
}
