use super::{
    BuiltInFunction, Lambda, Let, ListComprehension, Thunk, boolean::Boolean, call::Call, if_::If,
    if_list::IfList, if_map::IfMap, if_type::IfType, list::List, map::Map, none::None,
    number::Number, operation::Operation, record_construction::RecordConstruction,
    record_deconstruction::RecordDeconstruction, record_update::RecordUpdate, string::ByteString,
    type_coercion::TypeCoercion, variable::Variable,
};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Expression {
    Boolean(Boolean),
    BuiltInFunction(BuiltInFunction),
    Call(Call),
    If(If),
    IfList(IfList),
    IfMap(IfMap),
    IfType(IfType),
    Lambda(Lambda),
    Let(Let),
    List(List),
    ListComprehension(ListComprehension),
    Map(Map),
    None(None),
    Number(Number),
    Operation(Operation),
    RecordConstruction(RecordConstruction),
    RecordDeconstruction(RecordDeconstruction),
    RecordUpdate(RecordUpdate),
    String(ByteString),
    Thunk(Thunk),
    TypeCoercion(TypeCoercion),
    Variable(Variable),
}

impl Expression {
    pub fn position(&self) -> &Position {
        match self {
            Self::Boolean(boolean) => boolean.position(),
            Self::BuiltInFunction(function) => function.position(),
            Self::Call(call) => call.position(),
            Self::If(if_) => if_.position(),
            Self::IfList(if_) => if_.position(),
            Self::IfMap(if_) => if_.position(),
            Self::IfType(if_) => if_.position(),
            Self::Lambda(lambda) => lambda.position(),
            Self::Let(let_) => let_.position(),
            Self::List(list) => list.position(),
            Self::ListComprehension(comprehension) => comprehension.position(),
            Self::Map(map) => map.position(),
            Self::None(none) => none.position(),
            Self::Number(number) => number.position(),
            Self::Operation(operation) => operation.position(),
            Self::RecordConstruction(construction) => construction.position(),
            Self::RecordDeconstruction(deconstruction) => deconstruction.position(),
            Self::RecordUpdate(record_update) => record_update.position(),
            Self::String(string) => string.position(),
            Self::Thunk(thunk) => thunk.position(),
            Self::TypeCoercion(coercion) => coercion.position(),
            Self::Variable(variable) => variable.position(),
        }
    }
}

impl From<Boolean> for Expression {
    fn from(boolean: Boolean) -> Self {
        Self::Boolean(boolean)
    }
}

impl From<BuiltInFunction> for Expression {
    fn from(function: BuiltInFunction) -> Self {
        Self::BuiltInFunction(function)
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

impl From<RecordConstruction> for Expression {
    fn from(construction: RecordConstruction) -> Self {
        Self::RecordConstruction(construction)
    }
}

impl From<RecordDeconstruction> for Expression {
    fn from(deconstruction: RecordDeconstruction) -> Self {
        Self::RecordDeconstruction(deconstruction)
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

impl From<IfList> for Expression {
    fn from(if_: IfList) -> Self {
        Self::IfList(if_)
    }
}

impl From<IfMap> for Expression {
    fn from(if_: IfMap) -> Self {
        Self::IfMap(if_)
    }
}

impl From<Lambda> for Expression {
    fn from(lambda: Lambda) -> Self {
        Self::Lambda(lambda)
    }
}

impl From<Let> for Expression {
    fn from(let_: Let) -> Self {
        Self::Let(let_)
    }
}

impl From<List> for Expression {
    fn from(list: List) -> Self {
        Self::List(list)
    }
}

impl From<ListComprehension> for Expression {
    fn from(comprehension: ListComprehension) -> Self {
        Self::ListComprehension(comprehension)
    }
}

impl From<Map> for Expression {
    fn from(map: Map) -> Self {
        Self::Map(map)
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

impl From<Thunk> for Expression {
    fn from(thunk: Thunk) -> Self {
        Self::Thunk(thunk)
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
