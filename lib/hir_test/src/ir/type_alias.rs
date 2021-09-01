use crate::position::position;
use hir::{ir::*, types::Type};

pub trait FakeTypeAlias {
    fn fake(name: impl Into<String>, type_: impl Into<Type>, public: bool, external: bool) -> Self;
}

impl FakeTypeAlias for TypeAlias {
    fn fake(name: impl Into<String>, type_: impl Into<Type>, public: bool, external: bool) -> Self {
        Self::new(name, "", type_, public, external, position())
    }
}
