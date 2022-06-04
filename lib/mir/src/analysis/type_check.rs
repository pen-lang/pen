mod error;
mod name;

use crate::{
    ir::*,
    types::{self, Type},
};
pub use error::TypeCheckError;
use fnv::FnvHashMap;

pub fn check_types(module: &Module) -> Result<(), TypeCheckError> {
    name::check_names(module)?;

    let types = module
        .type_definitions()
        .iter()
        .map(|definition| (definition.name(), definition.type_()))
        .collect();
    let mut variables = FnvHashMap::<&str, Type>::default();

    for declaration in module.foreign_declarations() {
        variables.insert(declaration.name(), declaration.type_().clone().into());
    }

    for declaration in module.function_declarations() {
        variables.insert(declaration.name(), declaration.type_().clone().into());
    }

    for definition in module.function_definitions() {
        variables.insert(definition.name(), definition.type_().clone().into());
    }

    for definition in module.function_definitions() {
        check_definition(definition, &variables, &types)?;
    }

    for definition in module.foreign_definitions() {
        if !variables.contains_key(definition.name()) {
            return Err(TypeCheckError::ForeignDefinitionNotFound(
                definition.clone(),
            ));
        }
    }

    Ok(())
}

fn check_definition(
    definition: &FunctionDefinition,
    variables: &FnvHashMap<&str, Type>,
    types: &FnvHashMap<&str, &types::RecordBody>,
) -> Result<(), TypeCheckError> {
    let mut variables = variables.clone();

    for argument in definition
        .environment()
        .iter()
        .chain(definition.arguments())
    {
        variables.insert(argument.name(), argument.type_().clone());
    }

    check_equality(
        &check_expression(
            definition.body(),
            &variables,
            definition.result_type(),
            types,
        )?,
        &definition.result_type().clone(),
    )
}

fn check_expression(
    expression: &Expression,
    variables: &FnvHashMap<&str, Type>,
    result_type: &Type,
    types: &FnvHashMap<&str, &types::RecordBody>,
) -> Result<Type, TypeCheckError> {
    let check_expression =
        |expression, variables| check_expression(expression, variables, result_type, types);

    Ok(match expression {
        Expression::ArithmeticOperation(operation) => {
            check_equality(
                &check_expression(operation.lhs(), variables)?,
                &Type::Number,
            )?;
            check_equality(
                &check_expression(operation.rhs(), variables)?,
                &Type::Number,
            )?;

            Type::Number
        }
        Expression::Boolean(_) => Type::Boolean,
        Expression::Case(case) => check_case(case, variables, result_type, types)?,
        Expression::CloneVariables(clone) => {
            for (variable, type_) in clone.variables() {
                check_equality(&check_variable(&Variable::new(variable), variables)?, type_)?;
            }

            check_expression(clone.expression(), variables)?
        }
        Expression::ComparisonOperation(operation) => {
            check_equality(
                &check_expression(operation.lhs(), variables)?,
                &Type::Number,
            )?;
            check_equality(
                &check_expression(operation.rhs(), variables)?,
                &Type::Number,
            )?;

            Type::Boolean
        }
        Expression::DiscardHeap(discard) => check_expression(discard.expression(), variables)?,
        Expression::DropVariables(drop) => {
            check_drop_variables(drop, variables, result_type, types)?
        }
        Expression::Call(call) => {
            check_equality(
                &call.type_().clone().into(),
                &check_expression(call.function(), variables)?
                    .into_function()
                    .ok_or_else(|| TypeCheckError::FunctionExpected(call.function().clone()))?
                    .into(),
            )?;

            if call.arguments().len() != call.type_().arguments().len() {
                return Err(TypeCheckError::WrongArgumentCount(call.clone()));
            }

            for (argument, argument_type) in call.arguments().iter().zip(call.type_().arguments()) {
                check_equality(&check_expression(argument, variables)?, argument_type)?;
            }

            call.type_().result().clone()
        }
        Expression::If(if_) => {
            check_equality(
                &check_expression(if_.condition(), variables)?,
                &Type::Boolean,
            )?;

            let then = check_expression(if_.then(), variables)?;
            let else_ = check_expression(if_.else_(), variables)?;

            check_equality(&then, &else_)?;

            then
        }
        Expression::LetRecursive(let_) => {
            let variables = variables
                .clone()
                .into_iter()
                .chain([(
                    let_.definition().name(),
                    let_.definition().type_().clone().into(),
                )])
                .collect();

            check_definition(let_.definition(), &variables, types)?;
            check_expression(let_.expression(), &variables)?
        }
        Expression::Let(let_) => {
            check_equality(
                &check_expression(let_.bound_expression(), variables)?,
                let_.type_(),
            )?;

            check_expression(
                let_.expression(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(let_.name(), let_.type_().clone())])
                    .collect(),
            )?
        }
        Expression::MarkSync(mark) => check_expression(mark.expression(), variables)?,
        Expression::None => Type::None,
        Expression::Number(_) => Type::Number,
        Expression::Record(record) => check_record(record, variables, result_type, types)?,
        Expression::RecordField(field) => {
            check_equality(
                &check_expression(field.record(), variables)?,
                &field.type_().clone().into(),
            )?;

            types
                .get(field.type_().name())
                .ok_or_else(|| TypeCheckError::TypeNotFound(field.type_().clone()))?
                .fields()
                .get(field.index())
                .ok_or_else(|| TypeCheckError::FieldIndexOutOfBounds(field.clone()))?
                .clone()
        }
        Expression::RecordUpdate(update) => {
            let record_type = types
                .get(update.type_().name())
                .ok_or_else(|| TypeCheckError::TypeNotFound(update.type_().clone()))?;

            check_equality(
                &check_expression(update.record(), variables)?,
                &update.type_().clone().into(),
            )?;

            for field in update.fields() {
                check_equality(
                    &check_expression(field.expression(), variables)?,
                    &record_type.fields()[field.index()],
                )?;
            }

            update.type_().clone().into()
        }
        Expression::ReuseRecord(record) => {
            check_record(record.record(), variables, result_type, types)?
        }
        Expression::RetainHeap(reuse) => {
            check_drop_variables(reuse.drop(), variables, result_type, types)?
        }
        Expression::ByteString(_) => Type::ByteString,
        Expression::TryOperation(operation) => {
            let then_variables = variables
                .clone()
                .into_iter()
                .chain([(operation.name(), operation.type_().clone())])
                .collect();
            check_equality(
                &check_expression(operation.then(), &then_variables)?,
                result_type,
            )?;

            let type_ = check_expression(operation.operand(), variables)?;

            check_equality(&type_, &Type::Variant)?;

            type_
        }
        Expression::Variable(variable) => check_variable(variable, variables)?,
        Expression::Variant(variant) => {
            if matches!(variant.type_(), Type::Variant) {
                return Err(TypeCheckError::VariantInVariant(variant.clone().into()));
            }

            check_equality(
                &check_expression(variant.payload(), variables)?,
                variant.type_(),
            )?;

            Type::Variant
        }
    })
}

fn check_case(
    case: &Case,
    variables: &FnvHashMap<&str, Type>,
    result_type: &Type,
    types: &FnvHashMap<&str, &types::RecordBody>,
) -> Result<Type, TypeCheckError> {
    let check_expression = |expression: &Expression, variables: &FnvHashMap<&str, Type>| {
        check_expression(expression, variables, result_type, types)
    };

    check_equality(
        &check_expression(case.argument(), variables)?,
        &Type::Variant,
    )?;

    let mut expression_type = None;

    for alternative in case.alternatives() {
        if matches!(alternative.type_(), Type::Variant) {
            return Err(TypeCheckError::VariantInVariant(case.clone().into()));
        }

        let mut variables = variables.clone();

        variables.insert(alternative.name(), alternative.type_().clone());

        let alternative_type = check_expression(alternative.expression(), &variables)?;

        if let Some(expression_type) = &expression_type {
            check_equality(&alternative_type, expression_type)?;
        } else {
            expression_type = Some(alternative_type);
        }
    }

    if let Some(alternative) = case.default_alternative() {
        let mut variables = variables.clone();

        variables.insert(alternative.name(), Type::Variant);

        let alternative_type = check_expression(alternative.expression(), &variables)?;

        if let Some(expression_type) = &expression_type {
            check_equality(&alternative_type, expression_type)?;
        } else {
            expression_type = Some(alternative_type);
        }
    }

    expression_type.ok_or_else(|| TypeCheckError::NoAlternativeFound(case.clone()))
}

fn check_drop_variables(
    drop: &DropVariables,
    variables: &FnvHashMap<&str, Type>,
    result_type: &Type,
    types: &FnvHashMap<&str, &types::RecordBody>,
) -> Result<Type, TypeCheckError> {
    for (variable, type_) in drop.variables() {
        check_equality(&check_variable(&Variable::new(variable), variables)?, type_)?;
    }

    check_expression(drop.expression(), variables, result_type, types)
}

fn check_record(
    record: &Record,
    variables: &FnvHashMap<&str, Type>,
    result_type: &Type,
    types: &FnvHashMap<&str, &types::RecordBody>,
) -> Result<Type, TypeCheckError> {
    let record_type = types
        .get(record.type_().name())
        .ok_or_else(|| TypeCheckError::TypeNotFound(record.type_().clone()))?;

    if record.fields().len() != record_type.fields().len() {
        return Err(TypeCheckError::WrongFieldCount(record.clone()));
    }

    for (field, field_type) in record.fields().iter().zip(record_type.fields()) {
        check_equality(
            &check_expression(field, variables, result_type, types)?,
            field_type,
        )?;
    }

    Ok(record.type_().clone().into())
}

fn check_variable(
    variable: &Variable,
    variables: &FnvHashMap<&str, Type>,
) -> Result<Type, TypeCheckError> {
    variables
        .get(variable.name())
        .cloned()
        .ok_or_else(|| TypeCheckError::VariableNotFound(variable.clone()))
}

fn check_equality(one: &Type, other: &Type) -> Result<(), TypeCheckError> {
    if one == other {
        Ok(())
    } else {
        Err(TypeCheckError::TypesNotMatched(one.clone(), other.clone()))
    }
}

#[cfg(test)]
mod tests {
    use super::{check_types, error::*};
    use crate::{
        ir::*,
        types::{self, Type},
    };

    fn create_module_from_definitions(definitions: Vec<FunctionDefinition>) -> Module {
        Module::new(vec![], vec![], vec![], vec![], definitions)
    }

    fn create_module_with_records(
        type_definitions: Vec<TypeDefinition>,
        definitions: Vec<FunctionDefinition>,
    ) -> Module {
        Module::new(type_definitions, vec![], vec![], vec![], definitions)
    }

    #[test]
    fn check_types_with_empty_modules() {
        assert_eq!(
            check_types(&Module::new(vec![], vec![], vec![], vec![], vec![])),
            Ok(())
        );
    }

    #[test]
    fn check_types_of_variables() {
        let module = create_module_from_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            Variable::new("x"),
            Type::Number,
        )]);
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_variables() {
        let module = create_module_from_definitions(vec![
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                42.0,
                Type::Number,
            ),
            FunctionDefinition::new(
                "g",
                vec![Argument::new("x", Type::Number)],
                Variable::new("f"),
                Type::Number,
            ),
        ]);

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    #[test]
    fn check_types_of_functions() {
        let module = create_module_from_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            42.0,
            Type::Number,
        )]);

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_functions() {
        let module = create_module_from_definitions(vec![
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                42.0,
                Type::Number,
            ),
            FunctionDefinition::new(
                "g",
                vec![Argument::new("x", Type::Number)],
                Variable::new("f"),
                Type::Number,
            ),
        ]);

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    #[test]
    fn check_types_of_calls() {
        let module = create_module_from_definitions(vec![
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                42.0,
                Type::Number,
            ),
            FunctionDefinition::new(
                "g",
                vec![Argument::new("x", Type::Number)],
                Call::new(
                    types::Function::new(vec![Type::Number], Type::Number),
                    Variable::new("f"),
                    vec![42.0.into()],
                ),
                Type::Number,
            ),
        ]);

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn check_call_with_2_arguments() {
        let module = create_module_from_definitions(vec![
            FunctionDefinition::new(
                "f",
                vec![
                    Argument::new("x", Type::Number),
                    Argument::new("y", Type::Boolean),
                ],
                42.0,
                Type::Number,
            ),
            FunctionDefinition::new(
                "g",
                vec![Argument::new("x", Type::Number)],
                Call::new(
                    types::Function::new(vec![Type::Number, Type::Boolean], Type::Number),
                    Variable::new("f"),
                    vec![42.0.into(), true.into()],
                ),
                Type::Number,
            ),
        ]);

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_calls() {
        let module = create_module_from_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            Call::new(
                types::Function::new(vec![Type::Number], Type::Number),
                42.0,
                vec![42.0.into()],
            ),
            Type::Number,
        )]);

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::FunctionExpected(_))
        ));
    }

    #[test]
    fn fail_to_check_types_because_of_missing_variables() {
        let module = create_module_from_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            Variable::new("y"),
            Type::Number,
        )]);

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::VariableNotFound(_))
        ));
    }

    #[test]
    fn check_types_of_nested_let_expressions() {
        let module = create_module_from_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            Let::new(
                "y",
                Type::Number,
                42.0,
                Let::new("z", Type::Number, Variable::new("y"), Variable::new("z")),
            ),
            Type::Number,
        )]);

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_let_expression() {
        let module = create_module_from_definitions(vec![
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                42.0,
                Type::Number,
            ),
            FunctionDefinition::new(
                "g",
                vec![Argument::new("x", Type::Number)],
                Let::new("y", Type::Number, Variable::new("f"), Variable::new("y")),
                Type::Number,
            ),
        ]);

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    #[test]
    fn check_types_of_let_recursive() {
        let module = create_module_from_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            LetRecursive::new(
                FunctionDefinition::new(
                    "g",
                    vec![Argument::new("y", Type::Number)],
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Variable::new("x"),
                        Variable::new("y"),
                    ),
                    Type::Number,
                ),
                Call::new(
                    types::Function::new(vec![Type::Number], Type::Number),
                    Variable::new("g"),
                    vec![42.0.into()],
                ),
            ),
            Type::Number,
        )]);

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn check_types_of_recursive_let_recursive() {
        let module = create_module_from_definitions(vec![FunctionDefinition::new(
            "f",
            vec![Argument::new("x", Type::Number)],
            LetRecursive::new(
                FunctionDefinition::new(
                    "g",
                    vec![Argument::new("y", Type::Number)],
                    Call::new(
                        types::Function::new(vec![Type::Number], Type::Number),
                        Variable::new("g"),
                        vec![42.0.into()],
                    ),
                    Type::Number,
                ),
                Call::new(
                    types::Function::new(vec![Type::Number], Type::Number),
                    Variable::new("g"),
                    vec![42.0.into()],
                ),
            ),
            Type::Number,
        )]);

        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn check_types_of_declarations() {
        let module = Module::new(
            vec![],
            vec![],
            vec![],
            vec![FunctionDeclaration::new(
                "f",
                types::Function::new(vec![Type::Number], Type::Number),
            )],
            vec![FunctionDefinition::new(
                "g",
                vec![Argument::new("x", Type::Number)],
                Call::new(
                    types::Function::new(vec![Type::Number], Type::Number),
                    Variable::new("f"),
                    vec![Variable::new("x").into()],
                ),
                Type::Number,
            )],
        );
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn fail_to_check_types_of_declarations() {
        let module = Module::new(
            vec![],
            vec![],
            vec![],
            vec![FunctionDeclaration::new(
                "f",
                types::Function::new(vec![Type::Number], Type::Number),
            )],
            vec![FunctionDefinition::new(
                "g",
                vec![Argument::new("x", Type::Number)],
                Variable::new("f"),
                Type::Number,
            )],
        );

        assert!(matches!(
            check_types(&module),
            Err(TypeCheckError::TypesNotMatched(_, _))
        ));
    }

    mod if_ {
        use super::*;

        #[test]
        fn check() {
            assert_eq!(
                check_types(&create_module_from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Number)],
                        If::new(true, 42.0, 42.0),
                        Type::Number,
                    ),
                ])),
                Ok(())
            );
        }
    }

    mod case_expressions {
        use super::*;

        #[test]
        fn check_case_expressions_only_with_default_alternative() {
            assert_eq!(
                check_types(&create_module_from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Variant)],
                        Case::new(
                            Variable::new("x"),
                            vec![],
                            Some(DefaultAlternative::new("x", 42.0))
                        ),
                        Type::Number,
                    )
                ])),
                Ok(())
            );
        }

        #[test]
        fn check_case_expressions_with_one_alternative() {
            assert_eq!(
                check_types(&create_module_from_definitions(vec![
                    FunctionDefinition::new(
                        "f",
                        vec![Argument::new("x", Type::Variant)],
                        Case::new(
                            Variable::new("x"),
                            vec![Alternative::new(Type::Number, "y", Variable::new("y"))],
                            None
                        ),
                        Type::Number,
                    )
                ])),
                Ok(())
            );
        }

        #[test]
        fn fail_to_check_case_expressions_without_alternatives() {
            let module = create_module_from_definitions(vec![FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Variant)],
                Case::new(Variable::new("x"), vec![], None),
                Type::Number,
            )]);

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::NoAlternativeFound(_))
            ));
        }

        #[test]
        fn fail_to_check_case_expressions_with_inconsistent_alternative_types() {
            let module =
                create_module_from_definitions(vec![FunctionDefinition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", Type::Variant)],
                    Case::new(
                        Variable::new("x"),
                        vec![
                            Alternative::new(Type::Boolean, "x", Variable::new("x")),
                            Alternative::new(Type::Number, "x", 42.0),
                        ],
                        None,
                    ),
                    Type::Number,
                )]);

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::TypesNotMatched(_, _))
            ));
        }

        #[test]
        fn fail_for_unmatched_case_type() {
            assert!(matches!(
                check_types(&create_module_from_definitions(vec![
                    FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", Type::Variant)],
                        Case::new(
                            Variable::new("x"),
                            vec![Alternative::new(
                                types::Record::new("foo"),
                                "y",
                                Variable::new("y")
                            )],
                            None
                        ),
                        types::Record::new("bar"),
                    )
                ])),
                Err(TypeCheckError::TypesNotMatched(_, _))
            ));
        }

        #[test]
        fn fail_to_check_case_with_variant_alternative() {
            assert!(matches!(
                check_types(&create_module_from_definitions(vec![
                    FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", Type::Variant)],
                        Case::new(
                            Variable::new("x"),
                            vec![Alternative::new(Type::Variant, "y", Variable::new("y"))],
                            None
                        ),
                        Type::Variant,
                    )
                ])),
                Err(TypeCheckError::VariantInVariant(_))
            ));
        }
    }

    mod records {
        use super::*;

        #[test]
        fn check_records_with_no_field() {
            let record_type = types::Record::new("foo");

            assert_eq!(
                check_types(&create_module_with_records(
                    vec![TypeDefinition::new("foo", types::RecordBody::new(vec![]))],
                    vec![FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", Type::Number)],
                        Record::new(record_type.clone(), vec![]),
                        record_type,
                    )],
                )),
                Ok(())
            );
        }

        #[test]
        fn check_records_with_fields() {
            let record_type = types::Record::new("foo");

            assert_eq!(
                check_types(&create_module_with_records(
                    vec![TypeDefinition::new(
                        "foo",
                        types::RecordBody::new(vec![Type::Number])
                    )],
                    vec![FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", Type::Number)],
                        Record::new(record_type.clone(), vec![42.0.into()],),
                        record_type,
                    )],
                )),
                Ok(())
            );
        }

        #[test]
        fn fail_to_check_records_with_wrong_number_of_fields() {
            let record_type = types::Record::new("foo");

            let module = create_module_with_records(
                vec![TypeDefinition::new(
                    "foo",
                    types::RecordBody::new(vec![Type::Number]),
                )],
                vec![FunctionDefinition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", Type::Number)],
                    Record::new(record_type.clone(), vec![42.0.into(), 42.0.into()]),
                    record_type,
                )],
            );

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::WrongFieldCount(_))
            ));
        }

        #[test]
        fn fail_to_check_records_with_wrong_field_type() {
            let record_type = types::Record::new("foo");

            let module = create_module_with_records(
                vec![TypeDefinition::new(
                    "foo",
                    types::RecordBody::new(vec![Type::Number]),
                )],
                vec![FunctionDefinition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", Type::Number)],
                    Record::new(record_type.clone(), vec![true.into()]),
                    record_type,
                )],
            );

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::TypesNotMatched(_, _))
            ));
        }

        #[test]
        fn check_record_field() {
            let record_type = types::Record::new("foo");

            assert_eq!(
                check_types(&create_module_with_records(
                    vec![TypeDefinition::new(
                        "foo",
                        types::RecordBody::new(vec![Type::Number])
                    )],
                    vec![FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", Type::Number)],
                        RecordField::new(
                            record_type.clone(),
                            0,
                            Record::new(record_type, vec![42.0.into()],)
                        ),
                        Type::Number
                    )],
                )),
                Ok(())
            );
        }
    }

    mod variants {
        use super::*;

        #[test]
        fn check_variant() {
            assert_eq!(
                check_types(&create_module_from_definitions(vec![
                    FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", Type::Number)],
                        Variant::new(Type::Number, 42.0),
                        Type::Variant
                    )
                ],)),
                Ok(())
            );
        }

        #[test]
        fn fail_to_check_variant_in_variant() {
            assert!(matches!(
                check_types(&create_module_from_definitions(vec![
                    FunctionDefinition::with_environment(
                        "f",
                        vec![],
                        vec![Argument::new("x", Type::Variant)],
                        Variant::new(Type::Variant, Variable::new("x")),
                        Type::Variant
                    )
                ],)),
                Err(TypeCheckError::VariantInVariant(_))
            ));
        }
    }

    #[test]
    fn check_add_operator() {
        let module = create_module_from_definitions(vec![FunctionDefinition::with_environment(
            "f",
            vec![],
            vec![Argument::new("x", Type::Number)],
            ArithmeticOperation::new(ArithmeticOperator::Add, 42.0, 42.0),
            Type::Number,
        )]);
        assert_eq!(check_types(&module), Ok(()));
    }

    #[test]
    fn check_equality_operator() {
        let module = create_module_from_definitions(vec![FunctionDefinition::with_environment(
            "f",
            vec![],
            vec![Argument::new("x", Type::Number)],
            ComparisonOperation::new(ComparisonOperator::Equal, 42.0, 42.0),
            Type::Boolean,
        )]);
        assert_eq!(check_types(&module), Ok(()));
    }

    mod try_operations {
        use super::*;

        #[test]
        fn check_try_operation() {
            let module =
                create_module_from_definitions(vec![FunctionDefinition::with_environment(
                    "f",
                    vec![],
                    vec![Argument::new("x", Type::Variant)],
                    TryOperation::new(
                        Variable::new("x"),
                        "y",
                        Type::Number,
                        Variant::new(Type::Number, Variable::new("y")),
                    ),
                    Type::Variant,
                )]);
            assert_eq!(check_types(&module), Ok(()));
        }
    }

    mod foreign_declarations {
        use super::*;

        #[test]
        fn check_types_of_foreign_declarations() {
            let module = Module::new(
                vec![],
                vec![ForeignDeclaration::new(
                    "f",
                    "g",
                    types::Function::new(vec![Type::Number], Type::Number),
                    CallingConvention::Target,
                )],
                vec![],
                vec![],
                vec![FunctionDefinition::new(
                    "g",
                    vec![Argument::new("x", Type::Number)],
                    Call::new(
                        types::Function::new(vec![Type::Number], Type::Number),
                        Variable::new("f"),
                        vec![Variable::new("x").into()],
                    ),
                    Type::Number,
                )],
            );
            assert_eq!(check_types(&module), Ok(()));
        }

        #[test]
        fn fail_to_check_types_of_foreign_declarations() {
            let module = Module::new(
                vec![],
                vec![ForeignDeclaration::new(
                    "f",
                    "g",
                    types::Function::new(vec![Type::Number], Type::Number),
                    CallingConvention::Target,
                )],
                vec![],
                vec![],
                vec![FunctionDefinition::new(
                    "g",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("f"),
                    Type::Number,
                )],
            );

            assert!(matches!(
                check_types(&module),
                Err(TypeCheckError::TypesNotMatched(_, _))
            ));
        }
    }

    mod foreign_definitions {
        use super::*;

        #[test]
        fn check_types_of_foreign_definition_for_declaration() {
            let module = Module::new(
                vec![],
                vec![],
                vec![ForeignDefinition::new("f", "g", CallingConvention::Source)],
                vec![FunctionDeclaration::new(
                    "f",
                    types::Function::new(vec![Type::Number], Type::Number),
                )],
                vec![],
            );

            assert_eq!(check_types(&module), Ok(()));
        }

        #[test]
        fn check_types_of_foreign_definition_for_definition() {
            let module = Module::new(
                vec![],
                vec![],
                vec![ForeignDefinition::new("f", "g", CallingConvention::Source)],
                vec![],
                vec![FunctionDefinition::new(
                    "f",
                    vec![Argument::new("x", Type::Number)],
                    Variable::new("x"),
                    Type::Number,
                )],
            );

            assert_eq!(check_types(&module), Ok(()));
        }
    }

    #[test]
    fn check_duplicate_function_names() {
        let module = create_module_from_definitions(vec![
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Variable::new("x"),
                Type::Number,
            ),
            FunctionDefinition::new(
                "f",
                vec![Argument::new("x", Type::Number)],
                Variable::new("x"),
                Type::Number,
            ),
        ]);

        assert_eq!(
            check_types(&module),
            Err(TypeCheckError::DuplicateFunctionNames("f".into()))
        );
    }
}
