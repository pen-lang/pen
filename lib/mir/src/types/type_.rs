use super::{function::Function, record::Record};

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub enum Type {
    Boolean,
    ByteString,
    Function(Function),
    Number,
    Record(Record),
    Variant,
}

impl Type {
    pub fn into_function(self) -> Option<Function> {
        match self {
            Self::Function(function) => Some(function),
            _ => None,
        }
    }

    pub fn into_record(self) -> Option<Record> {
        match self {
            Self::Record(record) => Some(record),
            _ => None,
        }
    }
}

impl From<Function> for Type {
    fn from(function: Function) -> Self {
        Self::Function(function)
    }
}

impl From<Record> for Type {
    fn from(record: Record) -> Self {
        Self::Record(record)
    }
}
