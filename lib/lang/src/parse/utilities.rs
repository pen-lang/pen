use crate::{ast::*, position::Position};

#[derive(Clone, Copy, Debug, PartialEq)]
pub enum ParsedOperator {
    Add,
    Subtract,
    Multiply,
    Divide,
    Equal,
    NotEqual,
    LessThan,
    LessThanOrEqual,
    GreaterThan,
    GreaterThanOrEqual,
    And,
    Or,
}

pub fn reduce_operations(
    lhs: Expression,
    pairs: &[(ParsedOperator, Expression, Position)],
) -> Expression {
    match pairs {
        [] => lhs,
        [(operator, rhs, position)] => {
            create_operation(*operator, lhs, rhs.clone(), position).into()
        }
        [(operator, rhs, position), (next_operator, _, _), ..] => {
            if operator_priority(*operator) >= operator_priority(*next_operator) {
                reduce_operations(
                    create_operation(*operator, lhs, rhs.clone(), position).into(),
                    &pairs[1..],
                )
            } else {
                create_operation(
                    *operator,
                    lhs,
                    reduce_operations(rhs.clone(), &pairs[1..]),
                    position,
                )
                .into()
            }
        }
    }
}

fn create_operation(
    operator: ParsedOperator,
    lhs: impl Into<Expression>,
    rhs: impl Into<Expression>,
    position: &Position,
) -> Operation {
    match operator {
        ParsedOperator::Or => {
            BooleanOperation::new(BooleanOperator::Or, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::And => {
            BooleanOperation::new(BooleanOperator::And, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::Equal => {
            EqualityOperation::new(EqualityOperator::Equal, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::NotEqual => {
            EqualityOperation::new(EqualityOperator::NotEqual, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::Add => {
            ArithmeticOperation::new(ArithmeticOperator::Add, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::Subtract => {
            ArithmeticOperation::new(ArithmeticOperator::Subtract, lhs, rhs, position.clone())
                .into()
        }
        ParsedOperator::Multiply => {
            ArithmeticOperation::new(ArithmeticOperator::Multiply, lhs, rhs, position.clone())
                .into()
        }
        ParsedOperator::Divide => {
            ArithmeticOperation::new(ArithmeticOperator::Divide, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::LessThan => {
            OrderOperation::new(OrderOperator::LessThan, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::LessThanOrEqual => {
            OrderOperation::new(OrderOperator::LessThanOrEqual, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::GreaterThan => {
            OrderOperation::new(OrderOperator::GreaterThan, lhs, rhs, position.clone()).into()
        }
        ParsedOperator::GreaterThanOrEqual => OrderOperation::new(
            OrderOperator::GreaterThanOrEqual,
            lhs,
            rhs,
            position.clone(),
        )
        .into(),
    }
}

fn operator_priority(operator: ParsedOperator) -> usize {
    match operator {
        ParsedOperator::Or => 1,
        ParsedOperator::And => 2,
        ParsedOperator::Equal
        | ParsedOperator::NotEqual
        | ParsedOperator::LessThan
        | ParsedOperator::LessThanOrEqual
        | ParsedOperator::GreaterThan
        | ParsedOperator::GreaterThanOrEqual => 3,
        ParsedOperator::Add | ParsedOperator::Subtract => 4,
        ParsedOperator::Multiply | ParsedOperator::Divide => 5,
    }
}
