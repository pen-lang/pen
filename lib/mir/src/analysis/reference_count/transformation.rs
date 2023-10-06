use super::error::ReferenceCountError;
use crate::{ir::*, types::Type};
use fnv::{FnvHashMap, FnvHashSet};

// Closure environments need to be inferred before reference counting.
pub fn transform(module: &Module) -> Result<Module, ReferenceCountError> {
    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.foreign_declarations().to_vec(),
        module.foreign_definitions().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| {
                Ok(GlobalFunctionDefinition::new(
                    transform_function_definition(definition.definition(), true)?,
                    definition.is_public(),
                ))
            })
            .collect::<Result<_, _>>()?,
        module.type_information().clone(),
    ))
}

fn transform_function_definition(
    definition: &FunctionDefinition,
    global: bool,
) -> Result<FunctionDefinition, ReferenceCountError> {
    // Backend is expected to clone a function itself and its free variables at the
    // very beginning of the function.
    let owned_variables = if global {
        None
    } else {
        Some((definition.name().into(), definition.type_().clone().into()))
    }
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
        transform_expression(definition.body(), &owned_variables, &Default::default())?;

    Ok(FunctionDefinition::with_options(
        definition.name(),
        definition.environment().to_vec(),
        definition.arguments().to_vec(),
        definition.result_type().clone(),
        drop_variables(
            expression,
            owned_variables
                .keys()
                .filter(|variable| !moved_variables.contains(variable.as_str()))
                .cloned()
                .collect(),
            &owned_variables,
        ),
        definition.is_thunk(),
    ))
}

// Here, we transform expressions tracking moved variables and cloning variables
// moved already. The basic rules are listed below.
//
// - The returned values of functions are moved.
// - Every input of expressions is moved including conditions of if expressions
//   and records of record field operations.
// - Newly bound variables in let expressions are dropped if they are not moved
//   in their expressions.
fn transform_expression(
    expression: &Expression,
    owned_variables: &FnvHashMap<String, Type>,
    moved_variables: &FnvHashSet<String>,
) -> Result<(Expression, FnvHashSet<String>), ReferenceCountError> {
    Ok(match expression {
        Expression::ArithmeticOperation(operation) => {
            let (rhs, moved_variables) =
                transform_expression(operation.rhs(), owned_variables, moved_variables)?;
            let (lhs, moved_variables) =
                transform_expression(operation.lhs(), owned_variables, &moved_variables)?;

            (
                ArithmeticOperation::new(operation.operator(), lhs, rhs).into(),
                moved_variables,
            )
        }
        Expression::Case(case) => {
            let (default_alternative, default_alternative_moved_variables) =
                if let Some(alternative) = case.default_alternative() {
                    let (expression, moved_variables) = transform_expression(
                        alternative.expression(),
                        &owned_variables
                            .clone()
                            .into_iter()
                            .chain([(alternative.name().into(), Type::Variant)])
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
                    let (expression, moved_variables) = transform_expression(
                        alternative.expression(),
                        &owned_variables
                            .clone()
                            .into_iter()
                            .chain([(alternative.name().into(), alternative.type_().clone())])
                            .collect(),
                        &moved_variables
                            .clone()
                            .into_iter()
                            .filter(|variable| variable != alternative.name())
                            .collect(),
                    )?;

                    Ok((
                        Alternative::new(
                            alternative.types().to_vec(),
                            alternative.name(),
                            expression,
                        ),
                        moved_variables,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let alternative_moved_variables = default_alternative_moved_variables
                .iter()
                .filter(|&variable| {
                    if let Some(alternative) = case.default_alternative() {
                        variable != alternative.name()
                    } else {
                        true
                    }
                })
                .cloned()
                .chain(
                    alternative_tuples
                        .iter()
                        .flat_map(|(alternative, moved_variables)| {
                            moved_variables
                                .iter()
                                .filter(|&variable| variable != alternative.name())
                                .cloned()
                                .collect::<FnvHashSet<_>>()
                        }),
                )
                .collect::<FnvHashSet<_>>();

            let (argument, moved_variables) = transform_expression(
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
                                alternative.types().to_vec(),
                                alternative.name(),
                                drop_variables(
                                    alternative.expression().clone(),
                                    alternative_moved_variables
                                        .clone()
                                        .into_iter()
                                        .chain([alternative.name().into()])
                                        .collect::<FnvHashSet<_>>()
                                        .difference(&moved_variables)
                                        .cloned()
                                        .collect(),
                                    &owned_variables
                                        .clone()
                                        .into_iter()
                                        .chain([(
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
                                    .chain([alternative.name().into()])
                                    .collect::<FnvHashSet<_>>()
                                    .difference(&default_alternative_moved_variables)
                                    .cloned()
                                    .collect(),
                                &owned_variables
                                    .clone()
                                    .into_iter()
                                    .chain([(alternative.name().into(), Type::Variant)])
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
                transform_expression(operation.rhs(), owned_variables, moved_variables)?;
            let (lhs, moved_variables) =
                transform_expression(operation.lhs(), owned_variables, &moved_variables)?;

            (
                ComparisonOperation::new(operation.operator(), lhs, rhs).into(),
                moved_variables,
            )
        }
        Expression::Call(call) => {
            let (arguments, moved_variables) = call.arguments().iter().rev().try_fold(
                (vec![], moved_variables.clone()),
                |(arguments, moved_variables), argument| {
                    let (argument, moved_variables) =
                        transform_expression(argument, owned_variables, &moved_variables)?;

                    Ok((
                        [argument].into_iter().chain(arguments).collect(),
                        moved_variables,
                    ))
                },
            )?;

            let (function, moved_variables) =
                transform_expression(call.function(), owned_variables, &moved_variables)?;

            (
                Call::new(call.type_().clone(), function, arguments).into(),
                moved_variables,
            )
        }
        Expression::If(if_) => {
            let (then, then_moved_variables) =
                transform_expression(if_.then(), owned_variables, moved_variables)?;
            let (else_, else_moved_variables) =
                transform_expression(if_.else_(), owned_variables, moved_variables)?;

            let all_moved_variables = then_moved_variables
                .clone()
                .into_iter()
                .chain(else_moved_variables.clone())
                .collect();

            let (condition, moved_variables) =
                transform_expression(if_.condition(), owned_variables, &all_moved_variables)?;

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
                .chain([(let_.name().into(), let_.type_().clone())])
                .collect();
            let (expression, expression_moved_variables) = transform_expression(
                let_.expression(),
                &let_owned_variables,
                &moved_variables
                    .iter()
                    .filter(|&variable| variable != let_.name())
                    .cloned()
                    .collect(),
            )?;
            let (bound_expression, moved_variables) = transform_expression(
                let_.bound_expression(),
                owned_variables,
                &moved_variables
                    .clone()
                    .into_iter()
                    .chain(
                        expression_moved_variables
                            .iter()
                            .filter(|&variable| variable != let_.name())
                            .cloned(),
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
                            [let_.name().into()].into_iter().collect(),
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
                .chain([(
                    let_.definition().name().into(),
                    let_.definition().type_().clone().into(),
                )])
                .collect();
            let (expression, expression_moved_variables) = transform_expression(
                let_.expression(),
                &let_owned_variables,
                &moved_variables
                    .iter()
                    .filter(|&variable| variable != let_.definition().name())
                    .cloned()
                    .collect(),
            )?;
            let moved_variables = moved_variables
                .clone()
                .into_iter()
                .chain(
                    expression_moved_variables
                        .iter()
                        .filter(|&variable| variable != let_.definition().name())
                        .cloned(),
                )
                .collect();
            let cloned_variables = let_
                .definition()
                .environment()
                .iter()
                .map(|argument| argument.name().into())
                .collect::<FnvHashSet<_>>()
                .intersection(&moved_variables)
                .cloned()
                .collect::<FnvHashSet<_>>();

            (
                clone_variables(
                    LetRecursive::new(
                        transform_function_definition(let_.definition(), false)?,
                        if expression_moved_variables.contains(let_.definition().name()) {
                            expression
                        } else {
                            drop_variables(
                                expression,
                                [let_.definition().name().into()].into_iter().collect(),
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
                    .collect::<FnvHashSet<String>>(),
            )
        }
        Expression::Synchronize(synchronize) => {
            let (expression, moved_variables) =
                transform_expression(synchronize.expression(), owned_variables, moved_variables)?;

            (
                Synchronize::new(synchronize.type_().clone(), expression).into(),
                moved_variables,
            )
        }
        Expression::Record(record) => {
            let (fields, moved_variables) = record.fields().iter().rev().try_fold(
                (vec![], moved_variables.clone()),
                |(fields, moved_variables), field| {
                    let (field, moved_variables) =
                        transform_expression(field, owned_variables, &moved_variables)?;

                    Ok(([field].into_iter().chain(fields).collect(), moved_variables))
                },
            )?;

            (
                Record::new(record.type_().clone(), fields).into(),
                moved_variables,
            )
        }
        Expression::RecordField(field) => {
            let (record, moved_variables) =
                transform_expression(field.record(), owned_variables, moved_variables)?;

            (
                RecordField::new(field.type_().clone(), field.index(), record).into(),
                moved_variables,
            )
        }
        Expression::RecordUpdate(update) => {
            let (record, mut moved_variables) =
                transform_expression(update.record(), owned_variables, moved_variables)?;
            let mut fields = vec![];

            for field in update.fields() {
                let (expression, variables) =
                    transform_expression(field.expression(), owned_variables, &moved_variables)?;

                fields.push(RecordUpdateField::new(field.index(), expression));
                moved_variables = variables;
            }

            (
                RecordUpdate::new(update.type_().clone(), record, fields).into(),
                moved_variables,
            )
        }
        Expression::StringConcatenation(concatenation) => {
            let mut operands = vec![];
            let mut moved_variables = moved_variables.clone();

            for operand in concatenation.operands() {
                let (expression, variables) =
                    transform_expression(operand, owned_variables, &moved_variables)?;

                operands.push(expression);
                moved_variables = variables;
            }

            (StringConcatenation::new(operands).into(), moved_variables)
        }
        Expression::TryOperation(operation) => {
            let (then, then_moved_variables) =
                transform_expression(operation.then(), owned_variables, &Default::default())?;
            let then_moved_variables = then_moved_variables
                .into_iter()
                .filter(|name| name != operation.name())
                .collect::<FnvHashSet<_>>();

            let all_moved_variables = then_moved_variables
                .clone()
                .into_iter()
                .chain(moved_variables.clone())
                .collect();

            let (operand, operand_moved_variables) =
                transform_expression(operation.operand(), owned_variables, &all_moved_variables)?;

            (
                drop_variables(
                    TryOperation::new(
                        operand,
                        operation.name(),
                        operation.type_().clone(),
                        drop_variables(
                            then,
                            all_moved_variables
                                .difference(&then_moved_variables)
                                .cloned()
                                .collect(),
                            owned_variables,
                        ),
                    ),
                    all_moved_variables
                        .difference(moved_variables)
                        .cloned()
                        .collect(),
                    owned_variables,
                ),
                operand_moved_variables,
            )
        }
        Expression::TypeInformationFunction(information) => {
            let (expression, moved_variables) =
                transform_expression(information.variant(), owned_variables, moved_variables)?;

            (
                TypeInformationFunction::new(expression).into(),
                moved_variables,
            )
        }
        Expression::Variable(variable) => {
            if should_clone_variable(variable.name(), owned_variables, moved_variables) {
                (
                    clone_variables(
                        variable.clone(),
                        [variable.name().into()].into_iter().collect(),
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
                        .chain([variable.name().into()])
                        .collect(),
                )
            }
        }
        Expression::Variant(variant) => {
            let (expression, moved_variables) =
                transform_expression(variant.payload(), owned_variables, moved_variables)?;

            (
                Variant::new(variant.type_().clone(), expression).into(),
                moved_variables,
            )
        }
        Expression::Boolean(_)
        | Expression::ByteString(_)
        | Expression::None
        | Expression::Number(_) => (expression.clone(), moved_variables.clone()),
        Expression::CloneVariables(_) | Expression::DropVariables(_) => {
            return Err(ReferenceCountError::ExpressionNotSupported(
                expression.clone(),
            ));
        }
    })
}

fn clone_variables(
    expression: impl Into<Expression>,
    cloned_variables: FnvHashSet<String>,
    owned_variables: &FnvHashMap<String, Type>,
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
    dropped_variables: FnvHashSet<String>,
    owned_variables: &FnvHashMap<String, Type>,
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
    owned_variables: &FnvHashMap<String, Type>,
    moved_variables: &FnvHashSet<String>,
) -> bool {
    owned_variables.contains_key(variable) && moved_variables.contains(variable)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::FunctionDefinitionFake,
        types::{self, Type},
    };

    #[test]
    fn transform_record() {
        assert_eq!(
            transform_expression(
                &Record::new(
                    types::Record::new("a"),
                    vec![Variable::new("x").into(), Variable::new("x").into()]
                )
                .into(),
                &[("x".into(), Type::Number)].into_iter().collect(),
                &Default::default()
            )
            .unwrap(),
            (
                Record::new(
                    types::Record::new("a"),
                    vec![
                        CloneVariables::new(
                            [("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        )
                        .into(),
                        Variable::new("x").into()
                    ]
                )
                .into(),
                ["x".into()].into_iter().collect()
            ),
        );
    }

    #[test]
    fn transform_type_information() {
        assert_eq!(
            transform_expression(
                &TypeInformationFunction::new(Variable::new("x")).into(),
                &[("x".into(), Type::Variant)].into_iter().collect(),
                &Default::default()
            )
            .unwrap(),
            (
                TypeInformationFunction::new(Variable::new("x")).into(),
                ["x".into()].into_iter().collect()
            ),
        );
    }

    mod call {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_single() {
            assert_eq!(
                transform_expression(
                    &Call::new(
                        types::Function::new(vec![Type::Number], Type::Number),
                        Variable::new("f"),
                        vec![Variable::new("x").into()]
                    )
                    .into(),
                    &[
                        (
                            "f".into(),
                            types::Function::new(vec![Type::Number], Type::Number).into()
                        ),
                        ("x".into(), Type::Number)
                    ]
                    .into_iter()
                    .collect(),
                    &["f".into(), "x".into()].into_iter().collect(),
                )
                .unwrap(),
                (
                    Call::new(
                        types::Function::new(vec![Type::Number], Type::Number),
                        CloneVariables::new(
                            [(
                                "f".into(),
                                types::Function::new(vec![Type::Number], Type::Number).into()
                            )]
                            .into_iter()
                            .collect(),
                            Variable::new("f")
                        ),
                        vec![CloneVariables::new(
                            [("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        )
                        .into()]
                    )
                    .into(),
                    ["f".into(), "x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_nested() {
            assert_eq!(
                transform_expression(
                    &Call::new(
                        types::Function::new(vec![Type::Number], Type::Number),
                        Call::new(
                            types::Function::new(
                                vec![Type::Number],
                                types::Function::new(vec![Type::Number], Type::Number)
                            ),
                            Variable::new("f"),
                            vec![Variable::new("x").into()]
                        ),
                        vec![Variable::new("x").into()]
                    )
                    .into(),
                    &[
                        (
                            "f".into(),
                            types::Function::new(
                                vec![Type::Number],
                                types::Function::new(vec![Type::Number], Type::Number)
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
                    Call::new(
                        types::Function::new(vec![Type::Number], Type::Number),
                        Call::new(
                            types::Function::new(
                                vec![Type::Number],
                                types::Function::new(vec![Type::Number], Type::Number)
                            ),
                            Variable::new("f"),
                            vec![CloneVariables::new(
                                [("x".into(), Type::Number)].into_iter().collect(),
                                Variable::new("x")
                            )
                            .into()]
                        ),
                        vec![Variable::new("x").into()]
                    )
                    .into(),
                    ["f".into(), "x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_2_arguments() {
            assert_eq!(
                transform_expression(
                    &Call::new(
                        types::Function::new(vec![Type::Number, Type::Boolean], Type::Number),
                        Variable::new("f"),
                        vec![Variable::new("x").into(), Variable::new("y").into()]
                    )
                    .into(),
                    &[
                        (
                            "f".into(),
                            types::Function::new(vec![Type::Number, Type::Boolean], Type::Number)
                                .into()
                        ),
                        ("x".into(), Type::Number),
                        ("y".into(), Type::Boolean),
                    ]
                    .into_iter()
                    .collect(),
                    &["f".into(), "x".into(), "y".into()].into_iter().collect(),
                )
                .unwrap(),
                (
                    Call::new(
                        types::Function::new(vec![Type::Number, Type::Boolean], Type::Number),
                        CloneVariables::new(
                            [(
                                "f".into(),
                                types::Function::new(
                                    vec![Type::Number, Type::Boolean],
                                    Type::Number
                                )
                                .into()
                            )]
                            .into_iter()
                            .collect(),
                            Variable::new("f")
                        ),
                        vec![
                            CloneVariables::new(
                                [("x".into(), Type::Number)].into_iter().collect(),
                                Variable::new("x")
                            )
                            .into(),
                            CloneVariables::new(
                                [("y".into(), Type::Boolean)].into_iter().collect(),
                                Variable::new("y")
                            )
                            .into()
                        ]
                    )
                    .into(),
                    ["x".into(), "y".into(), "f".into(),].into_iter().collect()
                ),
            );
        }
    }

    mod let_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_with_moved_variable() {
            assert_eq!(
                transform_expression(
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
        fn transform_with_cloned_variable() {
            assert_eq!(
                transform_expression(
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
                            [("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        ),
                        Variable::new("x")
                    ),
                )
                .into(),
            );
        }

        #[test]
        fn transform_with_dropped_variable() {
            assert_eq!(
                transform_expression(
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
                    DropVariables::new([("x".into(), Type::Number)].into_iter().collect(), 42.0)
                )
                .into(),
            );
        }

        #[test]
        fn transform_with_moved_variable_in_bound_expression() {
            assert_eq!(
                transform_expression(
                    &Let::new("x", Type::Number, Variable::new("y"), Variable::new("x")).into(),
                    &[("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new("x", Type::Number, Variable::new("y"), Variable::new("x")).into(),
                    ["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_with_cloned_variable_in_bound_expression() {
            assert_eq!(
                transform_expression(
                    &Let::new("x", Type::Number, Variable::new("y"), Variable::new("y")).into(),
                    &[("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "x",
                        Type::Number,
                        CloneVariables::new(
                            [("y".into(), Type::Number)].into_iter().collect(),
                            Variable::new("y")
                        ),
                        DropVariables::new(
                            [("x".into(), Type::Number)].into_iter().collect(),
                            Variable::new("y")
                        )
                    )
                    .into(),
                    ["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_nested_let() {
            assert_eq!(
                transform_expression(
                    &Let::new(
                        "y",
                        Type::Number,
                        Let::new("x", Type::Number, Variable::new("x"), Variable::new("x")),
                        Variable::new("x")
                    )
                    .into(),
                    &[("x".into(), Type::Number)].into_iter().collect(),
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
                                [("x".into(), Type::Number)].into_iter().collect(),
                                Variable::new("x")
                            ),
                            Variable::new("x")
                        ),
                        DropVariables::new(
                            [("y".into(), Type::Number)].into_iter().collect(),
                            Variable::new("x")
                        )
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                )
            );
        }
    }

    mod let_recursive {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_with_moved_variable() {
            assert_eq!(
                transform_expression(
                    &LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            42.0
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
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        DropVariables::new(
                            [
                                (
                                    "f".into(),
                                    types::Function::new(vec![Type::Number], Type::Number).into()
                                ),
                                ("x".into(), Type::Number)
                            ]
                            .into_iter()
                            .collect(),
                            42.0,
                        )
                    ),
                    Variable::new("f")
                )
                .into(),
            );
        }

        #[test]
        fn transform_with_cloned_variable() {
            let f_type = types::Function::new(vec![Type::Number], Type::Number);
            let g_type = types::Function::new(
                vec![types::Function::new(vec![Type::Number], Type::Number).into()],
                types::Function::new(
                    vec![types::Function::new(vec![Type::Number], Type::Number).into()],
                    Type::Number,
                ),
            );

            assert_eq!(
                transform_expression(
                    &LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            42.0
                        ),
                        Call::new(
                            f_type.clone(),
                            Call::new(
                                g_type.clone(),
                                Variable::new("g"),
                                vec![Variable::new("f").into()]
                            ),
                            vec![Variable::new("f").into()]
                        )
                    )
                    .into(),
                    &Default::default(),
                    &Default::default()
                )
                .unwrap()
                .0,
                LetRecursive::new(
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        DropVariables::new(
                            [
                                (
                                    "f".into(),
                                    types::Function::new(vec![Type::Number], Type::Number).into()
                                ),
                                ("x".into(), Type::Number)
                            ]
                            .into_iter()
                            .collect(),
                            42.0,
                        )
                    ),
                    Call::new(
                        f_type,
                        Call::new(
                            g_type,
                            Variable::new("g"),
                            vec![CloneVariables::new(
                                [(
                                    "f".into(),
                                    types::Function::new(vec![Type::Number], Type::Number).into()
                                )]
                                .into_iter()
                                .collect(),
                                Variable::new("f")
                            )
                            .into()]
                        ),
                        vec![Variable::new("f").into()]
                    )
                )
                .into(),
            );
        }

        #[test]
        fn transform_with_dropped_variable() {
            assert_eq!(
                transform_expression(
                    &LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            42.0
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
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        DropVariables::new(
                            [
                                (
                                    "f".into(),
                                    types::Function::new(vec![Type::Number], Type::Number).into()
                                ),
                                ("x".into(), Type::Number),
                            ]
                            .into_iter()
                            .collect(),
                            42.0,
                        )
                    ),
                    DropVariables::new(
                        [(
                            "f".into(),
                            types::Function::new(vec![Type::Number], Type::Number).into()
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
        fn transform_with_moved_variable_in_environment() {
            assert_eq!(
                transform_expression(
                    &LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            42.0
                        )
                        .set_environment(vec![Argument::new("y", Type::Number)]),
                        Variable::new("f")
                    )
                    .into(),
                    &[("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            DropVariables::new(
                                [
                                    (
                                        "f".into(),
                                        types::Function::new(vec![Type::Number], Type::Number)
                                            .into()
                                    ),
                                    ("x".into(), Type::Number),
                                    ("y".into(), Type::Number)
                                ]
                                .into_iter()
                                .collect(),
                                42.0,
                            )
                        )
                        .set_environment(vec![Argument::new("y", Type::Number)]),
                        Variable::new("f")
                    )
                    .into(),
                    ["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_with_cloned_variable_in_environment() {
            assert_eq!(
                transform_expression(
                    &LetRecursive::new(
                        FunctionDefinition::new(
                            "f",
                            vec![Argument::new("x", Type::Number)],
                            Type::Number,
                            42.0
                        )
                        .set_environment(vec![Argument::new("y", Type::Number)]),
                        Call::new(
                            types::Function::new(vec![Type::Number], Type::Number),
                            Variable::new("f"),
                            vec![Variable::new("y").into()]
                        )
                    )
                    .into(),
                    &[("y".into(), Type::Number)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    CloneVariables::new(
                        [("y".into(), Type::Number)].into_iter().collect(),
                        LetRecursive::new(
                            FunctionDefinition::new(
                                "f",
                                vec![Argument::new("x", Type::Number)],
                                Type::Number,
                                DropVariables::new(
                                    [
                                        (
                                            "f".into(),
                                            types::Function::new(vec![Type::Number], Type::Number)
                                                .into()
                                        ),
                                        ("x".into(), Type::Number),
                                        ("y".into(), Type::Number),
                                    ]
                                    .into_iter()
                                    .collect(),
                                    42.0,
                                )
                            )
                            .set_environment(vec![Argument::new("y", Type::Number)]),
                            Call::new(
                                types::Function::new(vec![Type::Number], Type::Number),
                                Variable::new("f"),
                                vec![Variable::new("y").into()]
                            )
                        )
                    )
                    .into(),
                    ["y".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_let_recursive_in_let() {
            let function_type = types::Function::new(vec![Type::Number], Type::Number);

            assert_eq!(
                transform_expression(
                    &Let::new(
                        "g",
                        function_type.clone(),
                        LetRecursive::new(
                            FunctionDefinition::new(
                                "f",
                                vec![Argument::new("x", Type::Number)],
                                Type::Number,
                                Call::new(
                                    function_type.clone(),
                                    Variable::new("f"),
                                    vec![Variable::new("x").into()]
                                )
                            )
                            .set_environment(vec![Argument::new("f", Type::Number)]),
                            Variable::new("f")
                        ),
                        Variable::new("f")
                    )
                    .into(),
                    &[("f".into(), function_type.clone().into())]
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
                            [("f".into(), function_type.clone().into())]
                                .into_iter()
                                .collect(),
                            LetRecursive::new(
                                FunctionDefinition::new(
                                    "f",
                                    vec![Argument::new("x", Type::Number)],
                                    Type::Number,
                                    Call::new(
                                        function_type.clone(),
                                        Variable::new("f"),
                                        vec![Variable::new("x").into()]
                                    )
                                )
                                .set_environment(vec![Argument::new("f", Type::Number)]),
                                Variable::new("f")
                            )
                        ),
                        DropVariables::new(
                            [("g".into(), function_type.into())].into_iter().collect(),
                            Variable::new("f")
                        )
                    )
                    .into(),
                    ["f".into()].into_iter().collect()
                )
            );
        }

        #[test]
        fn transform_with_moved_free_variable() {
            assert_eq!(
                transform_expression(
                    &LetRecursive::new(
                        FunctionDefinition::new(
                            "x",
                            vec![Argument::new("y", Type::Number)],
                            Type::Number,
                            42.0
                        )
                        .set_environment(vec![Argument::new("x", Type::Number)]),
                        42.0
                    )
                    .into(),
                    &[("x".into(), Type::Number)].into_iter().collect(),
                    &["x".into()].into_iter().collect()
                )
                .unwrap(),
                (
                    CloneVariables::new(
                        [("x".into(), Type::Number)].into_iter().collect(),
                        LetRecursive::new(
                            FunctionDefinition::new(
                                "x",
                                vec![Argument::new("y", Type::Number)],
                                Type::Number,
                                DropVariables::new(
                                    [("y".into(), Type::Number), ("x".into(), Type::Number)]
                                        .into_iter()
                                        .collect(),
                                    42.0,
                                )
                            )
                            .set_environment(vec![Argument::new("x", Type::Number)]),
                            DropVariables::new(
                                [(
                                    "x".into(),
                                    types::Function::new(vec![Type::Number], Type::Number).into()
                                )]
                                .into_iter()
                                .collect(),
                                42.0,
                            )
                        )
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                )
            );
        }
    }

    mod definition {

        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_with_dropped_argument() {
            assert_eq!(
                transform_function_definition(
                    &FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        42.0
                    ),
                    false
                )
                .unwrap(),
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Type::Number,
                    DropVariables::new(
                        [
                            (
                                "f".into(),
                                types::Function::new(vec![Type::Number], Type::Number).into()
                            ),
                            ("x".into(), Type::Number)
                        ]
                        .into_iter()
                        .collect(),
                        42.0
                    )
                ),
            );
        }

        #[test]
        fn transform_with_dropped_free_variable() {
            assert_eq!(
                transform_function_definition(
                    &FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        Type::Number,
                        42.0
                    )
                    .set_environment(vec![Argument::new("y", Type::Number)]),
                    false
                )
                .unwrap(),
                FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Type::Number,
                    DropVariables::new(
                        [
                            (
                                "f".into(),
                                types::Function::new(vec![Type::Number], Type::Number).into()
                            ),
                            ("x".into(), Type::Number),
                            ("y".into(), Type::Number)
                        ]
                        .into_iter()
                        .collect(),
                        42.0
                    )
                )
                .set_environment(vec![Argument::new("y", Type::Number)]),
            );
        }
    }

    mod if_ {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_if() {
            assert_eq!(
                transform_expression(
                    &If::new(Variable::new("x"), Variable::new("y"), Variable::new("z")).into(),
                    &[
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
                            [("z".into(), Type::Number)].into_iter().collect(),
                            Variable::new("y")
                        ),
                        DropVariables::new(
                            [("y".into(), Type::Number)].into_iter().collect(),
                            Variable::new("z")
                        )
                    )
                    .into(),
                    ["x".into(), "y".into(), "z".into()].into_iter().collect()
                ),
            );
        }
    }

    mod case {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_case_with_default_alternative() {
            assert_eq!(
                transform_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![],
                        Some(DefaultAlternative::new("x", 42.0))
                    )
                    .into(),
                    &[("x".into(), Type::Variant)].into_iter().collect(),
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
                                [("x".into(), Type::Variant)].into_iter().collect(),
                                42.0
                            )
                        ))
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_case_with_alternatives() {
            assert_eq!(
                transform_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![
                            Alternative::new(vec![Type::Number], "x", Variable::new("x")),
                            Alternative::new(vec![Type::Boolean], "x", 42.0)
                        ],
                        None
                    )
                    .into(),
                    &[("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Case::new(
                        Variable::new("x"),
                        vec![
                            Alternative::new(vec![Type::Number], "x", Variable::new("x")),
                            Alternative::new(
                                vec![Type::Boolean],
                                "x",
                                DropVariables::new(
                                    [("x".into(), Type::Boolean)].into_iter().collect(),
                                    42.0
                                )
                            )
                        ],
                        None
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_case_with_alternatives_and_default_alternative() {
            assert_eq!(
                transform_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(vec![Type::ByteString], "x", 42.0)],
                        Some(DefaultAlternative::new("x", 42.0))
                    )
                    .into(),
                    &[("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(
                            vec![Type::ByteString],
                            "x",
                            DropVariables::new(
                                [("x".into(), Type::ByteString)].into_iter().collect(),
                                42.0
                            )
                        )],
                        Some(DefaultAlternative::new(
                            "x",
                            DropVariables::new(
                                [("x".into(), Type::Variant)].into_iter().collect(),
                                42.0
                            )
                        ))
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_case_with_moved_argument() {
            assert_eq!(
                transform_expression(
                    &Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(vec![Type::ByteString], "x", 42.0)],
                        Some(DefaultAlternative::new("x", 42.0))
                    )
                    .into(),
                    &[("x".into(), Type::Variant)].into_iter().collect(),
                    &["x".into()].into_iter().collect(),
                )
                .unwrap(),
                (
                    Case::new(
                        CloneVariables::new(
                            [("x".into(), Type::Variant)].into_iter().collect(),
                            Variable::new("x")
                        ),
                        vec![Alternative::new(
                            vec![Type::ByteString],
                            "x",
                            DropVariables::new(
                                [("x".into(), Type::ByteString)].into_iter().collect(),
                                42.0
                            )
                        )],
                        Some(DefaultAlternative::new(
                            "x",
                            DropVariables::new(
                                [("x".into(), Type::Variant)].into_iter().collect(),
                                42.0
                            )
                        ))
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                ),
            );
        }

        #[test]
        fn transform_case_in_let() {
            assert_eq!(
                transform_expression(
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
                    &[("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    Let::new(
                        "y",
                        Type::Variant,
                        Case::new(
                            CloneVariables::new(
                                [("x".into(), Type::Variant)].into_iter().collect(),
                                Variable::new("x")
                            ),
                            vec![],
                            Some(DefaultAlternative::new("x", Variable::new("x")))
                        ),
                        DropVariables::new(
                            [("y".into(), Type::Variant)].into_iter().collect(),
                            Variable::new("x")
                        )
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                )
            );
        }

        #[test]
        fn transform_case_in_let_with_shadowed_variable() {
            assert_eq!(
                transform_expression(
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
                    &[("x".into(), Type::Variant)].into_iter().collect(),
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
                    ["x".into()].into_iter().collect()
                )
            );
        }
    }

    mod try_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn transform_try_operation() {
            assert_eq!(
                transform_expression(
                    &TryOperation::new(
                        Variable::new("x"),
                        "y",
                        types::Type::Number,
                        Variant::new(types::Type::Number, Variable::new("y")),
                    )
                    .into(),
                    &[("x".into(), Type::Variant)].into_iter().collect(),
                    &Default::default()
                )
                .unwrap(),
                (
                    TryOperation::new(
                        Variable::new("x"),
                        "y",
                        types::Type::Number,
                        Variant::new(types::Type::Number, Variable::new("y"),),
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                )
            );
        }

        #[test]
        fn transform_try_operation_with_moved_operand() {
            assert_eq!(
                transform_expression(
                    &TryOperation::new(
                        Variable::new("x"),
                        "y",
                        types::Type::Number,
                        Variant::new(types::Type::Number, Variable::new("y")),
                    )
                    .into(),
                    &[("x".into(), Type::Variant)].into_iter().collect(),
                    &["x".into()].into_iter().collect(),
                )
                .unwrap(),
                (
                    TryOperation::new(
                        CloneVariables::new(
                            [("x".into(), Type::Variant)].into_iter().collect(),
                            Variable::new("x")
                        ),
                        "y",
                        types::Type::Number,
                        DropVariables::new(
                            [("x".into(), Type::Variant)].into_iter().collect(),
                            Variant::new(types::Type::Number, Variable::new("y"),)
                        ),
                    )
                    .into(),
                    ["x".into()].into_iter().collect()
                )
            );
        }
    }
}
