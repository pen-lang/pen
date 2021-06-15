use super::{
    environment, type_context::TypeContext, type_extraction, type_subsumption, CompileError,
};
use crate::{
    compile::type_resolution,
    hir::*,
    types::{self, Type},
};
use std::collections::HashMap;

pub fn check_types(module: &Module, type_context: &TypeContext) -> Result<(), CompileError> {
    let variables = environment::create_from_module(module);

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
        &check_block(
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

    Ok(type_extraction::extract_from_lambda(lambda))
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
            let element_types = type_resolution::resolve_record_elements(
                construction.type_(),
                type_context.types(),
                type_context.records(),
            )?
            .ok_or_else(|| CompileError::RecordExpected(construction.type_().position().clone()))?;

            for (name, element) in construction.elements() {
                check_subsumption(
                    &check_expression(element, variables, type_context)?,
                    element_types.get(name).ok_or_else(|| {
                        CompileError::RecordElementUnknown(element.position().clone())
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
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Variable(variable) => variables
            .get(variable.name())
            .ok_or_else(|| CompileError::VariableNotFound(variable.clone()))?
            .clone(),
        _ => todo!(),
    })
}

fn check_block(
    block: &Block,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    let mut variables = variables.clone();

    for statement in block.statements() {
        check_subsumption(
            &check_expression(statement.expression(), &variables, type_context)?,
            statement
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(statement.position().clone()))?,
            type_context.types(),
        )?;

        if let Some(name) = statement.name() {
            variables.insert(
                name.into(),
                statement
                    .type_()
                    .cloned()
                    .ok_or_else(|| CompileError::TypeNotInferred(statement.position().clone()))?,
            );
        }
    }

    check_expression(block.expression(), &variables, type_context)
}

fn check_subsumption(
    lower: &Type,
    upper: &Type,
    types: &HashMap<String, Type>,
) -> Result<(), CompileError> {
    if type_subsumption::check_subsumption(lower, upper, types)? {
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
    use super::*;
    use crate::{compile::list_type_configuration::LIST_TYPE_CONFIGURATION, position::Position};

    fn check_module(module: &Module) -> Result<(), CompileError> {
        check_types(module, &TypeContext::new(module, &LIST_TYPE_CONFIGURATION))
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
            vec![Definition::new(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Block::new(vec![], None::new(Position::dummy())),
                    Position::dummy(),
                ),
                false,
                Position::dummy(),
            )],
        ))
    }

    #[test]
    fn check_statement() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Block::new(
                        vec![Statement::new(
                            Some("y".into()),
                            None::new(Position::dummy()),
                            Some(types::None::new(Position::dummy()).into()),
                            Position::dummy(),
                        )],
                        Variable::new("y", Position::dummy()),
                    ),
                    Position::dummy(),
                ),
                false,
                Position::dummy(),
            )],
        ))
    }

    #[test]
    fn check_subsumption_of_function_result_type() -> Result<(), CompileError> {
        check_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::new(
                "x",
                Lambda::new(
                    vec![],
                    types::Union::new(
                        types::Number::new(Position::dummy()),
                        types::None::new(Position::dummy()),
                        Position::dummy(),
                    ),
                    Block::new(vec![], None::new(Position::dummy())),
                    Position::dummy(),
                ),
                false,
                Position::dummy(),
            )],
        ))
    }

    mod records {
        use super::*;

        #[test]
        fn check_record() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", Position::dummy());

            check_module(&Module::new(
                vec![TypeDefinition::new(
                    "r",
                    vec![types::RecordElement::new(
                        "x",
                        types::None::new(Position::dummy()),
                    )],
                    false,
                    false,
                    false,
                    Position::dummy(),
                )],
                vec![],
                vec![],
                vec![Definition::new(
                    "x",
                    Lambda::new(
                        vec![],
                        reference_type.clone(),
                        Block::new(
                            vec![],
                            RecordConstruction::new(
                                reference_type,
                                vec![("x".into(), None::new(Position::dummy()).into())]
                                    .into_iter()
                                    .collect(),
                                Position::dummy(),
                            ),
                        ),
                        Position::dummy(),
                    ),
                    false,
                    Position::dummy(),
                )],
            ))
        }

        #[test]
        fn fail_to_check_record_with_missing_element() {
            let reference_type = types::Reference::new("r", Position::dummy());

            assert!(matches!(
                check_module(&Module::new(
                    vec![TypeDefinition::new(
                        "r",
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(Position::dummy()),
                        )],
                        false,
                        false,
                        false,
                        Position::dummy(),
                    )],
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            Block::new(
                                vec![],
                                RecordConstruction::new(
                                    reference_type,
                                    Default::default(),
                                    Position::dummy(),
                                ),
                            ),
                            Position::dummy(),
                        ),
                        false,
                        Position::dummy(),
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
                    vec![TypeDefinition::new(
                        "r",
                        vec![],
                        false,
                        false,
                        false,
                        Position::dummy(),
                    )],
                    vec![],
                    vec![],
                    vec![Definition::new(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            Block::new(
                                vec![],
                                RecordConstruction::new(
                                    reference_type,
                                    vec![("x".into(), None::new(Position::dummy()).into())]
                                        .into_iter()
                                        .collect(),
                                    Position::dummy(),
                                ),
                            ),
                            Position::dummy(),
                        ),
                        false,
                        Position::dummy(),
                    )],
                )),
                Err(CompileError::RecordElementUnknown(_))
            ));
        }
    }
}
