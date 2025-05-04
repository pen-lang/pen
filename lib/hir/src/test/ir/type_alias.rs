use crate::{ir::*, types::Type};
use position::{Position, test::PositionFake};

pub trait TypeAliasFake {
    fn fake(name: impl Into<String>, type_: impl Into<Type>, public: bool, external: bool) -> Self;
}

impl TypeAliasFake for TypeAlias {
    fn fake(name: impl Into<String>, type_: impl Into<Type>, public: bool, external: bool) -> Self {
        Self::new(name, "", type_, public, external, Position::fake())
    }
}
