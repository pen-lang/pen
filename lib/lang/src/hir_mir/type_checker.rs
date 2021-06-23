use super::record_element_resolver;
use super::{environment_creator, type_context::TypeContext, type_extractor, CompileError};
use crate::{
    hir::*,
    types::{self, Type},
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
        Expression::Lambda(lambda) => check_lambda(lambda, variables, type_context)?.into(),
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::RecordConstruction(construction) => {
            let element_types = record_element_resolver::resolve_elements(
                construction.type_(),
                construction.position(),
                type_context,
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

            construction.type_().clone().into()
        }
        Expression::RecordUpdate(update) => {
            check_subsumption(
                &check_expression(update.record(), variables, type_context)?,
                update.type_(),
                type_context.types(),
            )?;

            let element_types = record_element_resolver::resolve_elements(
                update.type_(),
                update.position(),
                type_context,
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

            update.type_().clone().into()
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
