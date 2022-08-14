use super::{
    function::Function, list::List, map::Map, record::Record, reference::Reference, union::Union,
};
use position::Position;

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum Type {
    Function(Function),
    List(List),
    Map(Map),
    Record(Record),
    Reference(Reference),
    Union(Union),
}

impl Type {
    pub fn position(&self) -> &Position {
        match self {
            Self::Function(function) => function.position(),
            Self::List(list) => list.position(),
            Self::Map(map) => map.position(),
            Self::Record(record) => record.position(),
            Self::Reference(reference) => reference.position(),
            Self::Union(union) => union.position(),
        }
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
