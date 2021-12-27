use super::{
    arithmetic_operation::ArithmeticOperation, boolean_operation::BooleanOperation,
    equality_operation::EqualityOperation, not_operation::NotOperation,
    order_operation::OrderOperation, try_operation::TryOperation, AsyncOperation,
};
use position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Arithmetic(ArithmeticOperation),
    Async(AsyncOperation),
    Boolean(BooleanOperation),
    Equality(EqualityOperation),
    Not(NotOperation),
    Order(OrderOperation),
    Try(TryOperation),
}

impl Operation {
    pub fn position(&self) -> &Position {
        match self {
            Self::Arithmetic(operation) => operation.position(),
            Self::Async(operation) => operation.position(),
            Self::Boolean(operation) => operation.position(),
            Self::Equality(operation) => operation.position(),
            Self::Not(operation) => operation.position(),
            Self::Order(operation) => operation.position(),
            Self::Try(operation) => operation.position(),
        }
    }
}

impl From<ArithmeticOperation> for Operation {
    fn from(operation: ArithmeticOperation) -> Self {
        Self::Arithmetic(operation)
    }
}

impl From<AsyncOperation> for Operation {
    fn from(operation: AsyncOperation) -> Self {
        Self::Async(operation)
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
