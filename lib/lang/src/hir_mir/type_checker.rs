use super::{environment_creator, type_context::TypeContext, type_extractor, CompileError};
use crate::{
    hir::*,
    types::{self, analysis::type_resolver, Type},
};
use std::collections::HashMap;

pub fn check_types(module: &Module, type_context: &TypeContext) -> Result<(), CompileError> {
    let variables = environment_creator::create_from_module(module);

    for definition in module.definitions() {
        check_lambda(definition.lambda(), &variables, type_context)?;
    }

    Ok(())
}

fn check_lambda(
    lambda: &Lambda,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<types::Function, CompileError> {
    check_subsumption(
        &check_expression(
            lambda.body(),
            &variables
                .clone()
                .into_iter()
                .chain(
                    lambda
                        .arguments()
                        .iter()
                        .map(|argument| (argument.name().into(), argument.type_().clone())),
                )
                .collect(),
            type_context,
        )?,
        lambda.result_type(),
        type_context.types(),
    )?;

    Ok(type_extractor::extract_from_lambda(lambda))
}

fn check_expression(
    expression: &Expression,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => {
            let type_ = call
                .function_type()
                .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?;
            let function_type = type_resolver::resolve_to_function(type_, type_context.types())?
                .ok_or_else(|| {
                    CompileError::FunctionExpected(call.function().position().clone())
                })?;

            check_subsumption(
                &check_expression(call.function(), variables, type_context)?,
                type_,
                type_context.types(),
            )?;

            if call.arguments().len() != function_type.arguments().len() {
                return Err(CompileError::WrongArgumentCount(call.position().clone()));
            }

            for (argument, type_) in call.arguments().iter().zip(function_type.arguments()) {
                check_subsumption(
                    &check_expression(argument, variables, type_context)?,
                    type_,
                    type_context.types(),
                )?;
            }

            function_type.result().clone()
        }
        Expression::Lambda(lambda) => check_lambda(lambda, variables, type_context)?.into(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::RecordConstruction(construction) => {
            let element_types = type_resolver::resolve_record_elements(
                construction.type_(),
                construction.position(),
                type_context.types(),
                type_context.records(),
            )?;

            for (name, expression) in construction.elements() {
                check_subsumption(
                    &check_expression(expression, variables, type_context)?,
                    element_types.get(name).ok_or_else(|| {
                        CompileError::RecordElementUnknown(expression.position().clone())
                    })?,
                    type_context.types(),
                )?;
            }

            for name in element_types.keys() {
                if !construction.elements().contains_key(name) {
                    return Err(CompileError::RecordElementMissing(
                        construction.position().clone(),
                    ));
                }
            }

            construction.type_().clone()
        }
        Expression::RecordUpdate(update) => {
            check_subsumption(
                &check_expression(update.record(), variables, type_context)?,
                update.type_(),
                type_context.types(),
            )?;

            let element_types = type_resolver::resolve_record_elements(
                update.type_(),
                update.position(),
                type_context.types(),
                type_context.records(),
            )?;

            for (name, expression) in update.elements() {
                check_subsumption(
                    &check_expression(expression, variables, type_context)?,
                    element_types.get(name).ok_or_else(|| {
                        CompileError::RecordElementUnknown(expression.position().clone())
                    })?,
                    type_context.types(),
                )?;
            }

            update.type_().clone()
        }
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .ok_or_else(|| CompileError::VariableNotFound(variable.clone()))?
            .clone(),
        _ => todo!(),
    })
}

fn check_subsumption(
    lower: &Type,
    upper: &Type,
    types: &HashMap<String, Type>,
) -> Result<(), CompileError> {
    if types::analysis::check_subsumption(lower, upper, types)? {
        Ok(())
    } else {
        Err(CompileError::TypesNotMatched(
            lower.position().clone(),
            upper.position().clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::{super::list_type_configuration::LIST_TYPE_CONFIGURATION, *};
    use crate::{
        hir_mir::string_type_configuration::STRING_TYPE_CONFIGURATION, position::Position,
    };

    fn check_module(module: &Module) -> Result<(), CompileError> {
        check_types(
            module,
            &TypeContext::new(module, &LIST_TYPE_CONFIGURATION, &STRING_TYPE_CONFIGURATION),
        )
    }

    #[test]
    fn check_empty_module() -> Result<(), CompileError> {
        check_module(&Module::new(vec![], vec![], vec![], vec![]))
    }

    #[test]
    fn check_definition() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    None::new(Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )],
        ))
    }

    #[test]
    fn check_subsumption_of_function_result_type() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::None::new(Position::dummy()),
                        Position::dummy(),
                    ),
                    None::new(Position::dummy()),
                    Position::dummy(),
                ),
                false,
            )],
        ))
    }

    mod calls {
        use super::*;

        #[test]
        fn check_call() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![],
                            Some(
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        fn check_call_with_arguments() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![None::new(Position::dummy()).into()],
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::dummy()).into()],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_type() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![Number::new(42.0, Position::dummy()).into()],
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::dummy()).into()],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_count() {
            check_module(&Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::dummy()))],
                        types::None::new(Position::dummy()),
                        Call::new(
                            Variable::new("f", Position::dummy()),
                            vec![],
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::dummy()).into()],
                                    types::None::new(Position::dummy()),
                                    Position::dummy(),
                                )
                                .into(),
                            ),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
            .unwrap()
        }
    }

    mod records {
        use super::*;

        #[test]
        fn check_record() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", Position::dummy());

            check_module(&Module::new(
                vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![],
                        reference_type.clone(),
                        RecordConstruction::new(
                            reference_type,
                            vec![("x".into(), None::new(Position::dummy()).into())]
                                .into_iter()
                                .collect(),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
        }

        #[test]
        fn fail_to_check_record_with_missing_element() {
            let reference_type = types::Reference::new("r", Position::dummy());

            assert!(matches!(
                check_module(&Module::new(
                    vec![TypeDefinition::without_source(
                        "r",
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(Position::dummy()),
                        )],
                        false,
                        false,
                        false
                    )],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            RecordConstruction::new(
                                reference_type,
                                Default::default(),
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        false
                    )],
                )),
                Err(CompileError::RecordElementMissing(_))
            ));
        }

        #[test]
        fn fail_to_check_record_with_unknown_element() {
            let reference_type = types::Reference::new("r", Position::dummy());

            assert!(matches!(
                check_module(&Module::new(
                    vec![TypeDefinition::without_source(
                        "r",
                        vec![],
                        false,
                        false,
                        false
                    )],
                    vec![],
                    vec![],
                    vec![Definition::without_source(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            RecordConstruction::new(
                                reference_type,
                                vec![("x".into(), None::new(Position::dummy()).into())]
                                    .into_iter()
                                    .collect(),
                                Position::dummy(),
                            ),
                            Position::dummy(),
                        ),
                        false
                    )],
                )),
                Err(CompileError::RecordElementUnknown(_))
            ));
        }

        #[test]
        fn check_record_update() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", Position::dummy());

            check_module(&Module::new(
                vec![TypeDefinition::without_source(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                )],
                vec![],
                vec![],
                vec![Definition::without_source(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", reference_type.clone())],
                        reference_type.clone(),
                        RecordUpdate::new(
                            reference_type,
                            Variable::new("x", Position::dummy()),
                            vec![("x".into(), None::new(Position::dummy()).into())]
                                .into_iter()
                                .collect(),
                            Position::dummy(),
                        ),
                        Position::dummy(),
                    ),
                    false,
                )],
            ))
        }
    }
}
