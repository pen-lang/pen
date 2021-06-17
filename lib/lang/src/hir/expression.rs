use super::{
    boolean::Boolean, call::Call, if_::If, if_list::IfList, if_type::IfType, list::List,
    none::None, number::Number, operation::Operation, record_construction::RecordConstruction,
    record_element::RecordElement, record_update::RecordUpdate, string::ByteString,
    type_coercion::TypeCoercion, variable::Variable, Lambda,
};
use crate::position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Boolean(Boolean),
    Call(Call),
    If(If),
    IfList(IfList),
    IfType(IfType),
    Lambda(Lambda),
    List(List),
    None(None),
    Number(Number),
    Operation(Operation),
    RecordConstruction(RecordConstruction),
    RecordElement(RecordElement),
    RecordUpdate(RecordUpdate),
    String(ByteString),
    TypeCoercion(TypeCoercion),
    Variable(Variable),
}

impl Expression {
    pub fn position(&self) -> &Position {
        match self {
            Self::Boolean(boolean) => boolean.position(),
            Self::Call(call) => call.position(),
            Self::If(if_) => if_.position(),
            Self::IfList(if_) => if_.position(),
            Self::IfType(if_) => if_.position(),
            Self::Lambda(lambda) => lambda.position(),
            Self::List(list) => list.position(),
            Self::None(none) => none.position(),
            Self::Number(number) => number.position(),
            Self::Operation(operation) => operation.position(),
            Self::RecordConstruction(construction) => construction.position(),
            Self::RecordElement(element) => element.position(),
            Self::RecordUpdate(record_update) => record_update.position(),
            Self::String(string) => string.position(),
            Self::TypeCoercion(coercion) => coercion.position(),
            Self::Variable(variable) => variable.position(),
        }
    }
}

impl From<Call> for Expression {
    fn from(call: Call) -> Self {
        Self::Call(call)
    }
}

impl From<Boolean> for Expression {
    fn from(boolean: Boolean) -> Self {
        Self::Boolean(boolean)
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

impl From<RecordConstruction> for Expression {
    fn from(construction: RecordConstruction) -> Self {
        Self::RecordConstruction(construction)
    }
}

impl From<RecordElement> for Expression {
    fn from(element: RecordElement) -> Self {
        Self::RecordElement(element)
    }
}

impl From<RecordUpdate> for Expression {
    fn from(record_update: RecordUpdate) -> Self {
        Self::RecordUpdate(record_update)
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

impl<T: Into<Operation>> From<T> for Expression {
    fn from(operation: T) -> Self {
        Self::Operation(operation.into())
    }
}

impl From<TypeCoercion> for Expression {
    fn from(coercion: TypeCoercion) -> Self {
        Self::TypeCoercion(coercion)
    }
}

impl From<Variable> for Expression {
    fn from(variable: Variable) -> Self {
        Self::Variable(variable)
    }
}
