use super::{
    any::Any, boolean::Boolean, byte_string::ByteString, function::Function, list::List,
    none::None, number::Number, record::Record, reference::Reference, union::Union,
    unknown::Unknown, variable::Variable,
};
use crate::debug::SourceInformation;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Type {
    Any(Any),
    Boolean(Boolean),
    Function(Function),
    List(List),
    None(None),
    Number(Number),
    Record(Record),
    Reference(Reference),
    String(ByteString),
    Unknown(Unknown),
    Union(Union),
    Variable(Variable),
}

impl Type {
    pub fn source_information(&self) -> &SourceInformation {
        match self {
            Self::Any(any) => any.source_information(),
            Self::Boolean(boolean) => boolean.source_information(),
            Self::Function(function) => function.source_information(),
            Self::List(list) => list.source_information(),
            Self::None(none) => none.source_information(),
            Self::Number(number) => number.source_information(),
            Self::Record(record) => record.source_information(),
            Self::Reference(reference) => reference.source_information(),
            Self::String(string) => string.source_information(),
            Self::Unknown(unknown) => unknown.source_information(),
            Self::Union(union) => union.source_information(),
            Self::Variable(variable) => variable.source_information(),
        }
    }
}

impl From<Any> for Type {
    fn from(any: Any) -> Self {
        Self::Any(any)
    }
}

impl From<Boolean> for Type {
    fn from(boolean: Boolean) -> Self {
        Self::Boolean(boolean)
    }
}

impl From<ByteString> for Type {
    fn from(string: ByteString) -> Self {
        Self::String(string)
    }
}

impl From<Function> for Type {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

impl From<List> for Type {
    fn from(list: List) -> Self {
        Self::List(list)
    }
}

impl From<None> for Type {
    fn from(none: None) -> Self {
        Self::None(none)
    }
}

impl From<Number> for Type {
    fn from(number: Number) -> Self {
        Self::Number(number)
    }
}

impl From<Record> for Type {
    fn from(record: Record) -> Self {
        Self::Record(record)
    }
}

impl From<Reference> for Type {
    fn from(reference: Reference) -> Self {
        Self::Reference(reference)
    }
}

impl From<Unknown> for Type {
    fn from(unknown: Unknown) -> Self {
        Self::Unknown(unknown)
    }
}

impl From<Union> for Type {
    fn from(union: Union) -> Self {
        Self::Union(union)
    }
}

impl From<Variable> for Type {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}
