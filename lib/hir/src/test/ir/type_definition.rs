use crate::{ir::*, types};
use position::{Position, test::PositionFake};

pub trait TypeDefinitionFake {
    fn fake(
        name: impl Into<String>,
        fields: Vec<types::RecordField>,
        open: bool,
        public: bool,
        external: bool,
    ) -> Self;
}

impl TypeDefinitionFake for TypeDefinition {
    fn fake(
        name: impl Into<String>,
        fields: Vec<types::RecordField>,
        open: bool,
        public: bool,
        external: bool,
    ) -> Self {
        let name = name.into();

        Self::new(
            name.clone(),
            name,
            fields,
            open,
            public,
            external,
            Position::fake(),
        )
    }
}
