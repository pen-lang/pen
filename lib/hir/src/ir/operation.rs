use super::{
    AdditionOperation, arithmetic_operation::ArithmeticOperation,
    boolean_operation::BooleanOperation, equality_operation::EqualityOperation,
    not_operation::NotOperation, order_operation::OrderOperation, try_operation::TryOperation,
};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Addition(AdditionOperation),
    Arithmetic(ArithmeticOperation),
    Boolean(BooleanOperation),
    Equality(EqualityOperation),
    Not(NotOperation),
    Order(OrderOperation),
    Try(TryOperation),
}

impl Operation {
    pub fn position(&self) -> &Position {
        match self {
            Self::Addition(operation) => operation.position(),
            Self::Arithmetic(operation) => operation.position(),
            Self::Boolean(operation) => operation.position(),
            Self::Equality(operation) => operation.position(),
            Self::Not(operation) => operation.position(),
            Self::Order(operation) => operation.position(),
            Self::Try(operation) => operation.position(),
        }
    }
}

impl From<AdditionOperation> for Operation {
    fn from(operation: AdditionOperation) -> Self {
        Self::Addition(operation)
    }
}

impl From<ArithmeticOperation> for Operation {
    fn from(operation: ArithmeticOperation) -> Self {
        Self::Arithmetic(operation)
    }
}

impl From<BooleanOperation> for Operation {
    fn from(operation: BooleanOperation) -> Self {
        Self::Boolean(operation)
    }
}

impl From<EqualityOperation> for Operation {
    fn from(operation: EqualityOperation) -> Self {
        Self::Equality(operation)
    }
}

impl From<NotOperation> for Operation {
    fn from(operation: NotOperation) -> Self {
        Self::Not(operation)
    }
}

impl From<OrderOperation> for Operation {
    fn from(operation: OrderOperation) -> Self {
        Self::Order(operation)
    }
}

impl From<TryOperation> for Operation {
    fn from(operation: TryOperation) -> Self {
        Self::Try(operation)
    }
}
