use super::{
    any::Any, boolean::Boolean, byte_string::ByteString, error::Error, function::Function,
    list::List, map::Map, none::None, number::Number, record::Record, reference::Reference,
    union::Union,
};
use position::Position;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
pub enum Type {
    Any(Any),
    Boolean(Boolean),
    Error(Error),
    Function(Function),
    List(List),
    Map(Map),
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
            Self::Error(error) => error.position(),
            Self::Function(function) => function.position(),
            Self::List(list) => list.position(),
            Self::Map(map) => map.position(),
            Self::None(none) => none.position(),
            Self::Number(number) => number.position(),
            Self::Record(record) => record.position(),
            Self::Reference(reference) => reference.position(),
            Self::String(string) => string.position(),
            Self::Union(union) => union.position(),
        }
    }

    pub fn set_position(self, position: Position) -> Self {
        match self {
            Self::Any(any) => any.set_position(position).into(),
            Self::Boolean(boolean) => boolean.set_position(position).into(),
            Self::Error(error) => error.set_position(position).into(),
            Self::Function(function) => function.set_position(position).into(),
            Self::List(list) => list.set_position(position).into(),
            Self::Map(map) => map.set_position(position).into(),
            Self::None(none) => none.set_position(position).into(),
            Self::Number(number) => number.set_position(position).into(),
            Self::Record(record) => record.set_position(position).into(),
            Self::Reference(reference) => reference.set_position(position).into(),
            Self::String(string) => string.set_position(position).into(),
            Self::Union(union) => union.set_position(position).into(),
        }
    }

    pub fn into_function(self) -> Option<Function> {
        match self {
            Self::Function(function) => Some(function),
            _ => None,
        }
    }

    pub fn into_list(self) -> Option<List> {
        match self {
            Self::List(list) => Some(list),
            _ => None,
        }
    }

    pub fn into_map(self) -> Option<Map> {
        match self {
            Self::Map(map) => Some(map),
            _ => None,
        }
    }

    pub fn into_record(self) -> Option<Record> {
        match self {
            Self::Record(record) => Some(record),
            _ => None,
        }
    }

    pub fn is_any(&self) -> bool {
        matches!(self, Self::Any(_))
    }

    pub fn is_function(&self) -> bool {
        matches!(self, Self::Function(_))
    }

    pub fn is_list(&self) -> bool {
        matches!(self, Self::List(_))
    }

    pub fn is_map(&self) -> bool {
        matches!(self, Self::Map(_))
    }

    pub fn is_record(&self) -> bool {
        matches!(self, Self::Record(_))
    }

    pub fn is_union(&self) -> bool {
        matches!(self, Self::Union(_))
    }

    pub fn is_variant(&self) -> bool {
        self.is_any() || self.is_union()
    }

    pub fn as_list(&self) -> Option<&List> {
        if let Self::List(type_) = self {
            Some(type_)
        } else {
            None
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

impl From<Error> for Type {
    fn from(error: Error) -> Self {
        Self::Error(error)
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

impl From<Map> for Type {
    fn from(map: Map) -> Self {
        Self::Map(map)
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
