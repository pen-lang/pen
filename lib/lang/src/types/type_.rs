use super::{
    any::Any, boolean::Boolean, byte_string::ByteString, function::Function, list::List,
    none::None, number::Number, record::Record, reference::Reference, union::Union,
};
use crate::position::Position;
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
    Union(Union),
}

impl Type {
    pub fn position(&self) -> &Position {
        match self {
            Self::Any(any) => any.position(),
            Self::Boolean(boolean) => boolean.position(),
            Self::Function(function) => function.position(),
            Self::List(list) => list.position(),
            Self::None(none) => none.position(),
            Self::Number(number) => number.position(),
            Self::Record(record) => record.position(),
            Self::Reference(reference) => reference.position(),
            Self::String(string) => string.position(),
            Self::Union(union) => union.position(),
        }
    }

    pub fn into_function(self) -> Option<Function> {
        match self {
            Type::Function(function) => Some(function),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<List> {
        match self {
            Type::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn into_record(self) -> Option<Record> {
        match self {
            Type::Record(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Type::Function(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Type::List(_))
    }

    pub fn is_record(&self) -> bool {
        matches!(self, Type::Record(_))
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

impl From<Union> for Type {
    fn from(union: Union) -> Self {
        Self::Union(union)
    }
}
