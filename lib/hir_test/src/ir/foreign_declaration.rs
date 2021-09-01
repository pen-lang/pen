use crate::position::position;
use hir::{ir::*, types::Type};

pub trait FakeForeignDeclaration {
    fn fake(name: impl Into<String>, type_: impl Into<Type>) -> Self;
}

impl FakeForeignDeclaration for ForeignDeclaration {
    fn fake(name: impl Into<String>, type_: impl Into<Type>) -> Self {
        Self::new(name, "", CallingConvention::C, type_, position())
    }
}
