use crate::ir::*;

pub fn escapes(expression: &Expression, name: &str) -> bool {
    moves_expression(expression, name) || escapes_expression(expression, name)
}

fn escapes_expression(expression: &Expression, name: &str) -> bool {
    let escapes = |expression| escapes_expression(expression, name);
    let moves = |expression| moves_expression(expression, name);

    match expression {
        Expression::ArithmeticOperation(operation) => {
            escapes(operation.lhs()) || escapes(operation.rhs())
        }
        Expression::Case(case) => {
            escapes(case.argument())
                || case.alternatives().iter().any(|alternative| {
                    alternative.name() != name && escapes(alternative.expression())
                })
                || case
                    .default_alternative()
                    .map(|alternative| {
                        alternative.name() != name && escapes(alternative.expression())
                    })
                    .unwrap_or_default()
        }
        Expression::CloneVariables(clone) => escapes(clone.expression()),
        Expression::ComparisonOperation(operation) => {
            escapes(operation.lhs()) || escapes(operation.rhs())
        }
        Expression::DropVariables(drop) => escapes(drop.expression()),
        Expression::Call(call) => {
            call.arguments().iter().any(moves)
                || escapes(call.function())
                || call.arguments().iter().any(escapes)
        }
        Expression::If(if_) => {
            escapes(if_.condition()) || escapes(if_.then()) || escapes(if_.else_())
        }
        Expression::Let(let_) => {
            escapes(let_.bound_expression()) || let_.name() != name && escapes(let_.expression())
        }
        Expression::LetRecursive(let_) => {
            let_.definition().name() != name && escapes(let_.expression())
        }
        Expression::Synchronize(synchronize) => escapes(synchronize.expression()),
        Expression::Record(record) => record.fields().iter().any(escapes),
        Expression::RecordField(field) => escapes(field.record()),
        Expression::RecordUpdate(update) => {
            escapes(update.record())
                || update
                    .fields()
                    .iter()
                    .any(|field| escapes(field.expression()))
        }
        Expression::TryOperation(operation) => {
            escapes(operation.operand()) || escapes(operation.then())
        }
        Expression::Variant(variant) => escapes(variant.payload()),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_)
        | Expression::Variable(_) => false,
    }
}

// Check if a variable is moved shallowly.
fn moves_expression(expression: &Expression, name: &str) -> bool {
    let moves = |expression| moves_expression(expression, name);

    match expression {
        Expression::Case(case) => {
            case.alternatives()
                .iter()
                .any(|alternative| alternative.name() != name && moves(alternative.expression()))
                || case
                    .default_alternative()
                    .map(|alternative| {
                        alternative.name() != name && moves(alternative.expression())
                    })
                    .unwrap_or_default()
        }
        Expression::Call(call) => call.arguments().iter().any(moves),
        Expression::CloneVariables(clone) => moves(clone.expression()),
        Expression::DropVariables(drop) => moves(drop.expression()),
        Expression::If(if_) => moves(if_.then()) || moves(if_.else_()),
        Expression::Let(let_) => {
            if let_.name() == name {
                moves(let_.bound_expression()) && moves(let_.expression())
            } else {
                moves(let_.expression())
                    || moves(let_.bound_expression())
                        && moves_expression(let_.expression(), let_.name())
            }
        }
        // We consider variables to be moved whenever variables are in closure environment
        // even when the closures are not moved in later expressions
        // because we need to use normal data representation of those variables anyway
        // in lambda lifting if moved variables are closures.
        Expression::LetRecursive(let_) => {
            let_.definition()
                .environment()
                .iter()
                .any(|free_variable| free_variable.name() == name)
                || let_.definition().name() != name && moves(let_.expression())
        }
        Expression::Synchronize(synchronize) => moves(synchronize.expression()),
        Expression::Record(record) => record.fields().iter().any(moves),
        Expression::RecordField(field) => moves(field.record()),
        Expression::RecordUpdate(update) => {
            moves(update.record())
                || update
                    .fields()
                    .iter()
                    .any(|field| moves(field.expression()))
        }
        Expression::TryOperation(operation) => {
            moves(operation.operand()) || operation.name() != name && moves(operation.then())
        }
        Expression::Variable(variable) => variable.name() == name,
        Expression::Variant(variant) => moves(variant.payload()),
        Expression::ArithmeticOperation(_)
        | Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::ComparisonOperation(_)
        | Expression::None
        | Expression::Number(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{self, Type};

    #[test]
    fn escape_arithmetic_operation() {
        assert!(!escapes(
            &ArithmeticOperation::new(
                ArithmeticOperator::Add,
                Variable::new("x"),
                Expression::Number(0.0)
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_arithmetic_operation_with_call() {
        assert!(escapes(
            &ArithmeticOperation::new(
                ArithmeticOperator::Add,
                Call::new(
                    types::Function::new(vec![], Type::None),
                    Variable::new("f"),
                    vec![Variable::new("x").into()]
                ),
                Expression::Number(0.0)
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_call_function() {
        assert!(!escapes(
            &Call::new(
                types::Function::new(vec![], Type::None),
                Variable::new("f"),
                vec![]
            )
            .into(),
            "f",
        ));
    }

    #[test]
    fn escape_call_argument() {
        assert!(escapes(
            &Call::new(
                types::Function::new(vec![], Type::None),
                Variable::new("f"),
                vec![Variable::new("x").into()],
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_case_argument() {
        assert!(!escapes(
            &Case::new(Variable::new("x"), vec![], None).into(),
            "x",
        ));
    }

    #[test]
    fn escape_case_alternative() {
        assert!(escapes(
            &Case::new(
                Variant::new(Type::None, Expression::None),
                vec![Alternative::new(vec![Type::None], "y", Variable::new("x"))],
                None
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_shadowed_case_alternative() {
        assert!(!escapes(
            &Case::new(
                Variant::new(Type::None, Expression::None),
                vec![Alternative::new(vec![Type::None], "x", Variable::new("x"))],
                None
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_case_default_alternative() {
        assert!(escapes(
            &Case::new(
                Variant::new(Type::None, Expression::None),
                vec![],
                Some(DefaultAlternative::new("y", Variable::new("x")))
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_shadowed_case_default_alternative() {
        assert!(!escapes(
            &Case::new(
                Variant::new(Type::None, Expression::None),
                vec![],
                Some(DefaultAlternative::new("x", Variable::new("x")))
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_comparison_operation() {
        assert!(!escapes(
            &ComparisonOperation::new(
                ComparisonOperator::Equal,
                Variable::new("x"),
                Expression::Number(0.0)
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_if_condition() {
        assert!(!escapes(
            &If::new(Variable::new("x"), Expression::None, Expression::None).into(),
            "x",
        ));
    }

    #[test]
    fn escape_if_then() {
        assert!(escapes(
            &If::new(Expression::None, Variable::new("x"), Expression::None).into(),
            "x",
        ));
    }

    #[test]
    fn escape_if_else() {
        assert!(escapes(
            &If::new(Expression::None, Expression::None, Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn escape_let_expression() {
        assert!(escapes(
            &Let::new("y", Type::None, Expression::None, Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn escape_let_expression_with_call() {
        assert!(escapes(
            &Let::new(
                "y",
                Type::None,
                Variable::new("x"),
                Call::new(
                    types::Function::new(vec![], Type::None),
                    Variable::new("f"),
                    vec![Variable::new("y").into()]
                )
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_let_bound_expression_moved_in_expression() {
        assert!(escapes(
            &Let::new("y", Type::None, Variable::new("x"), Variable::new("y")).into(),
            "x",
        ));
    }

    #[test]
    fn escape_let_shadowed_bound_expression_moved_in_expression() {
        assert!(escapes(
            &Let::new("x", Type::None, Variable::new("x"), Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn escape_let_recursive() {
        assert!(escapes(
            &LetRecursive::new(
                FunctionDefinition::new("y", vec![], Type::None, Expression::None),
                Variable::new("x")
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_let_recursive_with_shadowed_variable() {
        assert!(!escapes(
            &LetRecursive::new(
                FunctionDefinition::new("x", vec![], Type::None, Expression::None),
                Variable::new("x")
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_let_recursive_with_environment() {
        assert!(escapes(
            &LetRecursive::new(
                FunctionDefinition::with_options(
                    "y",
                    vec![Argument::new("x", Type::None)],
                    vec![],
                    Type::None,
                    Expression::None,
                    false
                ),
                Expression::None,
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_record() {
        assert!(escapes(
            &Record::new(types::Record::new("foo"), vec![Variable::new("x").into()]).into(),
            "x",
        ));
    }

    #[test]
    fn escape_record_field() {
        assert!(escapes(
            &RecordField::new(types::Record::new("foo"), 0, Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn escape_original_record_in_record_update() {
        assert!(escapes(
            &RecordUpdate::new(types::Record::new("foo"), Variable::new("x"), vec![]).into(),
            "x",
        ));
    }

    #[test]
    fn escape_field_in_record_update() {
        assert!(escapes(
            &RecordUpdate::new(
                types::Record::new("foo"),
                Variable::new("y"),
                vec![RecordUpdateField::new(0, Variable::new("x"))]
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn escape_try_operation_operand() {
        assert!(escapes(
            &TryOperation::new(Variable::new("x"), "y", Type::None, Variable::new("y")).into(),
            "x"
        ));
    }

    #[test]
    fn escape_try_operation_then() {
        assert!(escapes(
            &TryOperation::new(Variable::new("y"), "z", Type::None, Variable::new("x")).into(),
            "x"
        ));
    }

    #[test]
    fn escape_try_operation_then_with_shadowed_variable() {
        assert!(!escapes(
            &TryOperation::new(Variable::new("y"), "x", Type::None, Variable::new("x")).into(),
            "x"
        ));
    }

    #[test]
    fn escape_variable() {
        assert!(escapes(&Variable::new("x").into(), "x"));
    }

    #[test]
    fn escape_variant() {
        assert!(escapes(
            &Variant::new(Type::None, Variable::new("x")).into(),
            "x",
        ));
    }
}
