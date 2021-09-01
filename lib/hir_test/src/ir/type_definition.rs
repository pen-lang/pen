use crate::position::position;
use hir::{ir::*, types};

pub trait FakeTypeDefinition {
    fn fake(
        name: impl Into<String>,
        elements: Vec<types::RecordElement>,
        open: bool,
        public: bool,
        external: bool,
    ) -> Self;
}

impl FakeTypeDefinition for TypeDefinition {
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
