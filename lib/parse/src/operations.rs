use ast::{analysis::operator_priority, *};
use position::Position;

#[derive(Clone, Debug)]
pub enum SuffixOperator {
    Call(Vec<Expression>, Position),
    RecordField(String, Position),
    Try(Position),
}

pub fn reduce_operations(
    lhs: impl Into<Expression>,
    pairs: &[(BinaryOperator, Expression, Position)],
) -> Expression {
    match pairs {
        [] => lhs.into(),
        [(operator, rhs, position)] => {
            BinaryOperation::new(*operator, lhs, rhs.clone(), position.clone()).into()
        }
        [(operator, rhs, position), (next_operator, _, _), ..] => {
            if operator_priority(*operator) >= operator_priority(*next_operator) {
                reduce_operations(
                    BinaryOperation::new(*operator, lhs, rhs.clone(), position.clone()),
                    &pairs[1..],
                )
            } else {
                let pairs = &pairs[1..];
                let (head, tail) = pairs.split_at(
                    pairs
                        .iter()
                        .position(|pair: &(_, _, _)| {
                            operator_priority(pair.0) <= operator_priority(*operator)
                        })
                        .unwrap_or(pairs.len()),
                );

                reduce_operations(
                    BinaryOperation::new(
                        *operator,
                        lhs,
                        reduce_operations(rhs.clone(), head),
                        position.clone(),
                    ),
                    tail,
                )
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    #[test]
    fn reduce_no_operation() {
        let variable = Variable::new("x", Position::fake());

        assert_eq!(reduce_operations(variable.clone(), &[]), variable.into());
    }

    #[test]
    fn reduce_one_operation() {
        let variable = Variable::new("x", Position::fake());

        assert_eq!(
            reduce_operations(
                variable.clone(),
                &[(
                    BinaryOperator::And,
                    variable.clone().into(),
                    Position::fake()
                )]
            ),
            BinaryOperation::new(
                BinaryOperator::And,
                variable.clone(),
                variable,
                Position::fake()
            )
            .into(),
        );
    }

    #[test]
    fn reduce_three_operations_with_low_priority_operator_in_middle() {
        let variable = Variable::new("x", Position::fake());

        assert_eq!(
            reduce_operations(
                variable.clone(),
                &[
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Or,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    )
                ]
            ),
            BinaryOperation::new(
                BinaryOperator::Or,
                BinaryOperation::new(
                    BinaryOperator::And,
                    variable.clone(),
                    variable.clone(),
                    Position::fake()
                ),
                BinaryOperation::new(
                    BinaryOperator::And,
                    variable.clone(),
                    variable,
                    Position::fake()
                ),
                Position::fake()
            )
            .into(),
        );
    }

    #[test]
    fn reduce_four_operations_with_low_priority_operator_in_middle() {
        let variable = Variable::new("x", Position::fake());

        assert_eq!(
            reduce_operations(
                variable.clone(),
                &[
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Or,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    )
                ]
            ),
            BinaryOperation::new(
                BinaryOperator::Or,
                BinaryOperation::new(
                    BinaryOperator::And,
                    BinaryOperation::new(
                        BinaryOperator::And,
                        variable.clone(),
                        variable.clone(),
                        Position::fake()
                    ),
                    variable.clone(),
                    Position::fake()
                ),
                BinaryOperation::new(
                    BinaryOperator::And,
                    variable.clone(),
                    variable,
                    Position::fake()
                ),
                Position::fake()
            )
            .into(),
        );
    }

    #[test]
    fn reduce_three_operations_with_high_priority_operator_in_middle() {
        let variable = Variable::new("x", Position::fake());

        assert_eq!(
            reduce_operations(
                variable.clone(),
                &[
                    (
                        BinaryOperator::Or,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Or,
                        variable.clone().into(),
                        Position::fake()
                    )
                ]
            ),
            BinaryOperation::new(
                BinaryOperator::Or,
                BinaryOperation::new(
                    BinaryOperator::Or,
                    variable.clone(),
                    BinaryOperation::new(
                        BinaryOperator::And,
                        variable.clone(),
                        variable.clone(),
                        Position::fake()
                    ),
                    Position::fake()
                ),
                variable,
                Position::fake()
            )
            .into(),
        );
    }

    #[test]
    fn reduce_operations_with_three_priorities() {
        let variable = Variable::new("x", Position::fake());

        assert_eq!(
            reduce_operations(
                variable.clone(),
                &[
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Equal,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Or,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    )
                ]
            ),
            BinaryOperation::new(
                BinaryOperator::Or,
                BinaryOperation::new(
                    BinaryOperator::And,
                    variable.clone(),
                    BinaryOperation::new(
                        BinaryOperator::Equal,
                        variable.clone(),
                        variable.clone(),
                        Position::fake()
                    ),
                    Position::fake()
                ),
                BinaryOperation::new(
                    BinaryOperator::And,
                    variable.clone(),
                    variable,
                    Position::fake()
                ),
                Position::fake()
            )
            .into(),
        );
    }

    #[test]
    fn reduce_operations_with_three_priorities_with_two_sequential_high_priority_operators() {
        let variable = Variable::new("x", Position::fake());

        assert_eq!(
            reduce_operations(
                variable.clone(),
                &[
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Equal,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Equal,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::Or,
                        variable.clone().into(),
                        Position::fake()
                    ),
                    (
                        BinaryOperator::And,
                        variable.clone().into(),
                        Position::fake()
                    )
                ]
            ),
            BinaryOperation::new(
                BinaryOperator::Or,
                BinaryOperation::new(
                    BinaryOperator::And,
                    variable.clone(),
                    BinaryOperation::new(
                        BinaryOperator::Equal,
                        BinaryOperation::new(
                            BinaryOperator::Equal,
                            variable.clone(),
                            variable.clone(),
                            Position::fake()
                        ),
                        variable.clone(),
                        Position::fake()
                    ),
                    Position::fake()
                ),
                BinaryOperation::new(
                    BinaryOperator::And,
                    variable.clone(),
                    variable,
                    Position::fake()
                ),
                Position::fake()
            )
            .into(),
        );
    }
}
