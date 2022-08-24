use crate::ir::*;

pub fn transform(expression: &Expression) -> Expression {
    transform_expression(expression, &Default::default())
}

fn transform_expression(expression: &Expression, variables: &hamt::Map<&str, &str>) -> Expression {
    let transform = |expression| transform_expression(expression, variables);

    match expression {
        Expression::ArithmeticOperation(operation) => ArithmeticOperation::new(
            operation.operator(),
            transform(operation.lhs()),
            transform(operation.rhs()),
        )
        .into(),
        Expression::Case(case) => Case::new(
            transform(case.argument()),
            case.alternatives()
                .iter()
                .map(|alternative| {
                    Alternative::new(
                        alternative.types().to_vec(),
                        alternative.name(),
                        transform_expression(
                            alternative.expression(),
                            variables
                                .remove(alternative.name())
                                .as_ref()
                                .unwrap_or(&variables),
                        ),
                    )
                })
                .collect(),
            case.default_alternative().map(|alternative| {
                DefaultAlternative::new(
                    alternative.name(),
                    transform_expression(
                        alternative.expression(),
                        variables
                            .remove(alternative.name())
                            .as_ref()
                            .unwrap_or(&variables),
                    ),
                )
            }),
        )
        .into(),
        Expression::CloneVariables(clone) => {
            CloneVariables::new(clone.variables().clone(), transform(clone.expression())).into()
        }
        Expression::ComparisonOperation(operation) => ComparisonOperation::new(
            operation.operator(),
            transform(operation.lhs()),
            transform(operation.rhs()),
        )
        .into(),
        Expression::DropVariables(drop) => {
            DropVariables::new(drop.variables().clone(), transform(drop.expression())).into()
        }
        Expression::Call(call) => Call::new(
            call.type_().clone(),
            transform(call.function()),
            call.arguments().iter().map(transform).collect(),
        )
        .into(),
        Expression::If(if_) => If::new(
            transform(if_.condition()),
            transform(if_.then()),
            transform(if_.else_()),
        )
        .into(),
        Expression::Let(let_) => match transform(let_.bound_expression()) {
            Expression::Variable(variable) => transform_expression(
                let_.expression(),
                &variables.insert(let_.name(), variable.name()),
            ),
            bound_expression => Let::new(
                let_.name(),
                let_.type_().clone(),
                bound_expression,
                transform_expression(
                    let_.expression(),
                    variables.remove(let_.name()).as_ref().unwrap_or(variables),
                ),
            )
            .into(),
        },
        Expression::LetRecursive(let_) => LetRecursive::new(
            transform_function_definition(let_.definition(), &variables),
            transform_expression(
                let_.expression(),
                variables
                    .remove(let_.definition().name())
                    .as_ref()
                    .unwrap_or(variables),
            ),
        )
        .into(),
        Expression::Synchronize(synchronize) => Synchronize::new(
            synchronize.type_().clone(),
            transform(synchronize.expression()),
        )
        .into(),
        Expression::Record(record) => Record::new(
            record.type_().clone(),
            record.fields().iter().map(transform).collect(),
        )
        .into(),
        Expression::RecordField(field) => RecordField::new(
            field.type_().clone(),
            field.index(),
            transform(field.record()),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            transform(update.record()),
            update
                .fields()
                .iter()
                .map(|field| RecordUpdateField::new(field.index(), transform(field.expression())))
                .collect(),
        )
        .into(),
        Expression::TryOperation(operation) => TryOperation::new(
            transform(operation.operand()),
            operation.name(),
            operation.type_().clone(),
            transform_expression(
                operation.then(),
                variables
                    .remove(operation.name())
                    .as_ref()
                    .unwrap_or(variables),
            ),
        )
        .into(),
        Expression::Variant(variant) => {
            Variant::new(variant.type_().clone(), transform(variant.payload())).into()
        }
        Expression::Variable(variable) => if let Some(&name) = variables.get(variable.name()) {
            Variable::new(name)
        } else {
            variable.clone()
        }
        .into(),
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_) => expression.clone(),
    }
}

fn transform_function_definition(
    definition: &FunctionDefinition,
    variables: &hamt::Map<&str, &str>,
) -> FunctionDefinition {
    FunctionDefinition::with_options(
        definition.name(),
        definition
            .environment()
            .iter()
            .map(|free_variable| {
                Argument::new(
                    variables
                        .get(free_variable.name())
                        .copied()
                        .unwrap_or(free_variable.name()),
                    free_variable.type_().clone(),
                )
            })
            .collect(),
        definition.arguments().to_vec(),
        definition.result_type().clone(),
        {
            let mut variables = variables
                .remove(definition.name())
                .unwrap_or(variables.clone());

            for argument in definition.arguments() {
                variables = variables
                    .remove(argument.name())
                    .unwrap_or(variables.clone());
            }

            transform_expression(definition.body(), &variables)
        },
        definition.is_thunk(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Type;
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_let() {
        assert_eq!(
            transform(&Let::new("x", Type::Number, Variable::new("y"), Variable::new("x")).into()),
            Variable::new("y").into()
        );
    }

    #[test]
    fn transform_nested_let() {
        assert_eq!(
            transform(
                &Let::new(
                    "x",
                    Type::Number,
                    Variable::new("y"),
                    Let::new("z", Type::Number, Variable::new("x"), Variable::new("z"))
                )
                .into()
            ),
            Variable::new("y").into()
        );
    }

    mod case {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        Case::new(
                            Variable::new("x"),
                            vec![Alternative::new(
                                vec![Type::Number],
                                "z",
                                Variable::new("x")
                            )],
                            Some(DefaultAlternative::new("z", Variable::new("x")))
                        )
                    )
                    .into()
                ),
                Case::new(
                    Variable::new("y"),
                    vec![Alternative::new(
                        vec![Type::Number],
                        "z",
                        Variable::new("y")
                    )],
                    Some(DefaultAlternative::new("z", Variable::new("y")))
                )
                .into()
            );
        }

        #[test]
        fn transform_with_shadowed_variable() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        Case::new(
                            Variable::new("x"),
                            vec![Alternative::new(
                                vec![Type::Number],
                                "x",
                                Variable::new("x")
                            )],
                            Some(DefaultAlternative::new("x", Variable::new("x")))
                        )
                    )
                    .into()
                ),
                Case::new(
                    Variable::new("y"),
                    vec![Alternative::new(
                        vec![Type::Number],
                        "x",
                        Variable::new("x")
                    )],
                    Some(DefaultAlternative::new("x", Variable::new("x")))
                )
                .into()
            );
        }

        #[test]
        fn transform_with_shadowed_alias_variable() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        Case::new(
                            Variable::new("x"),
                            vec![Alternative::new(
                                vec![Type::Number],
                                "y",
                                Variable::new("x")
                            )],
                            Some(DefaultAlternative::new("y", Variable::new("x")))
                        )
                    )
                    .into()
                ),
                Case::new(
                    Variable::new("y"),
                    vec![Alternative::new(
                        vec![Type::Number],
                        "y",
                        Variable::new("y")
                    )],
                    Some(DefaultAlternative::new("y", Variable::new("y")))
                )
                .into()
            );
        }
    }

    mod let_recursive {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        LetRecursive::new(
                            FunctionDefinition::new("f", vec![], Type::Number, Variable::new("x"),),
                            Variable::new("x")
                        )
                    )
                    .into()
                ),
                LetRecursive::new(
                    FunctionDefinition::new("f", vec![], Type::Number, Variable::new("y")),
                    Variable::new("y")
                )
                .into()
            );
        }

        #[test]
        fn transform_with_shadowed_function_name() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        LetRecursive::new(
                            FunctionDefinition::new("x", vec![], Type::Number, Variable::new("x")),
                            Variable::new("x")
                        )
                    )
                    .into()
                ),
                LetRecursive::new(
                    FunctionDefinition::new("x", vec![], Type::Number, Variable::new("x")),
                    Variable::new("x")
                )
                .into()
            );
        }

        #[test]
        fn transform_with_shadowed_argument_name() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        LetRecursive::new(
                            FunctionDefinition::new(
                                "f",
                                vec![Argument::new("x", Type::Number)],
                                Type::Number,
                                Variable::new("x")
                            ),
                            Variable::new("x")
                        )
                    )
                    .into()
                ),
                LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        Variable::new("x")
                    ),
                    Variable::new("y")
                )
                .into()
            );
        }

        #[test]
        fn transform_with_free_variable() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        LetRecursive::new(
                            FunctionDefinition::with_options(
                                "f",
                                vec![Argument::new("x", Type::Number)],
                                vec![],
                                Type::Number,
                                42.0,
                                false
                            ),
                            42.0,
                        )
                    )
                    .into()
                ),
                LetRecursive::new(
                    FunctionDefinition::with_options(
                        "f",
                        vec![Argument::new("y", Type::Number)],
                        vec![],
                        Type::Number,
                        42.0,
                        false
                    ),
                    42.0,
                )
                .into()
            );
        }
    }

    mod try_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        TryOperation::new(
                            Variable::new("x"),
                            "z",
                            Type::Number,
                            Variable::new("x")
                        )
                    )
                    .into()
                ),
                TryOperation::new(Variable::new("y"), "z", Type::Number, Variable::new("y")).into()
            );
        }

        #[test]
        fn transform_with_shadowed_variable() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        TryOperation::new(
                            Variable::new("x"),
                            "x",
                            Type::Number,
                            Variable::new("x")
                        )
                    )
                    .into()
                ),
                TryOperation::new(Variable::new("y"), "x", Type::Number, Variable::new("x")).into()
            );
        }

        #[test]
        fn transform_with_shadowed_alias_variable() {
            assert_eq!(
                transform(
                    &Let::new(
                        "x",
                        Type::Number,
                        Variable::new("y"),
                        TryOperation::new(
                            Variable::new("x"),
                            "y",
                            Type::Number,
                            Variable::new("x")
                        )
                    )
                    .into()
                ),
                TryOperation::new(Variable::new("y"), "y", Type::Number, Variable::new("y")).into()
            );
        }
    }
}
