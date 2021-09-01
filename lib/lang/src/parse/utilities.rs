use crate::ast::*;
use position::Position;

#[derive(Clone, Debug)]
pub enum SuffixOperator {
    Call(Vec<Expression>),
    Element(String),
    Try,
}

pub fn reduce_operations(
    lhs: Expression,
    pairs: &[(BinaryOperator, Expression, Position)],
) -> Expression {
    match pairs {
        [] => lhs,
        [(operator, rhs, position)] => {
            BinaryOperation::new(*operator, lhs, rhs.clone(), position.clone()).into()
        }
        [(operator, rhs, position), (next_operator, _, _), ..] => {
            if operator_priority(*operator) >= operator_priority(*next_operator) {
                reduce_operations(
                    BinaryOperation::new(*operator, lhs, rhs.clone(), position.clone()).into(),
                    &pairs[1..],
                )
            } else {
                BinaryOperation::new(
                    *operator,
                    lhs,
                    reduce_operations(rhs.clone(), &pairs[1..]),
                    position.clone(),
                )
                .into()
            }
        }
    }
}

fn operator_priority(operator: BinaryOperator) -> usize {
    match operator {
        BinaryOperator::Or => 1,
        BinaryOperator::And => 2,
        BinaryOperator::Equal
        | BinaryOperator::NotEqual
        | BinaryOperator::LessThan
        | BinaryOperator::LessThanOrEqual
        | BinaryOperator::GreaterThan
        | BinaryOperator::GreaterThanOrEqual => 3,
        BinaryOperator::Add | BinaryOperator::Subtract => 4,
        BinaryOperator::Multiply | BinaryOperator::Divide => 5,
    }
}
