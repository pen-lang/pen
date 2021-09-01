use super::{environment_creator, type_context::TypeContext, type_extractor, CompileError};
use hir::types::{self, Type};
use hir::{
    analysis::types::{
        record_element_resolver, type_canonicalizer, type_equality_checker,
        type_subsumption_checker, union_type_creator,
    },
    ir::*,
};
use std::collections::{HashMap, HashSet};

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
    let check_expression =
        |expression, variables: &_| check_expression(expression, variables, type_context);
    let check_subsumption =
        |lower: &_, upper| check_subsumption(lower, upper, type_context.types());

    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => {
            let type_ = call
                .function_type()
                .ok_or_else(|| CompileError::TypeNotInferred(call.position().clone()))?;
            let function_type =
                type_canonicalizer::canonicalize_function(type_, type_context.types())?
                    .ok_or_else(|| {
                        CompileError::FunctionExpected(call.function().position().clone())
                    })?;

            check_subsumption(&check_expression(call.function(), variables)?, type_)?;

            if call.arguments().len() != function_type.arguments().len() {
                return Err(CompileError::WrongArgumentCount(call.position().clone()));
            }

            for (argument, type_) in call.arguments().iter().zip(function_type.arguments()) {
                check_subsumption(&check_expression(argument, variables)?, type_)?;
            }

            function_type.result().clone()
        }
        Expression::If(if_) => {
            check_subsumption(
                &check_expression(if_.condition(), variables)?,
                &types::Boolean::new(if_.position().clone()).into(),
            )?;

            check_expression(if_.then(), variables)?;
            check_expression(if_.else_(), variables)?;

            type_extractor::extract_from_expression(expression, variables, type_context)?
        }
        Expression::IfList(if_) => {
            let list_type = types::List::new(
                if_.type_()
                    .ok_or_else(|| {
                        CompileError::TypeNotInferred(if_.argument().position().clone())
                    })?
                    .clone(),
                if_.position().clone(),
            );

            check_subsumption(
                &check_expression(if_.argument(), variables)?,
                &list_type.clone().into(),
            )?;

            check_expression(
                if_.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain(vec![
                        (
                            if_.first_name().into(),
                            types::Function::new(
                                vec![],
                                list_type.element().clone(),
                                if_.position().clone(),
                            )
                            .into(),
                        ),
                        (if_.rest_name().into(), list_type.into()),
                    ])
                    .collect(),
            )?;
            check_expression(if_.else_(), variables)?;

            type_extractor::extract_from_expression(expression, variables, type_context)?
        }
        Expression::IfType(if_) => {
            let argument_type = type_canonicalizer::canonicalize(
                &check_expression(if_.argument(), variables)?,
                type_context.types(),
            )?;

            if !argument_type.is_union() && !argument_type.is_any() {
                return Err(CompileError::UnionOrAnyTypeExpected(
                    if_.argument().position().clone(),
                ));
            }

            for branch in if_.branches() {
                check_expression(
                    branch.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain(vec![(if_.name().into(), branch.type_().clone())])
                        .collect(),
                )?;

                if type_canonicalizer::canonicalize(branch.type_(), type_context.types())?.is_any()
                {
                    return Err(CompileError::AnyTypeBranch(
                        branch.type_().position().clone(),
                    ));
                }
            }

            if let Some(branch) = if_.else_() {
                check_expression(
                    branch.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain(vec![(
                            if_.name().into(),
                            branch
                                .type_()
                                .ok_or_else(|| {
                                    CompileError::TypeNotInferred(branch.position().clone())
                                })?
                                .clone(),
                        )])
                        .collect(),
                )?;
            } else if !type_equality_checker::check(
                &argument_type,
                &union_type_creator::create(
                    &if_.branches()
                        .iter()
                        .map(|branch| branch.type_().clone())
                        .collect::<Vec<_>>(),
                    if_.position(),
                )
                .unwrap(),
                type_context.types(),
            )? {
                return Err(CompileError::MissingElseBlock(if_.position().clone()));
            }

            type_extractor::extract_from_expression(expression, variables, type_context)?
        }
        Expression::Lambda(lambda) => check_lambda(lambda, variables, type_context)?.into(),
        Expression::Let(let_) => {
            check_subsumption(
                &check_expression(let_.bound_expression(), variables)?,
                let_.type_().ok_or_else(|| {
                    CompileError::TypeNotInferred(let_.bound_expression().position().clone())
                })?,
            )?;

            check_expression(
                let_.expression(),
                &variables
                    .clone()
                    .into_iter()
                    .chain(if let Some(name) = let_.name() {
                        Some((
                            name.into(),
                            let_.type_()
                                .ok_or_else(|| {
                                    CompileError::TypeNotInferred(let_.position().clone())
                                })?
                                .clone(),
                        ))
                    } else {
                        None
                    })
                    .collect(),
            )?
        }
        Expression::List(list) => {
            for element in list.elements() {
                match element {
                    ListElement::Multiple(expression) => {
                        check_subsumption(
                            type_canonicalizer::canonicalize_list(
                                &check_expression(expression, variables)?,
                                type_context.types(),
                            )?
                            .ok_or_else(|| {
                                CompileError::ListExpected(expression.position().clone())
                            })?
                            .element(),
                            list.type_(),
                        )?;
                    }
                    ListElement::Single(expression) => {
                        check_subsumption(&check_expression(expression, variables)?, list.type_())?;
                    }
                }
            }

            types::List::new(list.type_().clone(), list.position().clone()).into()
        }
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::Operation(operation) => check_operation(operation, variables, type_context)?,
        Expression::RecordConstruction(construction) => {
            let element_types = record_element_resolver::resolve(
                construction.type_(),
                construction.position(),
                type_context.types(),
                type_context.records(),
            )?;

            for element in construction.elements() {
                check_subsumption(
                    &check_expression(element.expression(), variables)?,
                    element_types
                        .iter()
                        .find(|element_type| element_type.name() == element.name())
                        .ok_or_else(|| {
                            CompileError::RecordElementUnknown(expression.position().clone())
                        })?
                        .type_(),
                )?;
            }

            let element_names = construction
                .elements()
                .iter()
                .map(|element| element.name())
                .collect::<HashSet<_>>();

            for element_type in element_types {
                if !element_names.contains(element_type.name()) {
                    return Err(CompileError::RecordElementMissing(
                        construction.position().clone(),
                    ));
                }
            }

            construction.type_().clone()
        }
        Expression::RecordDeconstruction(deconstruction) => {
            let type_ = deconstruction
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(deconstruction.position().clone()))?;

            check_subsumption(
                &check_expression(deconstruction.record(), variables)?,
                type_,
            )?;

            let element_types = record_element_resolver::resolve(
                type_,
                deconstruction.position(),
                type_context.types(),
                type_context.records(),
            )?;

            element_types
                .iter()
                .find(|element_type| element_type.name() == deconstruction.element_name())
                .ok_or_else(|| {
                    CompileError::RecordElementUnknown(deconstruction.position().clone())
                })?
                .type_()
                .clone()
        }
        Expression::RecordUpdate(update) => {
            check_subsumption(
                &check_expression(update.record(), variables)?,
                update.type_(),
            )?;

            let element_types = record_element_resolver::resolve(
                update.type_(),
                update.position(),
                type_context.types(),
                type_context.records(),
            )?;

            for element in update.elements() {
                check_subsumption(
                    &check_expression(element.expression(), variables)?,
                    element_types
                        .iter()
                        .find(|element_type| element_type.name() == element.name())
                        .ok_or_else(|| {
                            CompileError::RecordElementUnknown(expression.position().clone())
                        })?
                        .type_(),
                )?;
            }

            update.type_().clone()
        }
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Thunk(thunk) => {
            let type_ = thunk
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(thunk.position().clone()))?;

            check_subsumption(&check_expression(thunk.expression(), variables)?, type_)?;

            type_extractor::extract_from_expression(expression, variables, type_context)?
        }
        Expression::TypeCoercion(coercion) => {
            check_subsumption(
                &check_expression(coercion.argument(), variables)?,
                coercion.from(),
            )?;

            if type_canonicalizer::canonicalize_list(coercion.from(), type_context.types())?
                .is_none()
                || type_canonicalizer::canonicalize_list(coercion.to(), type_context.types())?
                    .is_none()
            {
                check_subsumption(coercion.from(), coercion.to())?;
            }

            coercion.to().clone()
        }
        Expression::Variable(variable) => variables
            .get(variable.name())
            .ok_or_else(|| CompileError::VariableNotFound(variable.clone()))?
            .clone(),
    })
}

fn check_operation(
    operation: &Operation,
    variables: &HashMap<String, Type>,
    type_context: &TypeContext,
) -> Result<Type, CompileError> {
    let check_expression = |expression| check_expression(expression, variables, type_context);
    let check_subsumption =
        |lower: &_, upper| check_subsumption(lower, upper, type_context.types());

    Ok(match operation {
        Operation::Arithmetic(operation) => {
            let number_type = types::Number::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.lhs())?, &number_type)?;
            check_subsumption(&check_expression(operation.rhs())?, &number_type)?;

            number_type
        }
        Operation::Boolean(operation) => {
            let boolean_type = types::Boolean::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.lhs())?, &boolean_type)?;
            check_subsumption(&check_expression(operation.rhs())?, &boolean_type)?;

            boolean_type
        }
        Operation::Equality(operation) => {
            let operand_type = operation
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(operation.position().clone()))?;

            check_subsumption(&check_expression(operation.lhs())?, operand_type)?;
            check_subsumption(&check_expression(operation.rhs())?, operand_type)?;

            types::Boolean::new(operation.position().clone()).into()
        }
        Operation::Not(operation) => {
            let boolean_type = types::Boolean::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.expression())?, &boolean_type)?;

            boolean_type
        }
        Operation::Order(operation) => {
            let number_type = types::Number::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.lhs())?, &number_type)?;
            check_subsumption(&check_expression(operation.rhs())?, &number_type)?;

            types::Boolean::new(operation.position().clone()).into()
        }
        Operation::Try(operation) => {
            let position = operation.position();
            let success_type = operation
                .type_()
                .ok_or_else(|| CompileError::TypeNotInferred(position.clone()))?;
            let error_type = types::Reference::new(
                &type_context.error_type_configuration().error_type_name,
                position.clone(),
            )
            .into();
            let union_type = check_expression(operation.expression())?;

            check_subsumption(&error_type, &union_type)?;
            check_subsumption(success_type, &union_type)?;

            check_subsumption(
                &union_type,
                &types::Union::new(success_type.clone(), error_type, position.clone()).into(),
            )?;

            success_type.clone()
        }
    })
}

fn check_subsumption(
    lower: &Type,
    upper: &Type,
    types: &HashMap<String, Type>,
) -> Result<(), CompileError> {
    if type_subsumption_checker::check(lower, upper, types)? {
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
        test,
        {
            error_type_configuration::ERROR_TYPE_CONFIGURATION,
            string_type_configuration::STRING_TYPE_CONFIGURATION,
        },
    };
    use hir::test::{DefinitionFake, ForeignDeclarationFake, ModuleFake, TypeDefinitionFake};

    fn check_module(module: &Module) -> Result<(), CompileError> {
        check_types(
            module,
            &TypeContext::new(
                module,
                &LIST_TYPE_CONFIGURATION,
                &STRING_TYPE_CONFIGURATION,
                &ERROR_TYPE_CONFIGURATION,
            ),
        )
    }

    #[test]
    fn check_empty_module() -> Result<(), CompileError> {
        check_module(&Module::empty())
    }

    #[test]
    fn check_definition() -> Result<(), CompileError> {
        check_module(&Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::None::new(test::position()),
                None::new(test::position()),
                test::position(),
            ),
            false,
        )]))
    }

    #[test]
    fn fail_to_check_function_result_type_of_foreign_declaration() {
        let function_type = types::Function::new(
            vec![],
            types::Number::new(test::position()),
            test::position(),
        );

        assert_eq!(
            check_module(
                &Module::empty()
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(test::position()),
                            Call::new(
                                Some(function_type.clone().into()),
                                Variable::new("y", test::position()),
                                vec![],
                                test::position()
                            ),
                            test::position(),
                        ),
                        false,
                    )])
                    .set_foreign_declarations(vec![ForeignDeclaration::fake("y", function_type,)])
            ),
            Err(CompileError::TypesNotMatched(
                test::position(),
                test::position()
            ))
        );
    }

    #[test]
    fn check_thunk() {
        check_module(&Module::empty().set_definitions(vec![Definition::fake(
            "x",
            Lambda::new(
                vec![],
                types::Function::new(vec![], types::None::new(test::position()), test::position()),
                Thunk::new(
                    Some(types::None::new(test::position()).into()),
                    None::new(test::position()),
                    test::position(),
                ),
                test::position(),
            ),
            false,
        )]))
        .unwrap();
    }

    mod lambda {
        use super::*;

        #[test]
        fn check_subsumption_of_function_result_type() -> Result<(), CompileError> {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Union::new(
                        types::Number::new(test::position()),
                        types::None::new(test::position()),
                        test::position(),
                    ),
                    None::new(test::position()),
                    test::position(),
                ),
                false,
            )]))
        }

        #[test]
        fn fail_to_check_function_result_type() {
            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Number::new(test::position()),
                        None::new(test::position()),
                        test::position(),
                    ),
                    false,
                )])),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }
    }

    mod let_ {
        use super::*;

        #[test]
        fn check_let() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(test::position()),
                    Let::new(
                        Some("x".into()),
                        Some(types::None::new(test::position()).into()),
                        None::new(test::position()),
                        Variable::new("x", test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn fail_to_check_expression_in_let() {
            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        Let::new(
                            Some("x".into()),
                            Some(types::None::new(test::position()).into()),
                            None::new(test::position()),
                            NotOperation::new(None::new(test::position()), test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )])),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }
    }

    mod if_ {
        use super::*;

        #[test]
        fn check_if() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::Number::new(test::position()),
                    If::new(
                        Boolean::new(true, test::position()),
                        Number::new(0.0, test::position()),
                        Number::new(0.0, test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap()
        }

        #[test]
        fn check_if_of_union_type() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::Union::new(
                        types::Number::new(test::position()),
                        types::None::new(test::position()),
                        test::position(),
                    ),
                    If::new(
                        Boolean::new(true, test::position()),
                        Number::new(0.0, test::position()),
                        None::new(test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap()
        }

        #[test]
        fn fail_to_check_then_expression() {
            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Number::new(test::position()),
                        If::new(
                            Boolean::new(true, test::position()),
                            NotOperation::new(None::new(test::position()), test::position()),
                            Number::new(0.0, test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]),),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }

        #[test]
        fn fail_to_check_else_expression() {
            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Number::new(test::position()),
                        If::new(
                            Boolean::new(true, test::position()),
                            Number::new(0.0, test::position()),
                            NotOperation::new(None::new(test::position()), test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]),),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }
    }

    mod if_type {
        use super::*;

        #[test]
        fn check_with_union() {
            let union_type = types::Union::new(
                types::Number::new(test::position()),
                types::None::new(test::position()),
                test::position(),
            );

            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", union_type)],
                    types::None::new(test::position()),
                    IfType::new(
                        "y",
                        Variable::new("x", test::position()),
                        vec![
                            IfTypeBranch::new(
                                types::Number::new(test::position()),
                                None::new(test::position()),
                            ),
                            IfTypeBranch::new(
                                types::None::new(test::position()),
                                None::new(test::position()),
                            ),
                        ],
                        None,
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap()
        }

        #[test]
        fn check_with_any() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", types::Any::new(test::position()))],
                    types::None::new(test::position()),
                    IfType::new(
                        "y",
                        Variable::new("x", test::position()),
                        vec![IfTypeBranch::new(
                            types::None::new(test::position()),
                            None::new(test::position()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::Any::new(test::position()).into()),
                            None::new(test::position()),
                            test::position(),
                        )),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap()
        }

        #[test]
        fn check_result_of_union() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", types::Any::new(test::position()))],
                    types::Union::new(
                        types::Number::new(test::position()),
                        types::None::new(test::position()),
                        test::position(),
                    ),
                    IfType::new(
                        "y",
                        Variable::new("x", test::position()),
                        vec![IfTypeBranch::new(
                            types::None::new(test::position()),
                            Number::new(42.0, test::position()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::Any::new(test::position()).into()),
                            None::new(test::position()),
                            test::position(),
                        )),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap()
        }

        #[test]
        fn check_result_of_any() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", types::Any::new(test::position()))],
                    types::Any::new(test::position()),
                    IfType::new(
                        "y",
                        Variable::new("x", test::position()),
                        vec![IfTypeBranch::new(
                            types::None::new(test::position()),
                            None::new(test::position()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::Any::new(test::position()).into()),
                            Variable::new("y", test::position()),
                            test::position(),
                        )),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_due_to_wrong_argument_type() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![],
                    types::None::new(test::position()),
                    IfType::new(
                        "y",
                        None::new(test::position()),
                        vec![IfTypeBranch::new(
                            types::None::new(test::position()),
                            None::new(test::position()),
                        )],
                        None,
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_union_due_to_missing_else() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new(
                        "x",
                        types::Union::new(
                            types::Number::new(test::position()),
                            types::None::new(test::position()),
                            test::position(),
                        ),
                    )],
                    types::None::new(test::position()),
                    IfType::new(
                        "y",
                        Variable::new("x", test::position()),
                        vec![IfTypeBranch::new(
                            types::Number::new(test::position()),
                            None::new(test::position()),
                        )],
                        None,
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        #[should_panic]
        fn fail_to_check_any_due_to_missing_else() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new(
                        "x",
                        types::Union::new(
                            types::Number::new(test::position()),
                            types::None::new(test::position()),
                            test::position(),
                        ),
                    )],
                    types::None::new(test::position()),
                    IfType::new(
                        "y",
                        Variable::new("x", test::position()),
                        vec![IfTypeBranch::new(
                            types::Number::new(test::position()),
                            None::new(test::position()),
                        )],
                        None,
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        #[should_panic]
        fn fail_to_check_due_to_any_type_branch() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "f",
                Lambda::new(
                    vec![Argument::new("x", types::Any::new(test::position()))],
                    types::None::new(test::position()),
                    IfType::new(
                        "y",
                        Variable::new("x", test::position()),
                        vec![IfTypeBranch::new(
                            types::Any::new(test::position()),
                            None::new(test::position()),
                        )],
                        Some(ElseBranch::new(
                            Some(types::Any::new(test::position()).into()),
                            None::new(test::position()),
                            test::position(),
                        )),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }
    }

    mod calls {
        use super::*;

        #[test]
        fn check_call() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(test::position()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![],
                                    types::None::new(test::position()),
                                    test::position(),
                                )
                                .into(),
                            ),
                            Variable::new("f", test::position()),
                            vec![],
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]))
            .unwrap()
        }

        #[test]
        fn check_call_with_arguments() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(test::position()))],
                        types::None::new(test::position()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![types::None::new(test::position()).into()],
                                    types::None::new(test::position()),
                                    test::position(),
                                )
                                .into(),
                            ),
                            Variable::new("f", test::position()),
                            vec![None::new(test::position()).into()],
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_type() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(test::position()))],
                        types::None::new(test::position()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![types::None::new(test::position()).into()],
                                    types::None::new(test::position()),
                                    test::position(),
                                )
                                .into(),
                            ),
                            Variable::new("f", test::position()),
                            vec![Number::new(42.0, test::position()).into()],
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]))
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_count() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(test::position()))],
                        types::None::new(test::position()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![types::None::new(test::position()).into()],
                                    types::None::new(test::position()),
                                    test::position(),
                                )
                                .into(),
                            ),
                            Variable::new("f", test::position()),
                            vec![],
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]))
            .unwrap()
        }
    }

    mod operations {
        use super::*;

        #[test]
        fn check_arithmetic_operation() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Number::new(test::position()),
                    ArithmeticOperation::new(
                        ArithmeticOperator::Add,
                        Number::new(0.0, test::position()),
                        Number::new(0.0, test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_boolean_operation() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Boolean::new(test::position()),
                    BooleanOperation::new(
                        BooleanOperator::And,
                        Boolean::new(true, test::position()),
                        Boolean::new(true, test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn fail_to_check_boolean_operation() {
            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(test::position()),
                        BooleanOperation::new(
                            BooleanOperator::And,
                            Number::new(42.0, test::position()),
                            Boolean::new(true, test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )],)),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }

        #[test]
        fn check_equality_operation() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Boolean::new(test::position()),
                    EqualityOperation::new(
                        Some(types::Number::new(test::position()).into()),
                        EqualityOperator::Equal,
                        Number::new(0.0, test::position()),
                        Number::new(0.0, test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_equality_operation_with_subsumption() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(test::position()),
                        EqualityOperation::new(
                            Some(
                                types::Union::new(
                                    types::Number::new(test::position()),
                                    types::None::new(test::position()),
                                    test::position(),
                                )
                                .into(),
                            ),
                            EqualityOperator::Equal,
                            Number::new(0.0, test::position()),
                            None::new(test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]))
            .unwrap();
        }

        #[test]
        fn check_not_operation() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Boolean::new(test::position()),
                    NotOperation::new(Boolean::new(true, test::position()), test::position()),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_order_operation() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Boolean::new(test::position()),
                    OrderOperation::new(
                        OrderOperator::LessThan,
                        Number::new(0.0, test::position()),
                        Number::new(0.0, test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_try_operation() {
            let union_type = types::Union::new(
                types::None::new(test::position()),
                types::Reference::new("error", test::position()),
                test::position(),
            );

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "error",
                        vec![],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            union_type,
                            TryOperation::new(
                                Some(types::None::new(test::position()).into()),
                                Variable::new("x", test::position()),
                                test::position(),
                            ),
                            test::position(),
                        ),
                        false,
                    )]),
            )
            .unwrap();
        }

        #[test]
        fn check_try_operation_with_number() {
            let union_type = types::Union::new(
                types::Number::new(test::position()),
                types::Reference::new("error", test::position()),
                test::position(),
            );

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "error",
                        vec![],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            union_type,
                            ArithmeticOperation::new(
                                ArithmeticOperator::Add,
                                TryOperation::new(
                                    Some(types::Number::new(test::position()).into()),
                                    Variable::new("x", test::position()),
                                    test::position(),
                                ),
                                Number::new(42.0, test::position()),
                                test::position(),
                            ),
                            test::position(),
                        ),
                        false,
                    )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_try_operation_with_any() {
            let any_type = types::Any::new(test::position());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "error",
                        vec![],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![Definition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", any_type.clone())],
                            any_type.clone(),
                            TryOperation::new(
                                Some(any_type.into()),
                                Variable::new("x", test::position()),
                                test::position(),
                            ),
                            test::position(),
                        ),
                        false,
                    )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_try_operation_with_wrong_success_type() {
            let union_type = types::Union::new(
                types::None::new(test::position()),
                types::Reference::new("error", test::position()),
                test::position(),
            );

            assert_eq!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "error",
                            vec![],
                            false,
                            false,
                            false,
                        )])
                        .set_definitions(vec![Definition::fake(
                            "f",
                            Lambda::new(
                                vec![Argument::new("x", union_type.clone())],
                                union_type,
                                TryOperation::new(
                                    Some(types::Number::new(test::position()).into()),
                                    Variable::new("x", test::position()),
                                    test::position(),
                                ),
                                test::position(),
                            ),
                            false,
                        )]),
                ),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }

        #[test]
        fn fail_to_check_try_operation_with_wrong_operand_type() {
            assert_eq!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "error",
                            vec![],
                            false,
                            false,
                            false,
                        )])
                        .set_definitions(vec![Definition::fake(
                            "f",
                            Lambda::new(
                                vec![Argument::new("x", types::None::new(test::position()))],
                                types::Union::new(
                                    types::None::new(test::position()),
                                    types::Reference::new("error", test::position()),
                                    test::position(),
                                ),
                                TryOperation::new(
                                    Some(types::Number::new(test::position()).into()),
                                    Variable::new("x", test::position()),
                                    test::position(),
                                ),
                                test::position(),
                            ),
                            false,
                        )]),
                ),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }
    }

    mod record {
        use super::*;

        #[test]
        fn check_record() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", test::position());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "r",
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(test::position()),
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            RecordConstruction::new(
                                reference_type,
                                vec![RecordElement::new(
                                    "x",
                                    None::new(test::position()),
                                    test::position(),
                                )],
                                test::position(),
                            ),
                            test::position(),
                        ),
                        false,
                    )]),
            )
        }

        #[test]
        fn fail_to_check_record_with_missing_element() {
            let reference_type = types::Reference::new("r", test::position());

            assert!(matches!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "r",
                            vec![types::RecordElement::new(
                                "x",
                                types::None::new(test::position()),
                            )],
                            false,
                            false,
                            false
                        )])
                        .set_definitions(vec![Definition::fake(
                            "x",
                            Lambda::new(
                                vec![],
                                reference_type.clone(),
                                RecordConstruction::new(
                                    reference_type,
                                    Default::default(),
                                    test::position(),
                                ),
                                test::position(),
                            ),
                            false
                        )])
                ),
                Err(CompileError::RecordElementMissing(_))
            ));
        }

        #[test]
        fn fail_to_check_record_with_unknown_element() {
            let reference_type = types::Reference::new("r", test::position());

            assert!(matches!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "r",
                            vec![],
                            false,
                            false,
                            false
                        )])
                        .set_definitions(vec![Definition::fake(
                            "x",
                            Lambda::new(
                                vec![],
                                reference_type.clone(),
                                RecordConstruction::new(
                                    reference_type,
                                    vec![RecordElement::new(
                                        "x",
                                        None::new(test::position()),
                                        test::position()
                                    )],
                                    test::position(),
                                ),
                                test::position(),
                            ),
                            false
                        )])
                ),
                Err(CompileError::RecordElementUnknown(_))
            ));
        }

        #[test]
        fn check_record_update() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", test::position());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "r",
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(test::position()),
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", reference_type.clone())],
                            reference_type.clone(),
                            RecordUpdate::new(
                                reference_type,
                                Variable::new("x", test::position()),
                                vec![RecordElement::new(
                                    "x",
                                    None::new(test::position()),
                                    test::position(),
                                )],
                                test::position(),
                            ),
                            test::position(),
                        ),
                        false,
                    )]),
            )
        }

        #[test]
        fn check_record_deconstruction() -> Result<(), CompileError> {
            let reference_type = types::Reference::new("r", test::position());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "r",
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(test::position()),
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![Definition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", reference_type.clone())],
                            types::None::new(test::position()),
                            RecordDeconstruction::new(
                                Some(reference_type.into()),
                                Variable::new("x", test::position()),
                                "x",
                                test::position(),
                            ),
                            test::position(),
                        ),
                        false,
                    )]),
            )
        }

        #[test]
        fn fail_to_check_record_deconstruction_due_to_unknown_element() {
            let reference_type = types::Reference::new("r", test::position());

            assert_eq!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "r",
                            vec![types::RecordElement::new(
                                "x",
                                types::None::new(test::position()),
                            )],
                            false,
                            false,
                            false,
                        )])
                        .set_definitions(vec![Definition::fake(
                            "x",
                            Lambda::new(
                                vec![Argument::new("x", reference_type.clone())],
                                types::None::new(test::position()),
                                RecordDeconstruction::new(
                                    Some(reference_type.into()),
                                    Variable::new("x", test::position()),
                                    "y",
                                    test::position(),
                                ),
                                test::position(),
                            ),
                            false,
                        )])
                ),
                Err(CompileError::RecordElementUnknown(test::position()))
            );
        }

        #[test]
        fn fail_to_check_different_records() {
            assert_eq!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![
                            TypeDefinition::fake("r1", vec![], false, false, false,),
                            TypeDefinition::fake("r2", vec![], false, false, false,)
                        ])
                        .set_definitions(vec![Definition::fake(
                            "x",
                            Lambda::new(
                                vec![Argument::new(
                                    "x",
                                    types::Reference::new("r1", test::position())
                                )],
                                types::Reference::new("r2", test::position()),
                                Variable::new("x", test::position()),
                                test::position(),
                            ),
                            false,
                        )])
                ),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }
    }

    mod lists {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn check_list_with_single_element() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::List::new(types::None::new(test::position()), test::position()),
                    List::new(
                        types::None::new(test::position()),
                        vec![ListElement::Single(None::new(test::position()).into())],
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_list_with_multiple_element() {
            let list_type = types::List::new(types::None::new(test::position()), test::position());

            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![Argument::new("x", list_type.clone())],
                    list_type,
                    List::new(
                        types::None::new(test::position()),
                        vec![ListElement::Multiple(
                            Variable::new("x", test::position()).into(),
                        )],
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn fail_to_check_list_with_single_element() {
            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::List::new(types::None::new(test::position()), test::position()),
                        List::new(
                            types::None::new(test::position()),
                            vec![ListElement::Single(
                                Number::new(42.0, test::position()).into(),
                            )],
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )])),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }

        #[test]
        fn fail_to_check_list_with_multiple_element() {
            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::List::new(
                                types::Number::new(test::position()),
                                test::position()
                            )
                        )],
                        types::List::new(types::None::new(test::position()), test::position()),
                        List::new(
                            types::None::new(test::position()),
                            vec![ListElement::Multiple(
                                Variable::new("x", test::position()).into(),
                            )],
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]),),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position(),
                ))
            );
        }

        #[test]
        fn check_list_with_single_element_of_union() {
            let union_type = types::Union::new(
                types::Number::new(test::position()),
                types::None::new(test::position()),
                test::position(),
            );

            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::List::new(union_type.clone(), test::position()),
                    List::new(
                        union_type,
                        vec![ListElement::Single(None::new(test::position()).into())],
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_list_with_multiple_element_of_union() {
            let union_type = types::Union::new(
                types::Number::new(test::position()),
                types::None::new(test::position()),
                test::position(),
            );

            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![Argument::new(
                        "x",
                        types::List::new(types::None::new(test::position()), test::position()),
                    )],
                    types::List::new(union_type.clone(), test::position()),
                    List::new(
                        union_type,
                        vec![ListElement::Multiple(
                            Variable::new("x", test::position()).into(),
                        )],
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }
    }

    mod if_list {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn check_first_variable() {
            let list_type = types::List::new(types::None::new(test::position()), test::position());
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::None::new(test::position()),
                        IfList::new(
                            Some(types::None::new(test::position()).into()),
                            Variable::new("x", test::position()),
                            "y",
                            "ys",
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![],
                                        types::None::new(test::position()),
                                        test::position(),
                                    )
                                    .into(),
                                ),
                                Variable::new("y", test::position()),
                                vec![],
                                test::position(),
                            ),
                            None::new(test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]))
            .unwrap();
        }

        #[test]
        fn check_rest_variable() {
            let list_type = types::List::new(types::None::new(test::position()), test::position());
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![Argument::new("x", list_type.clone())],
                    list_type,
                    IfList::new(
                        Some(types::None::new(test::position()).into()),
                        Variable::new("x", test::position()),
                        "y",
                        "ys",
                        Variable::new("ys", test::position()),
                        Variable::new("x", test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_union_type_result() {
            let list_type = types::List::new(types::None::new(test::position()), test::position());
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::Union::new(
                            types::None::new(test::position()),
                            types::Number::new(test::position()),
                            test::position(),
                        ),
                        IfList::new(
                            Some(types::None::new(test::position()).into()),
                            Variable::new("x", test::position()),
                            "y",
                            "ys",
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![],
                                        types::None::new(test::position()),
                                        test::position(),
                                    )
                                    .into(),
                                ),
                                Variable::new("y", test::position()),
                                vec![],
                                test::position(),
                            ),
                            Number::new(42.0, test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]))
            .unwrap();
        }

        #[test]
        fn fail_to_check_argument() {
            let list_type =
                types::List::new(types::Number::new(test::position()), test::position());

            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::None::new(test::position()),
                        IfList::new(
                            Some(types::None::new(test::position()).into()),
                            Variable::new("x", test::position()),
                            "y",
                            "ys",
                            Variable::new("y", test::position()),
                            None::new(test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]),),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }

        #[test]
        fn fail_to_check_result() {
            let list_type = types::List::new(types::None::new(test::position()), test::position());

            assert_eq!(
                check_module(&Module::empty().set_definitions(vec![Definition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::None::new(test::position()),
                        IfList::new(
                            Some(types::None::new(test::position()).into()),
                            Variable::new("x", test::position()),
                            "y",
                            "ys",
                            Variable::new("y", test::position()),
                            Number::new(42.0, test::position()),
                            test::position(),
                        ),
                        test::position(),
                    ),
                    false,
                )]),),
                Err(CompileError::TypesNotMatched(
                    test::position(),
                    test::position()
                ))
            );
        }
    }

    mod type_coercion {
        use super::*;

        #[test]
        fn check_union() {
            let union_type = types::Union::new(
                types::Number::new(test::position()),
                types::None::new(test::position()),
                test::position(),
            );

            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    union_type.clone(),
                    TypeCoercion::new(
                        types::None::new(test::position()),
                        union_type,
                        None::new(test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_any() {
            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Any::new(test::position()),
                    TypeCoercion::new(
                        types::None::new(test::position()),
                        types::Any::new(test::position()),
                        None::new(test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }

        #[test]
        fn check_list() {
            let none_list_type =
                types::List::new(types::None::new(test::position()), test::position());
            let any_list_type =
                types::List::new(types::Any::new(test::position()), test::position());

            check_module(&Module::empty().set_definitions(vec![Definition::fake(
                "x",
                Lambda::new(
                    vec![Argument::new("x", none_list_type.clone())],
                    any_list_type.clone(),
                    TypeCoercion::new(
                        none_list_type,
                        any_list_type,
                        Variable::new("x", test::position()),
                        test::position(),
                    ),
                    test::position(),
                ),
                false,
            )]))
            .unwrap();
        }
    }
}
