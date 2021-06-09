mod error;

use crate::{ir::*, types::Type};
pub use error::ReferenceCountError;
use std::collections::{HashMap, HashSet};

// Closure environments need to be inferred before reference counting.
pub fn count_references(module: &Module) -> Result<Module, ReferenceCountError> {
    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.declarations().to_vec(),
        module
            .definitions()
            .iter()
            .map(convert_definition)
            .collect::<Result<_, _>>()?,
    ))
}

fn convert_definition(definition: &Definition) -> Result<Definition, ReferenceCountError> {
    // Backend is expected to clone a function itself and its free variables at the very beginning
    // of the function.
    let owned_variables = vec![(definition.name().into(), definition.type_().clone().into())]
        .into_iter()
        .chain(
            definition
                .environment()
                .iter()
                .chain(definition.arguments())
                .map(|argument| (argument.name().into(), argument.type_().clone())),
        )
        .collect();

    let (expression, moved_variables) =
        convert_expression(definition.body(), &owned_variables, &Default::default())?;

    Ok(Definition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        drop_variables(
            expression,
            owned_variables
                .keys()
                .filter(|variable| !moved_variables.contains(variable.as_str()))
                .cloned()
                .collect(),
            &owned_variables,
        ),
        definition.result_type().clone(),
        definition.is_thunk(),
    ))
}

// Here, we convert expressions tracking moved variables and cloning variables moved already.
// The basic rules are listed below.
//
// - The returned values of functions are moved.
// - Every input of expressions is moved including conditions of if expressions and records of record
//   element operations.
// - Newly bound variables in let expressions are dropped if they are not moved in their expressions.
fn convert_expression(
    expression: &Expression,
    owned_variables: &HashMap<String, Type>,
    moved_variables: &HashSet<String>,
) -> Result<(Expression, HashSet<String>), ReferenceCountError> {
    Ok(match expression {
        Expression::ArithmeticOperation(operation) => {
            let (rhs, moved_variables) =
                convert_expression(operation.rhs(), owned_variables, &moved_variables)?;
            let (lhs, moved_variables) =
                convert_expression(operation.lhs(), owned_variables, &moved_variables)?;

            (
                ArithmeticOperation::new(operation.operator(), lhs, rhs).into(),
                moved_variables,
            )
        }
        Expression::Case(case) => {
            let (default_alternative, default_alternative_moved_variables) =
                if let Some(alternative) = case.default_alternative() {
                    let (expression, moved_variables) = convert_expression(
                        alternative.expression(),
                        &owned_variables
                            .clone()
                            .into_iter()
                            .chain(vec![(alternative.name().into(), Type::Variant)])
                            .collect(),
                        &moved_variables
                            .clone()
                            .into_iter()
                            .filter(|variable| variable != alternative.name())
                            .collect(),
                    )?;

                    (
                        Some(DefaultAlternative::new(alternative.name(), expression)),
                        moved_variables,
                    )
                } else {
                    (None, moved_variables.clone())
                };

            let alternative_tuples = case
                .alternatives()
                .iter()
                .map(|alternative| {
                    let (expression, moved_variables) = convert_expression(
                        alternative.expression(),
                        &owned_variables
                            .clone()
                            .into_iter()
                            .chain(vec![(
                                alternative.name().into(),
                                alternative.type_().clone(),
                            )])
                            .collect(),
                        &moved_variables
                            .clone()
                            .into_iter()
                            .filter(|variable| variable != alternative.name())
                            .collect(),
                    )?;

                    Ok((
                        Alternative::new(
                            alternative.type_().clone(),
                            alternative.name(),
                            expression,
                        ),
                        moved_variables,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let alternative_moved_variables = default_alternative_moved_variables
                .iter()
                .cloned()
                .filter(|variable| {
                    if let Some(alternative) = case.default_alternative() {
                        variable != alternative.name()
                    } else {
                        true
                    }
                })
                .chain(
                    alternative_tuples
                        .iter()
                        .flat_map(|(alternative, moved_variables)| {
                            moved_variables
                                .iter()
                                .cloned()
                                .filter(|variable| variable != alternative.name())
                                .collect::<HashSet<String>>()
                        }),
                )
                .collect::<HashSet<_>>();

            let (argument, moved_variables) = convert_expression(
                case.argument(),
                owned_variables,
                &moved_variables
                    .iter()
                    .cloned()
                    .chain(alternative_moved_variables.clone())
                    .collect(),
            )?;

            (
                Case::new(
                    argument,
                    alternative_tuples
                        .into_iter()
                        .map(|(alternative, moved_variables)| {
                            Alternative::new(
                                alternative.type_().clone(),
                                alternative.name(),
                                drop_variables(
                                    alternative.expression().clone(),
                                    alternative_moved_variables
                                        .clone()
                                        .into_iter()
                                        .chain(vec![alternative.name().into()])
                                        .collect::<HashSet<_>>()
                                        .difference(&moved_variables)
                                        .cloned()
                                        .collect(),
                                    &owned_variables
                                        .clone()
                                        .into_iter()
                                        .chain(vec![(
                                            alternative.name().into(),
                                            alternative.type_().clone(),
                                        )])
                                        .collect(),
                                ),
                            )
                        })
                        .collect(),
                    default_alternative.map(|alternative| {
                        DefaultAlternative::new(
                            alternative.name(),
                            drop_variables(
                                alternative.expression().clone(),
                                alternative_moved_variables
                                    .into_iter()
                                    .chain(vec![alternative.name().into()])
                                    .collect::<HashSet<_>>()
                                    .difference(&default_alternative_moved_variables)
                                    .cloned()
                                    .collect(),
                                &owned_variables
                                    .clone()
                                    .into_iter()
                                    .chain(vec![(alternative.name().into(), Type::Variant)])
                                    .collect(),
                            ),
                        )
                    }),
                )
                .into(),
                moved_variables,
            )
        }
        Expression::ComparisonOperation(operation) => {
            let (rhs, moved_variables) =
                convert_expression(operation.rhs(), owned_variables, &moved_variables)?;
            let (lhs, moved_variables) =
                convert_expression(operation.lhs(), owned_variables, &moved_variables)?;

            (
                ComparisonOperation::new(operation.operator(), lhs, rhs).into(),
                moved_variables,
            )
        }
        Expression::FunctionApplication(application) => {
            let (argument, moved_variables) =
                convert_expression(application.argument(), owned_variables, moved_variables)?;
            let (function, moved_variables) =
                convert_expression(application.function(), owned_variables, &moved_variables)?;

            (
                FunctionApplication::new(application.type_().clone(), function, argument).into(),
                moved_variables,
            )
        }
        Expression::If(if_) => {
            let (then, then_moved_variables) =
                convert_expression(if_.then(), owned_variables, moved_variables)?;
            let (else_, else_moved_variables) =
                convert_expression(if_.else_(), owned_variables, moved_variables)?;

            let all_moved_variables = then_moved_variables
                .clone()
                .into_iter()
                .chain(else_moved_variables.clone())
                .collect();

            let (condition, moved_variables) =
                convert_expression(if_.condition(), owned_variables, &all_moved_variables)?;

            (
                If::new(
                    condition,
                    drop_variables(
                        then,
                        all_moved_variables
                            .difference(&then_moved_variables)
                            .cloned()
                            .collect(),
                        owned_variables,
                    ),
                    drop_variables(
                        else_,
                        all_moved_variables
                            .difference(&else_moved_variables)
                            .cloned()
                            .collect(),
                        owned_variables,
                    ),
                )
                .into(),
                moved_variables,
            )
        }
        Expression::Let(let_) => {
            let let_owned_variables = owned_variables
                .clone()
                .into_iter()
                .chain(vec![(let_.name().into(), let_.type_().clone())])
                .collect();
            let (expression, expression_moved_variables) = convert_expression(
                let_.expression(),
                &let_owned_variables,
                &moved_variables
                    .iter()
                    .cloned()
                    .filter(|variable| variable != let_.name())
                    .collect(),
            )?;
            let (bound_expression, moved_variables) = convert_expression(
                let_.bound_expression(),
                &owned_variables,
                &moved_variables
                    .clone()
                    .into_iter()
                    .chain(
                        expression_moved_variables
                            .iter()
                            .cloned()
                            .filter(|variable| variable != let_.name()),
                    )
                    .collect(),
            )?;

            (
                Let::new(
                    let_.name(),
                    let_.type_().clone(),
                    bound_expression,
                    if expression_moved_variables.contains(let_.name()) {
                        expression
                    } else {
                        drop_variables(
                            expression,
                            vec![let_.name().into()].into_iter().collect(),
                            &let_owned_variables,
                        )
                    },
                )
                .into(),
                moved_variables,
            )
        }
        Expression::LetRecursive(let_) => {
            let let_owned_variables = owned_variables
                .clone()
                .into_iter()
                .chain(vec![(
                    let_.definition().name().into(),
                    let_.definition().type_().clone().into(),
                )])
                .collect();
            let (expression, expression_moved_variables) = convert_expression(
                let_.expression(),
                &let_owned_variables,
                &moved_variables
                    .iter()
                    .cloned()
                    .filter(|variable| variable != let_.definition().name())
                    .collect(),
            )?;
            let moved_variables = moved_variables
                .clone()
                .into_iter()
                .chain(
                    expression_moved_variables
                        .iter()
                        .cloned()
                        .filter(|variable| variable != let_.definition().name()),
                )
                .collect();
            let cloned_variables = let_
                .definition()
                .environment()
                .iter()
                .map(|argument| argument.name().into())
                .collect::<HashSet<_>>()
                .intersection(&moved_variables)
                .cloned()
                .collect::<HashSet<_>>();

            (
                clone_variables(
                    LetRecursive::new(
                        convert_definition(let_.definition())?,
                        if expression_moved_variables.contains(let_.definition().name()) {
                            expression
                        } else {
                            drop_variables(
                                expression,
                                vec![let_.definition().name().into()].into_iter().collect(),
                                &let_owned_variables,
                            )
                        },
                    ),
                    cloned_variables,
                    owned_variables,
                ),
                moved_variables
                    .into_iter()
                    .chain(
                        let_.definition()
                            .environment()
                            .iter()
                            .map(|argument| argument.name().into()),
                    )
                    .collect::<HashSet<String>>(),
            )
        }
        Expression::Record(record) => {
            let (elements, moved_variables) = record.elements().iter().rev().fold(
                Ok((vec![], moved_variables.clone())),
                |result, element| {
                    let (elements, moved_variables) = result?;
                    let (element, moved_variables) =
                        convert_expression(element, owned_variables, &moved_variables)?;

                    Ok((
                        vec![element].into_iter().chain(elements).collect(),
                        moved_variables,
                    ))
                },
            )?;

            (
                Record::new(record.type_().clone(), elements).into(),
                moved_variables,
            )
        }
        Expression::RecordElement(element) => {
            let (record, moved_variables) =
                convert_expression(element.record(), owned_variables, moved_variables)?;

            (
                RecordElement::new(element.type_().clone(), element.index(), record).into(),
                moved_variables,
            )
        }
        Expression::Variable(variable) => {
            if should_clone_variable(variable.name(), owned_variables, moved_variables) {
                (
                    clone_variables(
                        variable.clone(),
                        vec![variable.name().into()].into_iter().collect(),
                        owned_variables,
                    ),
                    moved_variables.clone(),
                )
            } else {
                (
                    variable.clone().into(),
                    moved_variables
                        .clone()
                        .into_iter()
                        .chain(vec![variable.name().into()])
                        .collect(),
                )
            }
        }
        Expression::Variant(variant) => {
            let (expression, moved_variables) =
                convert_expression(variant.payload(), owned_variables, moved_variables)?;

            (
                Variant::new(variant.type_().clone(), expression).into(),
                moved_variables,
            )
        }
        Expression::Boolean(_) | Expression::ByteString(_) | Expression::Number(_) => {
            (expression.clone(), moved_variables.clone())
        }
        Expression::CloneVariables(_) | Expression::DropVariables(_) => {
            return Err(ReferenceCountError::ExpressionNotSupported(
                expression.clone(),
            ));
        }
    })
}

fn clone_variables(
    expression: impl Into<Expression>,
    cloned_variables: HashSet<String>,
    owned_variables: &HashMap<String, Type>,
) -> Expression {
    let expression = expression.into();

    if cloned_variables.is_empty() {
        expression
    } else {
        CloneVariables::new(
            owned_variables
                .clone()
                .into_iter()
                .filter(|(variable, _)| cloned_variables.contains(variable.as_str()))
                .collect(),
            expression,
        )
        .into()
    }
}

fn drop_variables(
    expression: impl Into<Expression>,
    dropped_variables: HashSet<String>,
    owned_variables: &HashMap<String, Type>,
) -> Expression {
    let expression = expression.into();

    if dropped_variables.is_empty() {
        expression
    } else {
        DropVariables::new(
            owned_variables
                .clone()
                .into_iter()
                .filter(|(variable, _)| dropped_variables.contains(variable.as_str()))
                .collect(),
            expression,
        )
        .into()
    }
}

fn should_clone_variable(
    variable: &str,
    owned_variables: &HashMap<String, Type>,
    moved_variables: &HashSet<String>,
) -> bool {
    owned_variables.contains_key(variable) && moved_variables.contains(variable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{self, Type};

    #[test]
    fn convert_record() {
        assert_eq!(
            convert_expression(
                &Record::new(
                    types::Record::new("a"),
                    vec![Variable::new("x").into(), Variable::new("x").into()]
                )
                .into(),
                &vec![("x".into(), Type::Number)].into_iter().collect(),
                &Default::default()
            )
            .unwrap(),
            (
                Record::new(
                    types::Record::new("a"),
                    vec![
                        CloneVariables::new(
                            vec![("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        )
                        .into(),
                        Variable::new("x").into()
                    ]
                )
                .into(),
                vec!["x".into()].into_iter().collect()
            ),
        );
    }

    mod function_applications {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_single() {
            assert_eq!(
                convert_expression(
                    &FunctionApplication::new(
                        types::Function::new(Type::Number, Type::Number),
                        Variable::new("f"),
                        Variable::new("x")
                    )
                    .into(),
                    &vec![
                        (
                            "f".into(),
                            types::Function::new(Type::Number, Type::Number).into()
                        ),
                        ("x".into(), Type::Number)
                    ]
                    .into_iter()
                    .collect(),
                    &vec!["f".into(), "x".into()].into_iter().collect(),
                )
                .unwrap(),
                (
                    FunctionApplication::new(
                        types::Function::new(Type::Number, Type::Number),
                        CloneVariables::new(
                            vec![(
                                "f".into(),
                                types::Function::new(Type::Number, Type::Number).into()
                            )]
                            .into_iter()
                            .collect(),
                            Variable::new("f")
                        ),
                        CloneVariables::new(
                            vec![("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        )
                    )
                    .into(),
                    vec!["f".into(), "x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_multiple() {
            assert_eq!(
                convert_expression(
                    &FunctionApplication::new(
                        types::Function::new(Type::Number, Type::Number),
                        FunctionApplication::new(
                            types::Function::new(
                                Type::Number,
                                types::Function::new(Type::Number, Type::Number)
                            ),
                            Variable::new("f"),
                            Variable::new("x")
                        ),
                        Variable::new("x")
                    )
                    .into(),
                    &vec![
                        (
                            "f".into(),
                            types::Function::new(
                                Type::Number,
                                types::Function::new(Type::Number, Type::Number)
                            )
                            .into()
                        ),
                        ("x".into(), Type::Number)
                    ]
                    .into_iter()
                    .collect(),
                    &Default::default(),
                )
                .unwrap(),
                (
                    FunctionApplication::new(
                        types::Function::new(Type::Number, Type::Number),
                        FunctionApplication::new(
                            types::Function::new(
                                Type::Number,
                                types::Function::new(Type::Number, Type::Number)
                            ),
                            Variable::new("f"),
                            CloneVariables::new(
                                vec![("x".into(), Type::Number)].into_iter().collect(),
                                Variable::new("x")
                            )
                        ),
                        Variable::new("x")
                    )
                    .into(),
                    vec!["f".into(), "x".into()].into_iter().collect()
                ),
            );
        }
    }

    mod let_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_with_moved_variable() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, 42.0, Variable::new("x")).into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                Let::new("x", Type::Number, 42.0, Variable::new("x")).into(),
            );
        }

        #[test]
        fn convert_with_cloned_variable() {
            assert_eq!(
                convert_expression(
                    &Let::new(
                        "x",
                        Type::Number,
                        42.0,
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Variable::new("x"),
                            Variable::new("x")
                        ),
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                Let::new(
                    "x",
                    Type::Number,
                    42.0,
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        CloneVariables::new(
                            vec![("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        ),
                        Variable::new("x")
                    ),
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_dropped_variable() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, 42.0, 42.0,).into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                Let::new(
                    "x",
                    Type::Number,
                    42.0,
                    DropVariables::new(
                        vec![("x".into(), Type::Number)].into_iter().collect(),
                        42.0
                    )
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_moved_variable_in_bound_expression() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, Variable::new("y"), Variable::new("x")).into(),
                    &vec![("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new("x", Type::Number, Variable::new("y"), Variable::new("x")).into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_with_cloned_variable_in_bound_expression() {
            assert_eq!(
                convert_expression(
                    &Let::new("x", Type::Number, Variable::new("y"), Variable::new("y")).into(),
                    &vec![("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "x",
                        Type::Number,
                        CloneVariables::new(
                            vec![("y".into(), Type::Number)].into_iter().collect(),
                            Variable::new("y")
                        ),
                        DropVariables::new(
                            vec![("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("y")
                        )
                    )
                    .into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_nested_let() {
            assert_eq!(
                convert_expression(
                    &Let::new(
                        "y",
                        Type::Number,
                        Let::new("x", Type::Number, Variable::new("x"), Variable::new("x")),
                        Variable::new("x")
                    )
                    .into(),
                    &vec![("x".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "y",
                        Type::Number,
                        Let::new(
                            "x",
                            Type::Number,
                            CloneVariables::new(
                                vec![("x".into(), Type::Number)].into_iter().collect(),
                                Variable::new("x")
                            ),
                            Variable::new("x")
                        ),
                        DropVariables::new(
                            vec![("y".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        )
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                )
            );
        }
    }

    mod let_recursive {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_with_moved_variable() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        DropVariables::new(
                            vec![
                                (
                                    "f".into(),
                                    types::Function::new(Type::Number, Type::Number).into()
                                ),
                                ("x".into(), Type::Number)
                            ]
                            .into_iter()
                            .collect(),
                            42.0,
                        ),
                        Type::Number
                    ),
                    Variable::new("f")
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_cloned_variable() {
            let f_type = types::Function::new(Type::Number, Type::Number);
            let g_type = types::Function::new(
                types::Function::new(Type::Number, Type::Number),
                types::Function::new(
                    types::Function::new(Type::Number, Type::Number),
                    Type::Number,
                ),
            );

            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        FunctionApplication::new(
                            f_type.clone(),
                            FunctionApplication::new(
                                g_type.clone(),
                                Variable::new("g"),
                                Variable::new("f")
                            ),
                            Variable::new("f")
                        )
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        DropVariables::new(
                            vec![
                                (
                                    "f".into(),
                                    types::Function::new(Type::Number, Type::Number).into()
                                ),
                                ("x".into(), Type::Number)
                            ]
                            .into_iter()
                            .collect(),
                            42.0,
                        ),
                        Type::Number
                    ),
                    FunctionApplication::new(
                        f_type,
                        FunctionApplication::new(
                            g_type,
                            Variable::new("g"),
                            CloneVariables::new(
                                vec![(
                                    "f".into(),
                                    types::Function::new(Type::Number, Type::Number).into()
                                )]
                                .into_iter()
                                .collect(),
                                Variable::new("f")
                            )
                        ),
                        Variable::new("f")
                    )
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_dropped_variable() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        42.0,
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                LetRecursive::new(
                    Definition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        DropVariables::new(
                            vec![
                                (
                                    "f".into(),
                                    types::Function::new(Type::Number, Type::Number).into()
                                ),
                                ("x".into(), Type::Number),
                            ]
                            .into_iter()
                            .collect(),
                            42.0,
                        ),
                        Type::Number
                    ),
                    DropVariables::new(
                        vec![(
                            "f".into(),
                            types::Function::new(Type::Number, Type::Number).into()
                        )]
                        .into_iter()
                        .collect(),
                        42.0,
                    )
                )
                .into(),
            );
        }

        #[test]
        fn convert_with_moved_variable_in_environment() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::with_environment(
                            "f",
                            vec![Argument::new("y", Type::Number)],
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    &vec![("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    LetRecursive::new(
                        Definition::with_environment(
                            "f",
                            vec![Argument::new("y", Type::Number)],
                            vec![Argument::new("x", Type::Number)],
                            DropVariables::new(
                                vec![
                                    (
                                        "f".into(),
                                        types::Function::new(Type::Number, Type::Number).into()
                                    ),
                                    ("x".into(), Type::Number),
                                    ("y".into(), Type::Number)
                                ]
                                .into_iter()
                                .collect(),
                                42.0,
                            ),
                            Type::Number
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_with_cloned_variable_in_environment() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::with_environment(
                            "f",
                            vec![Argument::new("y", Type::Number)],
                            vec![Argument::new("x", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        FunctionApplication::new(
                            types::Function::new(Type::Number, Type::Number),
                            Variable::new("f"),
                            Variable::new("y")
                        )
                    )
                    .into(),
                    &vec![("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    CloneVariables::new(
                        vec![("y".into(), Type::Number)].into_iter().collect(),
                        LetRecursive::new(
                            Definition::with_environment(
                                "f",
                                vec![Argument::new("y", Type::Number)],
                                vec![Argument::new("x", Type::Number)],
                                DropVariables::new(
                                    vec![
                                        (
                                            "f".into(),
                                            types::Function::new(Type::Number, Type::Number).into()
                                        ),
                                        ("x".into(), Type::Number),
                                        ("y".into(), Type::Number),
                                    ]
                                    .into_iter()
                                    .collect(),
                                    42.0,
                                ),
                                Type::Number
                            ),
                            FunctionApplication::new(
                                types::Function::new(Type::Number, Type::Number),
                                Variable::new("f"),
                                Variable::new("y")
                            )
                        )
                    )
                    .into(),
                    vec!["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_let_recursive_in_let() {
            let function_type = types::Function::new(Type::Number, Type::Number);

            assert_eq!(
                convert_expression(
                    &Let::new(
                        "g",
                        function_type.clone(),
                        LetRecursive::new(
                            Definition::with_environment(
                                "f",
                                vec![Argument::new("f", Type::Number)],
                                vec![Argument::new("x", Type::Number)],
                                FunctionApplication::new(
                                    function_type.clone(),
                                    Variable::new("f"),
                                    Variable::new("x")
                                ),
                                Type::Number
                            ),
                            Variable::new("f")
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    &vec![("f".into(), function_type.clone().into())]
                        .into_iter()
                        .collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "g",
                        function_type.clone(),
                        CloneVariables::new(
                            vec![("f".into(), function_type.clone().into())]
                                .into_iter()
                                .collect(),
                            LetRecursive::new(
                                Definition::with_environment(
                                    "f",
                                    vec![Argument::new("f", Type::Number)],
                                    vec![Argument::new("x", Type::Number)],
                                    FunctionApplication::new(
                                        function_type.clone(),
                                        Variable::new("f"),
                                        Variable::new("x")
                                    ),
                                    Type::Number
                                ),
                                Variable::new("f")
                            )
                        ),
                        DropVariables::new(
                            vec![("g".into(), function_type.into())]
                                .into_iter()
                                .collect(),
                            Variable::new("f")
                        )
                    )
                    .into(),
                    vec!["f".into()].into_iter().collect()
                )
            );
        }

        #[test]
        fn convert_with_moved_free_variable() {
            assert_eq!(
                convert_expression(
                    &LetRecursive::new(
                        Definition::with_environment(
                            "x",
                            vec![Argument::new("x", Type::Number)],
                            vec![Argument::new("y", Type::Number)],
                            42.0,
                            Type::Number
                        ),
                        42.0
                    )
                    .into(),
                    &vec![("x".into(), Type::Number)].into_iter().collect(),
                    &vec!["x".into()].into_iter().collect()
                )
                .unwrap(),
                (
                    CloneVariables::new(
                        vec![("x".into(), Type::Number)].into_iter().collect(),
                        LetRecursive::new(
                            Definition::with_environment(
                                "x",
                                vec![Argument::new("x", Type::Number)],
                                vec![Argument::new("y", Type::Number)],
                                DropVariables::new(
                                    vec![("y".into(), Type::Number), ("x".into(), Type::Number)]
                                        .into_iter()
                                        .collect(),
                                    42.0,
                                ),
                                Type::Number
                            ),
                            DropVariables::new(
                                vec![(
                                    "x".into(),
                                    types::Function::new(Type::Number, Type::Number).into()
                                )]
                                .into_iter()
                                .collect(),
                                42.0,
                            )
                        )
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                )
            );
        }
    }

    mod definitions {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_with_dropped_argument() {
            assert_eq!(
                convert_definition(&Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    42.0,
                    Type::Number
                ))
                .unwrap(),
                Definition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    DropVariables::new(
                        vec![
                            (
                                "f".into(),
                                types::Function::new(Type::Number, Type::Number).into()
                            ),
                            ("x".into(), Type::Number)
                        ]
                        .into_iter()
                        .collect(),
                        42.0
                    ),
                    Type::Number
                ),
            );
        }

        #[test]
        fn convert_with_dropped_free_variable() {
            assert_eq!(
                convert_definition(&Definition::with_environment(
                    "f",
                    vec![Argument::new("y", Type::Number)],
                    vec![Argument::new("x", Type::Number)],
                    42.0,
                    Type::Number
                ))
                .unwrap(),
                Definition::with_environment(
                    "f",
                    vec![Argument::new("y", Type::Number)],
                    vec![Argument::new("x", Type::Number)],
                    DropVariables::new(
                        vec![
                            (
                                "f".into(),
                                types::Function::new(Type::Number, Type::Number).into()
                            ),
                            ("x".into(), Type::Number),
                            ("y".into(), Type::Number)
                        ]
                        .into_iter()
                        .collect(),
                        42.0
                    ),
                    Type::Number
                ),
            );
        }
    }

    mod if_ {
        use super::*;

        #[test]
        fn convert_if() {
            assert_eq!(
                convert_expression(
                    &If::new(Variable::new("x"), Variable::new("y"), Variable::new("z")).into(),
                    &vec![
                        ("x".into(), Type::Number),
                        ("y".into(), Type::Number),
                        ("z".into(), Type::Number),
                    ]
                    .into_iter()
                    .collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    If::new(
                        Variable::new("x"),
                        DropVariables::new(
                            vec![("z".into(), Type::Number)].into_iter().collect(),
                            Variable::new("y")
                        ),
                        DropVariables::new(
                            vec![("y".into(), Type::Number)].into_iter().collect(),
                            Variable::new("z")
                        )
                    )
                    .into(),
                    vec!["x".into(), "y".into(), "z".into()]
                        .into_iter()
                        .collect()
                ),
            );
        }
    }

    mod case {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn convert_case_with_default_alternative() {
            assert_eq!(
                convert_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![],
                        Some(DefaultAlternative::new("x", 42.0))
                    )
                    .into(),
                    &vec![("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Case::new(
                        Variable::new("x"),
                        vec![],
                        Some(DefaultAlternative::new(
                            "x",
                            DropVariables::new(
                                vec![("x".into(), Type::Variant)].into_iter().collect(),
                                42.0
                            )
                        ))
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_case_with_alternatives() {
            assert_eq!(
                convert_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![
                            Alternative::new(Type::Number, "x", Variable::new("x")),
                            Alternative::new(Type::Boolean, "x", 42.0)
                        ],
                        None
                    )
                    .into(),
                    &vec![("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Case::new(
                        Variable::new("x"),
                        vec![
                            Alternative::new(Type::Number, "x", Variable::new("x")),
                            Alternative::new(
                                Type::Boolean,
                                "x",
                                DropVariables::new(
                                    vec![("x".into(), Type::Boolean)].into_iter().collect(),
                                    42.0
                                )
                            )
                        ],
                        None
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_case_with_alternatives_and_default_alternative() {
            assert_eq!(
                convert_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(Type::ByteString, "x", 42.0)],
                        Some(DefaultAlternative::new("x", 42.0))
                    )
                    .into(),
                    &vec![("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(
                            Type::ByteString,
                            "x",
                            DropVariables::new(
                                vec![("x".into(), Type::ByteString)].into_iter().collect(),
                                42.0
                            )
                        )],
                        Some(DefaultAlternative::new(
                            "x",
                            DropVariables::new(
                                vec![("x".into(), Type::Variant)].into_iter().collect(),
                                42.0
                            )
                        ))
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_case_with_moved_argument() {
            assert_eq!(
                convert_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(Type::ByteString, "x", 42.0)],
                        Some(DefaultAlternative::new("x", 42.0))
                    )
                    .into(),
                    &vec![("x".into(), Type::Variant)].into_iter().collect(),
                    &vec!["x".into()].into_iter().collect(),
                )
                .unwrap(),
                (
                    Case::new(
                        CloneVariables::new(
                            vec![("x".into(), Type::Variant)].into_iter().collect(),
                            Variable::new("x")
                        ),
                        vec![Alternative::new(
                            Type::ByteString,
                            "x",
                            DropVariables::new(
                                vec![("x".into(), Type::ByteString)].into_iter().collect(),
                                42.0
                            )
                        )],
                        Some(DefaultAlternative::new(
                            "x",
                            DropVariables::new(
                                vec![("x".into(), Type::Variant)].into_iter().collect(),
                                42.0
                            )
                        ))
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn convert_case_in_let() {
            assert_eq!(
                convert_expression(
                    &Let::new(
                        "y",
                        Type::Variant,
                        Case::new(
                            Variable::new("x"),
                            vec![],
                            Some(DefaultAlternative::new("x", Variable::new("x")))
                        ),
                        Variable::new("x")
                    )
                    .into(),
                    &vec![("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "y",
                        Type::Variant,
                        Case::new(
                            CloneVariables::new(
                                vec![("x".into(), Type::Variant)].into_iter().collect(),
                                Variable::new("x")
                            ),
                            vec![],
                            Some(DefaultAlternative::new("x", Variable::new("x")))
                        ),
                        DropVariables::new(
                            vec![("y".into(), Type::Variant)].into_iter().collect(),
                            Variable::new("x")
                        )
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                )
            );
        }

        #[test]
        fn convert_case_in_let_with_shadowed_variable() {
            assert_eq!(
                convert_expression(
                    &Let::new(
                        "x",
                        Type::Variant,
                        Case::new(
                            Variable::new("x"),
                            vec![],
                            Some(DefaultAlternative::new("x", Variable::new("x")))
                        ),
                        Variable::new("x")
                    )
                    .into(),
                    &vec![("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "x",
                        Type::Variant,
                        Case::new(
                            Variable::new("x"),
                            vec![],
                            Some(DefaultAlternative::new("x", Variable::new("x")))
                        ),
                        Variable::new("x"),
                    )
                    .into(),
                    vec!["x".into()].into_iter().collect()
                )
            );
        }
    }
}
