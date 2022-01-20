use super::ReferenceCountError;
use crate::ir::*;
use std::collections::{BTreeMap, BTreeSet, HashMap};

pub fn validate(module: &Module) -> Result<(), ReferenceCountError> {
    for definition in module.definitions() {
        validate_global_definition(definition)?;
    }

    Ok(())
}

fn validate_global_definition(definition: &Definition) -> Result<(), ReferenceCountError> {
    validate_definition_body(
        definition.body(),
        collect_definition_local_variables(definition)
            .into_iter()
            .map(|name| (name, 1))
            .collect(),
    )
}

fn validate_local_definition(definition: &Definition) -> Result<(), ReferenceCountError> {
    validate_definition_body(
        definition.body(),
        [definition.name().into()]
            .into_iter()
            .chain(collect_definition_local_variables(definition))
            .map(|name| (name, 1))
            .collect(),
    )
}

fn collect_definition_local_variables(definition: &Definition) -> BTreeSet<String> {
    definition
        .environment()
        .iter()
        .chain(definition.arguments())
        .map(|argument| argument.name().into())
        .collect()
}

fn validate_definition_body(
    body: &Expression,
    mut variables: HashMap<String, isize>,
) -> Result<(), ReferenceCountError> {
    move_expression(body, &mut variables)?;

    let invalid_variables = variables
        .into_iter()
        .filter(|(_, count)| count != &0)
        .collect::<BTreeMap<_, _>>();

    if !invalid_variables.is_empty() {
        return Err(ReferenceCountError::InvalidExpression(invalid_variables));
    }

    Ok(())
}

fn move_expression(
    expression: &Expression,
    variables: &mut HashMap<String, isize>,
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

                if variables != &alternative_variables {
                    return Err(ReferenceCountError::InvalidAlternative(alternative.clone()));
                }
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
        Expression::DropVariables(drop) => {
            for name in drop.variables().keys() {
                drop_variable(name, variables);
            }

            move_expression(drop.expression(), variables)?;
        }
        Expression::If(if_) => {
            move_expression(if_.condition(), variables)?;

            let mut then_variables = variables.clone();
            move_expression(if_.then(), &mut then_variables)?;

            move_expression(if_.else_(), variables)?;

            if variables != &then_variables {
                return Err(ReferenceCountError::InvalidIf(if_.clone()));
            }
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

            let name = let_.definition().name();
            let old_count = variables.insert(name.into(), 1);

            move_expression(let_.expression(), variables)?;

            if variables[name] != 0 {
                return Err(ReferenceCountError::InvalidLetRecursive(let_.clone()));
            } else if let Some(old) = old_count {
                variables.insert(name.into(), old);
            }
        }
        Expression::None => {}
        Expression::Number(_) => {}
        Expression::Record(record) => {
            for field in record.fields() {
                move_expression(field, variables)?;
            }
        }
        Expression::RecordField(field) => {
            move_expression(field.record(), variables)?;
        }
        Expression::TryOperation(_) => todo!(),
        Expression::Variable(variable) => {
            drop_variable(variable.name(), variables);
        }
        Expression::Variant(variant) => {
            move_expression(variant.payload(), variables)?;
        }
    }

    Ok(())
}

fn validate_let_like(
    name: &str,
    expression: &Expression,
    variables: &mut HashMap<String, isize>,
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

fn clone_variable(name: impl AsRef<str>, variables: &mut HashMap<String, isize>) {
    variables.insert(name.as_ref().into(), variables[name.as_ref()] + 1);
}

fn drop_variable(name: impl AsRef<str>, variables: &mut HashMap<String, isize>) {
    variables.insert(name.as_ref().into(), variables[name.as_ref()] - 1);
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{self, Type};

    #[test]
    fn validate_empty_module() {
        validate(&Module::new(vec![], vec![], vec![], vec![], vec![])).unwrap();
    }

    #[test]
    fn validate_none() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new("f", vec![], Expression::None, Type::None)],
        ))
        .unwrap();
    }

    #[test]
    fn validate_variable_clone() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                CloneVariables::new(
                    [("x".into(), Type::Number)].into_iter().collect(),
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Variable::new("x"),
                        Variable::new("x"),
                    ),
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_variable_drop() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                DropVariables::new(
                    [("x".into(), Type::None)].into_iter().collect(),
                    Expression::None,
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_variable_drop() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Expression::None,
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_variable_move() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Variable::new("x"),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_let() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Let::new("y", Type::None, Variable::new("x"), Variable::new("y")),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_let_with_undropped_bound_variable() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Let::new("y", Type::None, Variable::new("x"), Expression::None),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_let_with_shadowed_variable() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Let::new("x", Type::None, Variable::new("x"), Variable::new("x")),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_let_with_shadowed_variable() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                Let::new("x", Type::None, Variable::new("x"), Expression::None),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_let_recursive() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                LetRecursive::new(
                    Definition::with_options(
                        "g",
                        vec![Argument::new("x", Type::None)],
                        vec![],
                        DropVariables::new(
                            [("g".into(), types::Function::new(vec![], Type::None).into())]
                                .into_iter()
                                .collect(),
                            Variable::new("x"),
                        ),
                        Type::None,
                        false,
                    ),
                    Variable::new("g"),
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_let_recursive_with_undropped_function() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                LetRecursive::new(
                    Definition::with_options(
                        "g",
                        vec![Argument::new("x", Type::None)],
                        vec![],
                        DropVariables::new(
                            [("g".into(), types::Function::new(vec![], Type::None).into())]
                                .into_iter()
                                .collect(),
                            Variable::new("x"),
                        ),
                        Type::None,
                        false,
                    ),
                    Expression::None,
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_definition_in_let_recursive() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                LetRecursive::new(
                    Definition::with_options(
                        "g",
                        vec![Argument::new("x", Type::None)],
                        vec![],
                        Variable::new("x"),
                        Type::None,
                        false,
                    ),
                    Variable::new("g"),
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_if() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                If::new(
                    Expression::Boolean(true),
                    Variable::new("x"),
                    Variable::new("x"),
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_if() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::None)],
                If::new(
                    Expression::Boolean(true),
                    Variable::new("x"),
                    Expression::None,
                ),
                Type::None,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_case_with_default_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_case_with_default_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![],
                    Some(DefaultAlternative::new("y", Expression::None)),
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_case_with_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![Alternative::new(Type::None, "y", Variable::new("y"))],
                    None,
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_case_with_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![Alternative::new(Type::None, "y", Expression::None)],
                    None,
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_case_with_alternative_and_default_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![Alternative::new(Type::None, "y", Variable::new("y"))],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_case_with_alternative_and_default_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![Alternative::new(Type::None, "y", Expression::None)],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }

    #[test]
    fn validate_case_with_two_alternatives_and_default_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![
                        Alternative::new(Type::None, "y", Variable::new("y")),
                        Alternative::new(Type::None, "y", Variable::new("y")),
                    ],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }

    #[test]
    #[should_panic]
    fn fail_to_validate_case_with_two_alternatives_and_default_alternative() {
        validate(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(
                    Variable::new("x"),
                    vec![
                        Alternative::new(Type::None, "y", Variable::new("y")),
                        Alternative::new(Type::None, "y", Expression::None),
                    ],
                    Some(DefaultAlternative::new("y", Variable::new("y"))),
                ),
                Type::Variant,
            )],
        ))
        .unwrap();
    }
}
