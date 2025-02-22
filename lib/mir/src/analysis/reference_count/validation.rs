use super::ReferenceCountError;
use crate::ir::*;
use fnv::{FnvHashMap, FnvHashSet};

pub fn validate(module: &Module) -> Result<(), ReferenceCountError> {
    for definition in module.function_definitions() {
        validate_global_function_definition(definition)?;
    }

    Ok(())
}

fn validate_global_function_definition(
    definition: &GlobalFunctionDefinition,
) -> Result<(), ReferenceCountError> {
    let definition = definition.definition();

    validate_definition_body(
        definition.body(),
        collect_definition_local_variables(definition)
            .into_iter()
            .map(|name| (name, 1))
            .collect(),
    )
}

fn validate_local_definition(definition: &FunctionDefinition) -> Result<(), ReferenceCountError> {
    validate_definition_body(
        definition.body(),
        [definition.name().into()]
            .into_iter()
            .chain(collect_definition_local_variables(definition))
            .map(|name| (name, 1))
            .collect(),
    )
}

fn collect_definition_local_variables(definition: &FunctionDefinition) -> FnvHashSet<String> {
    definition
        .environment()
        .iter()
        .chain(definition.arguments())
        .map(|argument| argument.name().into())
        .collect()
}

fn validate_definition_body(
    body: &Expression,
    mut variables: FnvHashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    move_expression(body, &mut variables)?;

    let invalid_variables = variables
        .into_iter()
        .filter(|(_, count)| count != &0)
        .collect::<FnvHashMap<_, _>>();

    if !invalid_variables.is_empty() {
        return Err(ReferenceCountError::InvalidLocalVariables(
            invalid_variables,
        ));
    }

    Ok(())
}

fn move_expression(
    expression: &Expression,
    variables: &mut FnvHashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    match expression {
        Expression::ArithmeticOperation(operation) => {
            move_expression(operation.lhs(), variables)?;
            move_expression(operation.rhs(), variables)?;
        }
        Expression::Boolean(_) => {}
        Expression::ByteString(_) => {}
        Expression::Call(call) => {
            move_expression(call.function(), variables)?;

            for argument in call.arguments() {
                move_expression(argument, variables)?;
            }
        }
        Expression::Case(case) => {
            move_expression(case.argument(), variables)?;

            let old_variables = variables.clone();

            if let Some(alternative) = case.default_alternative() {
                validate_let_like(alternative.name(), alternative.expression(), variables)?;
            } else {
                let alternative = &case.alternatives()[0];

                validate_let_like(alternative.name(), alternative.expression(), variables)?;
            }

            for alternative in case.alternatives() {
                let mut alternative_variables = old_variables.clone();

                validate_let_like(
                    alternative.name(),
                    alternative.expression(),
                    &mut alternative_variables,
                )?;

                validate_conditional_variables(variables, &alternative_variables)?
            }
        }
        Expression::CloneVariables(clone) => {
            for name in clone.variables().keys() {
                clone_variable(name, variables);
            }

            move_expression(clone.expression(), variables)?;
        }
        Expression::ComparisonOperation(operation) => {
            move_expression(operation.lhs(), variables)?;
            move_expression(operation.rhs(), variables)?;
        }
        Expression::DropVariables(drop) => move_drop_variables(drop, variables)?,
        Expression::If(if_) => {
            move_expression(if_.condition(), variables)?;

            let mut then_variables = variables.clone();
            move_expression(if_.then(), &mut then_variables)?;

            move_expression(if_.else_(), variables)?;

            validate_conditional_variables(variables, &then_variables)?
        }
        Expression::Let(let_) => {
            move_expression(let_.bound_expression(), variables)?;
            validate_let_like(let_.name(), let_.expression(), variables)?;
        }
        Expression::LetRecursive(let_) => {
            validate_local_definition(let_.definition())?;

            for free_variable in let_.definition().environment() {
                drop_variable(free_variable.name(), variables);
            }

            validate_let_like(let_.definition().name(), let_.expression(), variables)?;
        }
        Expression::Synchronize(synchronize) => {
            move_expression(synchronize.expression(), variables)?;
        }
        Expression::None => {}
        Expression::Number(_) => {}
        Expression::Record(record) => {
            move_record(record, variables)?;
        }
        Expression::RecordField(field) => {
            move_expression(field.record(), variables)?;
        }
        Expression::RecordUpdate(update) => {
            move_expression(update.record(), variables)?;

            for field in update.fields() {
                move_expression(field.expression(), variables)?;
            }
        }
        Expression::StringConcatenation(concatenation) => {
            for operand in concatenation.operands() {
                move_expression(operand, variables)?;
            }
        }
        Expression::TryOperation(operation) => {
            move_expression(operation.operand(), variables)?;

            let mut variables = variables.clone();

            validate_let_like(operation.name(), operation.then(), &mut variables)?;

            if !variables.values().all(|&count| count == 0) {
                return Err(ReferenceCountError::InvalidLocalVariables(
                    variables.into_iter().collect(),
                ));
            }
        }
        Expression::TypeInformationFunction(information) => {
            move_expression(information.variant(), variables)?;
        }
        Expression::Variable(variable) => {
            drop_variable(variable.name(), variables);
        }
        Expression::Variant(variant) => {
            move_expression(variant.payload(), variables)?;
        }
    }

    Ok(())
}

fn move_drop_variables(
    drop: &DropVariables,
    variables: &mut FnvHashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    for name in drop.variables().keys() {
        drop_variable(name, variables);
    }

    move_expression(drop.expression(), variables)
}

fn move_record(
    record: &Record,
    variables: &mut FnvHashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    for field in record.fields() {
        move_expression(field, variables)?;
    }

    Ok(())
}

fn validate_let_like(
    name: &str,
    expression: &Expression,
    variables: &mut FnvHashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    let old_count = variables.insert(name.into(), 1);

    move_expression(expression, variables)?;

    if variables[name] != 0 {
        return Err(ReferenceCountError::InvalidLocalVariable(
            name.into(),
            variables[name],
        ));
    } else if let Some(old) = old_count {
        variables.insert(name.into(), old);
    }

    Ok(())
}

fn validate_conditional_variables(
    then_variables: &FnvHashMap<String, isize>,
    else_variables: &FnvHashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    let then_variables = filter_valid_variables(then_variables);
    let else_variables = filter_valid_variables(else_variables);

    if then_variables != else_variables {
        return Err(ReferenceCountError::UnmatchedVariables(
            then_variables,
            else_variables,
        ));
    }

    Ok(())
}

fn filter_valid_variables(variables: &FnvHashMap<String, isize>) -> FnvHashMap<String, isize> {
    variables
        .iter()
        .filter(|(_, count)| **count != 0)
        .map(|(name, &count)| (name.clone(), count))
        .collect()
}

fn clone_variable(name: impl AsRef<str>, variables: &mut FnvHashMap<String, isize>) {
    let name = name.as_ref();

    if let Some(count) = variables.get(name).cloned() {
        variables.insert(name.into(), count + 1);
    }
}

fn drop_variable(name: impl AsRef<str>, variables: &mut FnvHashMap<String, isize>) {
    let name = name.as_ref();

    if let Some(count) = variables.get(name).cloned() {
        variables.insert(name.into(), count - 1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        test::{FunctionDefinitionFake, ModuleFake},
        types::{self, Type},
    };

    #[test]
    fn validate_empty_module() {
        validate(&Module::empty()).unwrap();
    }

    #[test]
    fn validate_none() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![],
                Type::None,
                Expression::None,
            )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_variable_clone() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Type::None,
                CloneVariables::new(
                    [("x".into(), Type::Number)].into_iter().collect(),
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Variable::new("x"),
                        Variable::new("x"),
                    ),
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_variable_drop() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Type::None,
                DropVariables::new(
                    [("x".into(), Type::None)].into_iter().collect(),
                    Expression::None,
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_variable_drop() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::None)],
                    Type::None,
                    Expression::None
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariables(
                [("x".into(), 1)].into_iter().collect()
            ))
        );
    }

    #[test]
    fn validate_variable_move() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Type::None,
                Variable::new("x"),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_let() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Type::None,
                Let::new("y", Type::None, Variable::new("x"), Variable::new("y")),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_let_with_leaked_bound_variable() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::None)],
                    Type::None,
                    Let::new("y", Type::None, Variable::new("x"), Expression::None)
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("y".into(), 1))
        );
    }

    #[test]
    fn validate_let_with_shadowed_variable() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Type::None,
                Let::new("x", Type::None, Variable::new("x"), Variable::new("x")),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_let_with_shadowed_variable() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::None)],
                    Type::None,
                    Let::new("x", Type::None, Variable::new("x"), Expression::None)
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("x".into(), 1))
        );
    }

    #[test]
    fn validate_let_recursive() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Type::None,
                LetRecursive::new(
                    FunctionDefinition::new(
                        "g",
                        vec![],
                        Type::None,
                        DropVariables::new(
                            [("g".into(), types::Function::new(vec![], Type::None).into())]
                                .into_iter()
                                .collect(),
                            Variable::new("x"),
                        ),
                    )
                    .set_environment(vec![Argument::new("x", Type::None)]),
                    Variable::new("g"),
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_let_recursive_with_leaked_function() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::None)],
                    Type::None,
                    LetRecursive::new(
                        FunctionDefinition::new(
                            "g",
                            vec![],
                            Type::None,
                            DropVariables::new(
                                [("g".into(), types::Function::new(vec![], Type::None).into())]
                                    .into_iter()
                                    .collect(),
                                Variable::new("x"),
                            )
                        )
                        .set_environment(vec![Argument::new("x", Type::None)]),
                        Expression::None,
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("g".into(), 1))
        );
    }

    #[test]
    fn fail_to_validate_definition_in_let_recursive() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::None)],
                    Type::None,
                    LetRecursive::new(
                        FunctionDefinition::with_options(
                            "g",
                            vec![Argument::new("x", Type::None)],
                            vec![],
                            Type::None,
                            Variable::new("x"),
                            false,
                        ),
                        Variable::new("g"),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariables(
                [("g".into(), 1)].into_iter().collect()
            ))
        );
    }

    #[test]
    fn validate_if() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Type::None,
                If::new(
                    Expression::Boolean(true),
                    Variable::new("x"),
                    Variable::new("x"),
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_if() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::None)],
                    Type::None,
                    If::new(
                        Expression::Boolean(true),
                        Variable::new("x"),
                        Expression::None,
                    )
                )],)
            ),
            Err(ReferenceCountError::UnmatchedVariables(
                [("x".into(), 1)].into_iter().collect(),
                Default::default(),
            ))
        );
    }

    #[test]
    fn validate_case_with_default_alternative() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Type::Variant,
                Case::new(
                    Variable::new("x"),
                    vec![],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_case_with_default_alternative() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    Case::new(
                        Variable::new("x"),
                        vec![],
                        Some(DefaultAlternative::new("y", Expression::None)),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("y".into(), 1))
        );
    }

    #[test]
    fn validate_case_with_alternative() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Type::Variant,
                Case::new(
                    Variable::new("x"),
                    vec![Alternative::new(vec![Type::None], "y", Variable::new("y"))],
                    None,
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_case_with_alternative() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(vec![Type::None], "y", Expression::None)],
                        None,
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("y".into(), 1))
        );
    }

    #[test]
    fn validate_case_with_alternative_and_default_alternative() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Type::Variant,
                Case::new(
                    Variable::new("x"),
                    vec![Alternative::new(vec![Type::None], "y", Variable::new("y"))],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_case_with_alternative_and_default_alternative() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    Case::new(
                        Variable::new("x"),
                        vec![Alternative::new(vec![Type::None], "y", Expression::None)],
                        Some(DefaultAlternative::new("y", Variable::new("y"))),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("y".into(), 1))
        );
    }

    #[test]
    fn validate_case_with_two_alternatives_and_default_alternative() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Type::Variant,
                Case::new(
                    Variable::new("x"),
                    vec![
                        Alternative::new(vec![Type::None], "y", Variable::new("y")),
                        Alternative::new(vec![Type::None], "y", Variable::new("y")),
                    ],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_case_with_two_alternatives_and_default_alternative() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    Case::new(
                        Variable::new("x"),
                        vec![
                            Alternative::new(vec![Type::None], "y", Variable::new("y")),
                            Alternative::new(vec![Type::None], "y", Expression::None),
                        ],
                        Some(DefaultAlternative::new("y", Variable::new("y"))),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("y".into(), 1))
        );
    }

    #[test]
    fn validate_case_with_different_bound_variables() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Type::Variant,
                Case::new(
                    Variable::new("x"),
                    vec![
                        Alternative::new(vec![Type::None], "y", Variable::new("y")),
                        Alternative::new(vec![Type::None], "z", Variable::new("z")),
                    ],
                    None,
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn validate_try_operation() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Type::Variant,
                TryOperation::new(
                    Variable::new("x"),
                    "y",
                    Type::None,
                    Variant::new(Type::None, Variable::new("y")),
                ),
            )]),
        )
        .unwrap();
    }

    #[test]
    fn fail_to_validate_try_operation() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    TryOperation::new(
                        Variable::new("x"),
                        "y",
                        Type::None,
                        Variant::new(Type::None, Expression::None),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariable("y".into(), 1))
        );
    }

    #[test]
    fn fail_to_validate_try_operation_with_double_drops() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    TryOperation::new(
                        Variable::new("x"),
                        "y",
                        Type::None,
                        DropVariables::new(
                            [("y".into(), Type::None)].into_iter().collect(),
                            Variable::new("x"),
                        ),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariables(
                [("x".into(), -1), ("y".into(), 0)].into_iter().collect()
            ))
        );
    }

    #[test]
    fn fail_to_validate_try_operation_with_unused_outer_variable() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    TryOperation::new(
                        Expression::None,
                        "y",
                        Type::None,
                        Variant::new(Type::None, Variable::new("y")),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariables(
                [("x".into(), 1), ("y".into(), 0)].into_iter().collect()
            ))
        );
    }

    #[test]
    fn fail_to_validate_try_operation_with_shadowed_variable() {
        assert_eq!(
            validate(
                &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Variant)],
                    Type::Variant,
                    TryOperation::new(
                        Expression::None,
                        "x",
                        Type::None,
                        Variant::new(Type::None, Variable::new("x")),
                    )
                )],)
            ),
            Err(ReferenceCountError::InvalidLocalVariables(
                [("x".into(), 1)].into_iter().collect()
            ))
        );
    }

    #[test]
    fn validate_global_variable() {
        assert_eq!(
            validate(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::new("f", vec![], Type::None, Expression::None),
                FunctionDefinition::new(
                    "g",
                    vec![],
                    Type::None,
                    Call::new(
                        types::Function::new(vec![], Type::None),
                        Variable::new("f"),
                        vec![],
                    )
                )
            ],)),
            Ok(())
        );
    }

    #[test]
    fn validate_type_information() {
        validate(
            &Module::empty().set_function_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Type::None,
                TypeInformationFunction::new(Variable::new("x")),
            )]),
        )
        .unwrap();
    }
}
