use super::{
    boolean::Boolean, call::Call, if_::If, if_list::IfList, if_type::IfType, list::List,
    none::None, number::Number, record::Record, string::ByteString, variable::Variable,
    BinaryOperation, Lambda, RecordDeconstruction, UnaryOperation,
};
use crate::SpawnOperation;
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    SpawnOperation(AsyncOperation),
    BinaryOperation(BinaryOperation),
    Boolean(Boolean),
    Call(Call),
    If(If),
    IfList(IfList),
    IfType(IfType),
    Lambda(Lambda),
    List(List),
    None(None),
    Number(Number),
    Record(Record),
    RecordDeconstruction(RecordDeconstruction),
    String(ByteString),
    UnaryOperation(UnaryOperation),
    Variable(Variable),
}

impl Expression {
    pub fn position(&self) -> &Position {
        match self {
            Self::SpawnOperation(operation) => operation.position(),
            Self::BinaryOperation(operation) => operation.position(),
            Self::Boolean(boolean) => boolean.position(),
            Self::Call(call) => call.position(),
            Self::If(if_) => if_.position(),
            Self::IfList(if_) => if_.position(),
            Self::IfType(if_) => if_.position(),
            Self::Lambda(lambda) => lambda.position(),
            Self::List(list) => list.position(),
            Self::None(none) => none.position(),
            Self::Number(number) => number.position(),
            Self::Record(record) => record.position(),
            Self::RecordDeconstruction(operation) => operation.position(),
            Self::String(string) => string.position(),
            Self::UnaryOperation(operation) => operation.position(),
            Self::Variable(variable) => variable.position(),
        }
    }
}

impl From<SpawnOperation> for Expression {
    fn from(operation: SpawnOperation) -> Self {
        Self::SpawnOperation(operation)
    }
}

impl From<BinaryOperation> for Expression {
    fn from(operation: BinaryOperation) -> Self {
        Self::BinaryOperation(operation)
    }
}

impl From<Boolean> for Expression {
    fn from(boolean: Boolean) -> Self {
        Self::Boolean(boolean)
    }
}

impl From<Call> for Expression {
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}

impl From<IfType> for Expression {
    fn from(if_: IfType) -> Self {
        Self::IfType(if_)
    }
}

impl From<ByteString> for Expression {
    fn from(string: ByteString) -> Self {
        Self::String(string)
    }
}

impl From<If> for Expression {
    fn from(if_: If) -> Self {
        Self::If(if_)
    }
}

impl From<Lambda> for Expression {
    fn from(lambda: Lambda) -> Self {
        Self::Lambda(lambda)
    }
}

impl From<List> for Expression {
    fn from(list: List) -> Self {
        Self::List(list)
    }
}

impl From<IfList> for Expression {
    fn from(if_: IfList) -> Self {
        Self::IfList(if_)
    }
}

impl From<None> for Expression {
    fn from(none: None) -> Self {
        Self::None(none)
    }
}

impl From<Number> for Expression {
    fn from(number: Number) -> Self {
        Self::Number(number)
    }
}

impl From<Record> for Expression {
    fn from(record: Record) -> Self {
        Self::Record(record)
    }
}

impl From<RecordDeconstruction> for Expression {
    fn from(operation: RecordDeconstruction) -> Self {
        Self::RecordDeconstruction(operation)
    }
}

impl From<UnaryOperation> for Expression {
    fn from(operation: UnaryOperation) -> Self {
        Self::UnaryOperation(operation)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}
