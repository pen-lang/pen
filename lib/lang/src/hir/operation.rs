use super::{
    arithmetic_operation::ArithmeticOperation, boolean_operation::BooleanOperation,
    equality_operation::EqualityOperation, order_operation::OrderOperation,
};
use crate::position::Position;

#[derive(Clone, Debug, PartialEq)]
pub enum Operation {
    Arithmetic(ArithmeticOperation),
    Boolean(BooleanOperation),
    Equality(EqualityOperation),
    Order(OrderOperation),
}

impl Operation {
    pub fn position(&self) -> &Position {
        match self {
            Self::Arithmetic(operation) => operation.position(),
            Self::Boolean(operation) => operation.position(),
            Self::Equality(operation) => operation.position(),
            Self::Order(operation) => operation.position(),
        }
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

impl From<OrderOperation> for Operation {
    fn from(operation: OrderOperation) -> Self {
        Self::Order(operation)
    }
}
