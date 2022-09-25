use crate::ir::*;

// Does a function need to be boxed into a uniform representation?
pub fn is_boxed(expression: &Expression, name: &str) -> bool {
    let is_uniform = |expression| is_boxed(expression, name);

    match expression {
        Expression::ArithmeticOperation(operation) => {
            is_uniform(operation.lhs()) || is_uniform(operation.rhs())
        }
        Expression::Case(case) => {
            is_uniform(case.argument())
                || case.alternatives().iter().any(|alternative| {
                    alternative.name() != name && is_uniform(alternative.expression())
                })
                || case
                    .default_alternative()
                    .map(|alternative| {
                        alternative.name() != name && is_uniform(alternative.expression())
                    })
                    .unwrap_or_default()
        }
        Expression::CloneVariables(clone) => is_uniform(clone.expression()),
        Expression::ComparisonOperation(operation) => {
            is_uniform(operation.lhs()) || is_uniform(operation.rhs())
        }
        Expression::DropVariables(drop) => is_uniform(drop.expression()),
        Expression::Call(call) => {
            call.arguments().iter().any(is_uniform)
                || !matches!(call.function(), Expression::Variable(_))
                    && is_uniform(call.function())
                || call.arguments().iter().any(is_uniform)
        }
        Expression::If(if_) => {
            is_uniform(if_.condition()) || is_uniform(if_.then()) || is_uniform(if_.else_())
        }
        Expression::Let(let_) => {
            is_uniform(let_.bound_expression())
                || let_.name() != name && is_uniform(let_.expression())
        }
        Expression::LetRecursive(let_) => {
            let_.definition()
                .environment()
                .iter()
                .any(|free_variable| free_variable.name() == name)
                || let_.definition().name() != name && is_uniform(let_.expression())
        }
        Expression::Record(record) => record.fields().iter().any(is_uniform),
        Expression::RecordField(field) => is_uniform(field.record()),
        Expression::RecordUpdate(update) => {
            is_uniform(update.record())
                || update
                    .fields()
                    .iter()
                    .any(|field| is_uniform(field.expression()))
        }
        Expression::StringConcatenation(concatenation) => {
            concatenation.operands().iter().any(is_uniform)
        }
        Expression::Synchronize(synchronize) => is_uniform(synchronize.expression()),
        Expression::TryOperation(operation) => {
            is_uniform(operation.operand()) || is_uniform(operation.then())
        }
        Expression::TypeInformation(_) => todo!(),
        Expression::Variable(variable) => variable.name() == name,
        Expression::Variant(variant) => is_uniform(variant.payload()),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_) => false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{self, Type};

    #[test]
    fn check_arithmetic_operation() {
        assert!(is_boxed(
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
    fn check_arithmetic_operation_with_call_argument() {
        assert!(is_boxed(
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
    fn check_arithmetic_operation_with_call_function() {
        assert!(!is_boxed(
            &ArithmeticOperation::new(
                ArithmeticOperator::Add,
                Call::new(
                    types::Function::new(vec![], Type::None),
                    Variable::new("x"),
                    vec![]
                ),
                Expression::Number(0.0)
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn check_call_function() {
        assert!(!is_boxed(
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
    fn check_call_argument() {
        assert!(is_boxed(
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
    fn check_case_argument() {
        assert!(is_boxed(
            &Case::new(Variable::new("x"), vec![], None).into(),
            "x",
        ));
    }

    #[test]
    fn check_case_alternative() {
        assert!(is_boxed(
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
    fn check_case_alternative_with_shadowed_variable() {
        assert!(!is_boxed(
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
    fn check_case_default_alternative() {
        assert!(is_boxed(
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
    fn check_case_default_alternative_with_shadowed_variable() {
        assert!(!is_boxed(
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
    fn check_comparison_operation() {
        assert!(is_boxed(
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
    fn check_if_condition() {
        assert!(is_boxed(
            &If::new(Variable::new("x"), Expression::None, Expression::None).into(),
            "x",
        ));
    }

    #[test]
    fn check_if_then() {
        assert!(is_boxed(
            &If::new(Expression::None, Variable::new("x"), Expression::None).into(),
            "x",
        ));
    }

    #[test]
    fn check_if_else() {
        assert!(is_boxed(
            &If::new(Expression::None, Expression::None, Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn check_let_expression() {
        assert!(is_boxed(
            &Let::new("y", Type::None, Expression::None, Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn check_let_expression_with_shadowed_variable() {
        assert!(!is_boxed(
            &Let::new("x", Type::None, Expression::None, Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn check_let_bound_expression() {
        assert!(is_boxed(
            &Let::new("y", Type::None, Variable::new("x"), Expression::None).into(),
            "x",
        ));
    }

    #[test]
    fn check_let_recursive() {
        assert!(is_boxed(
            &LetRecursive::new(
                FunctionDefinition::new("y", vec![], Type::None, Expression::None),
                Variable::new("x")
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn check_let_recursive_with_shadowed_variable() {
        assert!(!is_boxed(
            &LetRecursive::new(
                FunctionDefinition::new("x", vec![], Type::None, Expression::None),
                Variable::new("x")
            )
            .into(),
            "x",
        ));
    }

    #[test]
    fn check_let_recursive_with_environment() {
        assert!(is_boxed(
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
    fn check_record() {
        assert!(is_boxed(
            &Record::new(types::Record::new("foo"), vec![Variable::new("x").into()]).into(),
            "x",
        ));
    }

    #[test]
    fn check_record_field() {
        assert!(is_boxed(
            &RecordField::new(types::Record::new("foo"), 0, Variable::new("x")).into(),
            "x",
        ));
    }

    #[test]
    fn check_original_record_in_record_update() {
        assert!(is_boxed(
            &RecordUpdate::new(types::Record::new("foo"), Variable::new("x"), vec![]).into(),
            "x",
        ));
    }

    #[test]
    fn check_field_in_record_update() {
        assert!(is_boxed(
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
    fn check_try_operation_operand() {
        assert!(is_boxed(
            &TryOperation::new(Variable::new("x"), "y", Type::None, Variable::new("y")).into(),
            "x"
        ));
    }

    #[test]
    fn check_try_operation_then() {
        assert!(is_boxed(
            &TryOperation::new(Variable::new("y"), "z", Type::None, Variable::new("x")).into(),
            "x"
        ));
    }

    #[test]
    fn check_try_operation_then_with_shadowed_variable() {
        assert!(is_boxed(
            &TryOperation::new(Variable::new("y"), "x", Type::None, Variable::new("x")).into(),
            "x"
        ));
    }

    #[test]
    fn check_variable() {
        assert!(is_boxed(&Variable::new("x").into(), "x"));
    }

    #[test]
    fn check_variant() {
        assert!(is_boxed(
            &Variant::new(Type::None, Variable::new("x")).into(),
            "x",
        ));
    }
}
