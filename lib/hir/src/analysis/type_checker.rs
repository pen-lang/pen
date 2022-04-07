use super::{context::AnalysisContext, module_environment_creator, AnalysisError};
use crate::{
    analysis::{
        record_field_resolver, type_canonicalizer, type_equality_checker, type_extractor,
        type_subsumption_checker, union_type_creator,
    },
    ir::*,
    types::{self, Type},
};
use fnv::{FnvHashMap, FnvHashSet};

pub fn check_types(context: &AnalysisContext, module: &Module) -> Result<(), AnalysisError> {
    let variables = module_environment_creator::create(module);

    for definition in module.function_definitions() {
        check_lambda(context, definition.lambda(), &variables)?;
    }

    Ok(())
}

fn check_lambda(
    context: &AnalysisContext,
    lambda: &Lambda,
    variables: &FnvHashMap<String, Type>,
) -> Result<types::Function, AnalysisError> {
    check_subsumption(
        &check_expression(
            context,
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
        )?,
        lambda.result_type(),
        context.types(),
    )?;

    Ok(type_extractor::extract_from_lambda(lambda))
}

fn check_expression(
    context: &AnalysisContext,
    expression: &Expression,
    variables: &FnvHashMap<String, Type>,
) -> Result<Type, AnalysisError> {
    let check_expression =
        |expression, variables: &_| check_expression(context, expression, variables);
    let check_subsumption = |lower: &_, upper| check_subsumption(lower, upper, context.types());

    Ok(match expression {
        Expression::Boolean(boolean) => types::Boolean::new(boolean.position().clone()).into(),
        Expression::Call(call) => {
            let type_ = call
                .function_type()
                .ok_or_else(|| AnalysisError::TypeNotInferred(call.position().clone()))?;
            let function_type = type_canonicalizer::canonicalize_function(type_, context.types())?
                .ok_or_else(|| {
                    AnalysisError::FunctionExpected(call.function().position().clone())
                })?;

            check_subsumption(&check_expression(call.function(), variables)?, type_)?;

            if call.arguments().len() != function_type.arguments().len() {
                return Err(AnalysisError::WrongArgumentCount(call.position().clone()));
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

            type_extractor::extract_from_expression(context, expression, variables)?
        }
        Expression::IfList(if_) => {
            let list_type = types::List::new(
                if_.type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(if_.list().position().clone()))?
                    .clone(),
                if_.position().clone(),
            );

            check_subsumption(
                &check_expression(if_.list(), variables)?,
                &list_type.clone().into(),
            )?;

            check_expression(
                if_.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([
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

            type_extractor::extract_from_expression(context, expression, variables)?
        }
        Expression::IfMap(if_) => {
            let map_type = types::Map::new(
                if_.key_type()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(if_.map().position().clone()))?
                    .clone(),
                if_.value_type()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(if_.map().position().clone()))?
                    .clone(),
                if_.position().clone(),
            );

            check_subsumption(&check_expression(if_.key(), variables)?, map_type.key())?;
            check_subsumption(
                &check_expression(if_.map(), variables)?,
                &map_type.clone().into(),
            )?;

            check_expression(
                if_.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(if_.name().into(), map_type.value().clone())])
                    .collect(),
            )?;
            check_expression(if_.else_(), variables)?;

            type_extractor::extract_from_expression(context, expression, variables)?
        }
        Expression::IfType(if_) => {
            let argument_type = type_canonicalizer::canonicalize(
                &check_expression(if_.argument(), variables)?,
                context.types(),
            )?;

            if !argument_type.is_variant() {
                return Err(AnalysisError::VariantExpected(
                    if_.argument().position().clone(),
                ));
            }

            for branch in if_.branches() {
                check_subsumption(branch.type_(), &argument_type)?;

                check_expression(
                    branch.expression(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(if_.name().into(), branch.type_().clone())])
                        .collect(),
                )?;

                if type_canonicalizer::canonicalize(branch.type_(), context.types())?.is_any() {
                    return Err(AnalysisError::AnyTypeBranch(
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
                        .chain([(
                            if_.name().into(),
                            branch
                                .type_()
                                .ok_or_else(|| {
                                    AnalysisError::TypeNotInferred(branch.position().clone())
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
                context.types(),
            )? {
                return Err(AnalysisError::MissingElseBlock(if_.position().clone()));
            }

            type_extractor::extract_from_expression(context, expression, variables)?
        }
        Expression::Lambda(lambda) => check_lambda(context, lambda, variables)?.into(),
        Expression::Let(let_) => {
            check_subsumption(
                &check_expression(let_.bound_expression(), variables)?,
                let_.type_().ok_or_else(|| {
                    AnalysisError::TypeNotInferred(let_.bound_expression().position().clone())
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
                                    AnalysisError::TypeNotInferred(let_.position().clone())
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
                                context.types(),
                            )?
                            .ok_or_else(|| {
                                AnalysisError::ListExpected(expression.position().clone())
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
        Expression::ListComprehension(comprehension) => {
            let position = comprehension.position();
            let input_type = comprehension
                .input_type()
                .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;

            check_subsumption(
                &check_expression(
                    comprehension.element(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(
                            comprehension.element_name().into(),
                            types::Function::new(vec![], input_type.clone(), position.clone())
                                .into(),
                        )])
                        .collect(),
                )?,
                comprehension.output_type(),
            )?;

            check_subsumption(
                &check_expression(comprehension.list(), variables)?,
                &types::List::new(input_type.clone(), position.clone()).into(),
            )?;

            types::List::new(comprehension.output_type().clone(), position.clone()).into()
        }
        Expression::Map(map) => {
            for element in map.elements() {
                match element {
                    MapElement::Insertion(entry) => {
                        check_subsumption(
                            &check_expression(entry.key(), variables)?,
                            map.key_type(),
                        )?;
                        check_subsumption(
                            &check_expression(entry.value(), variables)?,
                            map.value_type(),
                        )?;
                    }
                    MapElement::Map(expression) => {
                        let map_type = type_canonicalizer::canonicalize_map(
                            &check_expression(expression, variables)?,
                            context.types(),
                        )?
                        .ok_or_else(|| AnalysisError::MapExpected(expression.position().clone()))?;

                        check_subsumption(map_type.key(), map.key_type())?;
                        check_subsumption(map_type.value(), map.value_type())?;
                    }
                    MapElement::Removal(expression) => {
                        check_subsumption(
                            &check_expression(expression, variables)?,
                            map.key_type(),
                        )?;
                    }
                }
            }

            types::Map::new(
                map.key_type().clone(),
                map.value_type().clone(),
                map.position().clone(),
            )
            .into()
        }
        Expression::None(none) => types::None::new(none.position().clone()).into(),
        Expression::Number(number) => types::Number::new(number.position().clone()).into(),
        Expression::Operation(operation) => check_operation(context, operation, variables)?,
        Expression::RecordConstruction(construction) => {
            let field_types = record_field_resolver::resolve(
                construction.type_(),
                construction.position(),
                context.types(),
                context.records(),
            )?;

            for field in construction.fields() {
                check_subsumption(
                    &check_expression(field.expression(), variables)?,
                    field_types
                        .iter()
                        .find(|field_type| field_type.name() == field.name())
                        .ok_or_else(|| AnalysisError::UnknownRecordField(field.position().clone()))?
                        .type_(),
                )?;
            }

            let field_names = construction
                .fields()
                .iter()
                .map(|field| field.name())
                .collect::<FnvHashSet<_>>();

            for field_type in field_types {
                if !field_names.contains(field_type.name()) {
                    return Err(AnalysisError::RecordFieldMissing(
                        construction.position().clone(),
                    ));
                }
            }

            construction.type_().clone()
        }
        Expression::RecordDeconstruction(deconstruction) => {
            let type_ = deconstruction
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(deconstruction.position().clone()))?;

            check_subsumption(
                &check_expression(deconstruction.record(), variables)?,
                type_,
            )?;

            let field_types = record_field_resolver::resolve(
                type_,
                deconstruction.position(),
                context.types(),
                context.records(),
            )?;

            field_types
                .iter()
                .find(|field_type| field_type.name() == deconstruction.field_name())
                .ok_or_else(|| {
                    AnalysisError::UnknownRecordField(deconstruction.position().clone())
                })?
                .type_()
                .clone()
        }
        Expression::RecordUpdate(update) => {
            check_subsumption(
                &check_expression(update.record(), variables)?,
                update.type_(),
            )?;

            let field_types = record_field_resolver::resolve(
                update.type_(),
                update.position(),
                context.types(),
                context.records(),
            )?;

            for field in update.fields() {
                check_subsumption(
                    &check_expression(field.expression(), variables)?,
                    field_types
                        .iter()
                        .find(|field_type| field_type.name() == field.name())
                        .ok_or_else(|| AnalysisError::UnknownRecordField(field.position().clone()))?
                        .type_(),
                )?;
            }

            update.type_().clone()
        }
        Expression::String(string) => types::ByteString::new(string.position().clone()).into(),
        Expression::Thunk(thunk) => {
            let type_ = thunk
                .type_()
                .ok_or_else(|| AnalysisError::TypeNotInferred(thunk.position().clone()))?;

            check_subsumption(&check_expression(thunk.expression(), variables)?, type_)?;

            type_extractor::extract_from_expression(context, expression, variables)?
        }
        Expression::TypeCoercion(coercion) => {
            check_subsumption(
                &check_expression(coercion.argument(), variables)?,
                coercion.from(),
            )?;

            let to_type = type_canonicalizer::canonicalize(coercion.to(), context.types())?;

            if !to_type.is_list() && !to_type.is_map() {
                check_subsumption(coercion.from(), coercion.to())?;
            }

            coercion.to().clone()
        }
        Expression::Variable(variable) => variables
            .get(variable.name())
            .ok_or_else(|| AnalysisError::VariableNotFound(variable.clone()))?
            .clone(),
    })
}

fn check_operation(
    context: &AnalysisContext,
    operation: &Operation,
    variables: &FnvHashMap<String, Type>,
) -> Result<Type, AnalysisError> {
    let check_expression = |expression| check_expression(context, expression, variables);
    let check_subsumption = |lower: &_, upper| check_subsumption(lower, upper, context.types());

    Ok(match operation {
        Operation::Arithmetic(operation) => {
            let number_type = types::Number::new(operation.position().clone()).into();

            check_subsumption(&check_expression(operation.lhs())?, &number_type)?;
            check_subsumption(&check_expression(operation.rhs())?, &number_type)?;

            number_type
        }
        Operation::Spawn(operation) => {
            if !operation.function().arguments().is_empty() {
                return Err(AnalysisError::SpawnOperationArguments(
                    operation.position().clone(),
                ));
            }

            check_lambda(context, operation.function(), variables)?.into()
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
                .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?;

            check_subsumption(&check_expression(operation.lhs())?, operand_type)?;
            check_subsumption(&check_expression(operation.rhs())?, operand_type)?;

            let lhs_type =
                type_extractor::extract_from_expression(context, operation.lhs(), variables)?;
            let rhs_type =
                type_extractor::extract_from_expression(context, operation.rhs(), variables)?;

            if !type_subsumption_checker::check(&lhs_type, &rhs_type, context.types())?
                && !type_subsumption_checker::check(&rhs_type, &lhs_type, context.types())?
            {
                return Err(AnalysisError::TypeNotComparable(
                    operation.position().clone(),
                ));
            }

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
                .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
            let union_type = check_expression(operation.expression())?;

            check_subsumption(context.error_type()?, &union_type)?;
            check_subsumption(success_type, &union_type)?;

            check_subsumption(
                &union_type,
                &types::Union::new(
                    success_type.clone(),
                    context.error_type()?.clone(),
                    position.clone(),
                )
                .into(),
            )?;

            success_type.clone()
        }
    })
}

fn check_subsumption(
    lower: &Type,
    upper: &Type,
    types: &FnvHashMap<String, Type>,
) -> Result<(), AnalysisError> {
    if type_subsumption_checker::check(lower, upper, types)? {
        Ok(())
    } else {
        Err(AnalysisError::TypesNotMatched(
            lower.position().clone(),
            upper.position().clone(),
        ))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analysis::type_collector,
        test::{ForeignDeclarationFake, FunctionDefinitionFake, ModuleFake, TypeDefinitionFake},
    };
    use position::{test::PositionFake, Position};

    fn check_module(module: &Module) -> Result<(), AnalysisError> {
        check_types(
            &AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_records(module),
                Some(types::Record::new("error", Position::fake()).into()),
            ),
            module,
        )
    }

    #[test]
    fn check_empty_module() -> Result<(), AnalysisError> {
        check_module(&Module::empty())
    }

    #[test]
    fn check_definition() -> Result<(), AnalysisError> {
        check_module(
            &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::fake()),
                    None::new(Position::fake()),
                    Position::fake(),
                ),
                false,
            )]),
        )
    }

    #[test]
    fn fail_to_check_function_result_type_of_foreign_declaration() {
        let function_type = types::Function::new(
            vec![],
            types::Number::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            check_module(
                &Module::empty()
                    .set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Call::new(
                                Some(function_type.clone().into()),
                                Variable::new("y", Position::fake()),
                                vec![],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                    .set_foreign_declarations(vec![ForeignDeclaration::fake("y", function_type,)])
            ),
            Err(AnalysisError::TypesNotMatched(
                Position::fake(),
                Position::fake()
            ))
        );
    }

    #[test]
    fn check_thunk() {
        check_module(
            &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                "x",
                Lambda::new(
                    vec![],
                    types::Function::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Thunk::new(
                        Some(types::None::new(Position::fake()).into()),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    Position::fake(),
                ),
                false,
            )]),
        )
        .unwrap();
    }

    mod lambda {
        use super::*;

        #[test]
        fn check_subsumption_of_function_result_type() -> Result<(), AnalysisError> {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Union::new(
                            types::Number::new(Position::fake()),
                            types::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        None::new(Position::fake()),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
        }

        #[test]
        fn fail_to_check_function_result_type() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Number::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }
    }

    mod let_ {
        use super::*;

        #[test]
        fn check_let() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Let::new(
                            Some("x".into()),
                            Some(types::None::new(Position::fake()).into()),
                            None::new(Position::fake()),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_expression_in_let() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Let::new(
                                Some("x".into()),
                                Some(types::None::new(Position::fake()).into()),
                                None::new(Position::fake()),
                                NotOperation::new(None::new(Position::fake()), Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }
    }

    mod if_ {
        use super::*;

        #[test]
        fn check_if() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Number::new(Position::fake()),
                        If::new(
                            Boolean::new(true, Position::fake()),
                            Number::new(0.0, Position::fake()),
                            Number::new(0.0, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        fn check_if_of_union_type() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Union::new(
                            types::Number::new(Position::fake()),
                            types::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        If::new(
                            Boolean::new(true, Position::fake()),
                            Number::new(0.0, Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        fn fail_to_check_then_expression() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::Number::new(Position::fake()),
                            If::new(
                                Boolean::new(true, Position::fake()),
                                NotOperation::new(None::new(Position::fake()), Position::fake()),
                                Number::new(0.0, Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }

        #[test]
        fn fail_to_check_else_expression() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::Number::new(Position::fake()),
                            If::new(
                                Boolean::new(true, Position::fake()),
                                Number::new(0.0, Position::fake()),
                                NotOperation::new(None::new(Position::fake()), Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }
    }

    mod if_type {
        use super::*;

        #[test]
        fn check_with_union() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", union_type)],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::Number::new(Position::fake()),
                                    None::new(Position::fake()),
                                ),
                                IfTypeBranch::new(
                                    types::None::new(Position::fake()),
                                    None::new(Position::fake()),
                                ),
                            ],
                            None,
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        fn check_with_any() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                            )],
                            Some(ElseBranch::new(
                                Some(types::Any::new(Position::fake()).into()),
                                None::new(Position::fake()),
                                Position::fake(),
                            )),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        fn check_result_of_union() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::fake()))],
                        types::Union::new(
                            types::Number::new(Position::fake()),
                            types::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::fake()),
                                Number::new(42.0, Position::fake()),
                            )],
                            Some(ElseBranch::new(
                                Some(types::Any::new(Position::fake()).into()),
                                None::new(Position::fake()),
                                Position::fake(),
                            )),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        fn check_result_of_any() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::fake()))],
                        types::Any::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                            )],
                            Some(ElseBranch::new(
                                Some(types::Any::new(Position::fake()).into()),
                                Variable::new("y", Position::fake()),
                                Position::fake(),
                            )),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_due_to_wrong_argument_type() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            None::new(Position::fake()),
                            vec![IfTypeBranch::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                            )],
                            None,
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_union_due_to_missing_else() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Union::new(
                                types::Number::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake(),
                            ),
                        )],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::Number::new(Position::fake()),
                                None::new(Position::fake()),
                            )],
                            None,
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        #[should_panic]
        fn fail_to_check_any_due_to_missing_else() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Union::new(
                                types::Number::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake(),
                            ),
                        )],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::Number::new(Position::fake()),
                                None::new(Position::fake()),
                            )],
                            None,
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        #[should_panic]
        fn fail_to_check_due_to_any_type_branch() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::Any::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::Any::new(Position::fake()),
                                None::new(Position::fake()),
                            )],
                            Some(ElseBranch::new(
                                Some(types::Any::new(Position::fake()).into()),
                                None::new(Position::fake()),
                                Position::fake(),
                            )),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        #[should_panic]
        fn fail_to_check_union_due_to_mismatched_branch_type() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        types::None::new(Position::fake()),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::Boolean::new(Position::fake()),
                                None::new(Position::fake()),
                            )],
                            Some(ElseBranch::new(
                                Some(union_type.into()),
                                None::new(Position::fake()),
                                Position::fake(),
                            )),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }
    }

    mod calls {
        use super::*;

        #[test]
        fn check_call() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into(),
                            ),
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        fn check_call_with_arguments() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::fake()).into()],
                                    types::None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into(),
                            ),
                            Variable::new("f", Position::fake()),
                            vec![None::new(Position::fake()).into()],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_type() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::fake()).into()],
                                    types::None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into(),
                            ),
                            Variable::new("f", Position::fake()),
                            vec![Number::new(42.0, Position::fake()).into()],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }

        #[test]
        #[should_panic]
        fn fail_to_check_call_with_wrong_argument_count() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::fake()))],
                        types::None::new(Position::fake()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![types::None::new(Position::fake()).into()],
                                    types::None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into(),
                            ),
                            Variable::new("f", Position::fake()),
                            vec![],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap()
        }
    }

    mod operations {
        use super::*;

        #[test]
        fn check_arithmetic_operation() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Number::new(Position::fake()),
                        ArithmeticOperation::new(
                            ArithmeticOperator::Add,
                            Number::new(0.0, Position::fake()),
                            Number::new(0.0, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_boolean_operation() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::fake()),
                        BooleanOperation::new(
                            BooleanOperator::And,
                            Boolean::new(true, Position::fake()),
                            Boolean::new(true, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_boolean_operation() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Boolean::new(Position::fake()),
                            BooleanOperation::new(
                                BooleanOperator::And,
                                Number::new(42.0, Position::fake()),
                                Boolean::new(true, Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }

        #[test]
        fn check_equality_operation() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::fake()),
                        EqualityOperation::new(
                            Some(types::Number::new(Position::fake()).into()),
                            EqualityOperator::Equal,
                            Number::new(0.0, Position::fake()),
                            Number::new(0.0, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_equality_operation_with_subsumption() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::fake()),
                        EqualityOperation::new(
                            Some(
                                types::Union::new(
                                    types::Number::new(Position::fake()),
                                    types::None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into(),
                            ),
                            EqualityOperator::Equal,
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_equality_operation() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Boolean::new(Position::fake()),
                            EqualityOperation::new(
                                Some(
                                    types::Union::new(
                                        types::Number::new(Position::fake()),
                                        types::None::new(Position::fake()),
                                        Position::fake(),
                                    )
                                    .into(),
                                ),
                                EqualityOperator::Equal,
                                Number::new(42.0, Position::fake()),
                                None::new(Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypeNotComparable(Position::fake()))
            );
        }

        #[test]
        fn check_not_operation() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::fake()),
                        NotOperation::new(Boolean::new(true, Position::fake()), Position::fake()),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_order_operation() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::fake()),
                        OrderOperation::new(
                            OrderOperator::LessThan,
                            Number::new(0.0, Position::fake()),
                            Number::new(0.0, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_try_operation() {
            let union_type = types::Union::new(
                types::None::new(Position::fake()),
                types::Reference::new("error", Position::fake()),
                Position::fake(),
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
                    .set_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            union_type,
                            TryOperation::new(
                                Some(types::None::new(Position::fake()).into()),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            )
            .unwrap();
        }

        #[test]
        fn check_try_operation_with_number() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::Reference::new("error", Position::fake()),
                Position::fake(),
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
                    .set_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            union_type,
                            ArithmeticOperation::new(
                                ArithmeticOperator::Add,
                                TryOperation::new(
                                    Some(types::Number::new(Position::fake()).into()),
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                ),
                                Number::new(42.0, Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_try_operation_with_any() {
            let any_type = types::Any::new(Position::fake());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "error",
                        vec![],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", any_type.clone())],
                            any_type.clone(),
                            TryOperation::new(
                                Some(any_type.into()),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_try_operation_with_wrong_success_type() {
            let union_type = types::Union::new(
                types::None::new(Position::fake()),
                types::Reference::new("error", Position::fake()),
                Position::fake(),
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
                        .set_definitions(vec![FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![Argument::new("x", union_type.clone())],
                                union_type,
                                TryOperation::new(
                                    Some(types::Number::new(Position::fake()).into()),
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false,
                        )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
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
                        .set_definitions(vec![FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![Argument::new("x", types::None::new(Position::fake()))],
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    types::Reference::new("error", Position::fake()),
                                    Position::fake(),
                                ),
                                TryOperation::new(
                                    Some(types::Number::new(Position::fake()).into()),
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false,
                        )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }

        #[test]
        fn check_spawn_operation() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Function::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        SpawnOperation::new(
                            Lambda::new(
                                vec![],
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_spawn_operation() {
            let none_type = types::None::new(Position::fake());

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::Function::new(
                                vec![none_type.clone().into()],
                                none_type.clone(),
                                Position::fake(),
                            ),
                            SpawnOperation::new(
                                Lambda::new(
                                    vec![Argument::new("x", none_type.clone())],
                                    none_type,
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::SpawnOperationArguments(Position::fake()))
            );
        }
    }

    mod record {
        use super::*;

        #[test]
        fn check_record() -> Result<(), AnalysisError> {
            let reference_type = types::Reference::new("r", Position::fake());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "r",
                        vec![types::RecordField::new(
                            "x",
                            types::None::new(Position::fake()),
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            reference_type.clone(),
                            RecordConstruction::new(
                                reference_type,
                                vec![RecordField::new(
                                    "x",
                                    None::new(Position::fake()),
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            )
        }

        #[test]
        fn fail_to_check_record_with_missing_field() {
            let reference_type = types::Reference::new("r", Position::fake());

            assert!(matches!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "r",
                            vec![types::RecordField::new(
                                "x",
                                types::None::new(Position::fake()),
                            )],
                            false,
                            false,
                            false
                        )])
                        .set_definitions(vec![FunctionDefinition::fake(
                            "x",
                            Lambda::new(
                                vec![],
                                reference_type.clone(),
                                RecordConstruction::new(
                                    reference_type,
                                    Default::default(),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false
                        )])
                ),
                Err(AnalysisError::RecordFieldMissing(_))
            ));
        }

        #[test]
        fn fail_to_check_record_with_unknown_field() {
            let reference_type = types::Reference::new("r", Position::fake());

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
                        .set_definitions(vec![FunctionDefinition::fake(
                            "x",
                            Lambda::new(
                                vec![],
                                reference_type.clone(),
                                RecordConstruction::new(
                                    reference_type,
                                    vec![RecordField::new(
                                        "x",
                                        None::new(Position::fake()),
                                        Position::fake()
                                    )],
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false
                        )])
                ),
                Err(AnalysisError::UnknownRecordField(_))
            ));
        }

        #[test]
        fn check_record_update() -> Result<(), AnalysisError> {
            let reference_type = types::Reference::new("r", Position::fake());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "r",
                        vec![types::RecordField::new(
                            "x",
                            types::None::new(Position::fake()),
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", reference_type.clone())],
                            reference_type.clone(),
                            RecordUpdate::new(
                                reference_type,
                                Variable::new("x", Position::fake()),
                                vec![RecordField::new(
                                    "x",
                                    None::new(Position::fake()),
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            )
        }

        #[test]
        fn check_record_deconstruction() -> Result<(), AnalysisError> {
            let reference_type = types::Reference::new("r", Position::fake());

            check_module(
                &Module::empty()
                    .set_type_definitions(vec![TypeDefinition::fake(
                        "r",
                        vec![types::RecordField::new(
                            "x",
                            types::None::new(Position::fake()),
                        )],
                        false,
                        false,
                        false,
                    )])
                    .set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", reference_type.clone())],
                            types::None::new(Position::fake()),
                            RecordDeconstruction::new(
                                Some(reference_type.into()),
                                Variable::new("x", Position::fake()),
                                "x",
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
            )
        }

        #[test]
        fn fail_to_check_record_deconstruction_due_to_unknown_field() {
            let reference_type = types::Reference::new("r", Position::fake());

            assert_eq!(
                check_module(
                    &Module::empty()
                        .set_type_definitions(vec![TypeDefinition::fake(
                            "r",
                            vec![types::RecordField::new(
                                "x",
                                types::None::new(Position::fake()),
                            )],
                            false,
                            false,
                            false,
                        )])
                        .set_definitions(vec![FunctionDefinition::fake(
                            "x",
                            Lambda::new(
                                vec![Argument::new("x", reference_type.clone())],
                                types::None::new(Position::fake()),
                                RecordDeconstruction::new(
                                    Some(reference_type.into()),
                                    Variable::new("x", Position::fake()),
                                    "y",
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false,
                        )])
                ),
                Err(AnalysisError::UnknownRecordField(Position::fake()))
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
                        .set_definitions(vec![FunctionDefinition::fake(
                            "x",
                            Lambda::new(
                                vec![Argument::new(
                                    "x",
                                    types::Reference::new("r1", Position::fake())
                                )],
                                types::Reference::new("r2", Position::fake()),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            false,
                        )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }
    }

    mod lists {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn check_list_with_single_element() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::List::new(types::None::new(Position::fake()), Position::fake()),
                        List::new(
                            types::None::new(Position::fake()),
                            vec![ListElement::Single(None::new(Position::fake()).into())],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_list_with_multiple_element() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        list_type,
                        List::new(
                            types::None::new(Position::fake()),
                            vec![ListElement::Multiple(
                                Variable::new("x", Position::fake()).into(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_list_with_single_element() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::List::new(types::None::new(Position::fake()), Position::fake()),
                            List::new(
                                types::None::new(Position::fake()),
                                vec![ListElement::Single(
                                    Number::new(42.0, Position::fake()).into(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }

        #[test]
        fn fail_to_check_list_with_multiple_element() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::List::new(
                                    types::Number::new(Position::fake()),
                                    Position::fake()
                                )
                            )],
                            types::List::new(types::None::new(Position::fake()), Position::fake()),
                            List::new(
                                types::None::new(Position::fake()),
                                vec![ListElement::Multiple(
                                    Variable::new("x", Position::fake()).into(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake(),
                ))
            );
        }

        #[test]
        fn check_list_with_single_element_of_union() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::List::new(union_type.clone(), Position::fake()),
                        List::new(
                            union_type,
                            vec![ListElement::Single(None::new(Position::fake()).into())],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_list_with_multiple_element_of_union() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::List::new(types::None::new(Position::fake()), Position::fake()),
                        )],
                        types::List::new(union_type.clone(), Position::fake()),
                        List::new(
                            union_type,
                            vec![ListElement::Multiple(
                                Variable::new("x", Position::fake()).into(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_list_comprehension() {
            let element_type = types::None::new(Position::fake());

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::List::new(element_type.clone(), Position::fake()),
                        ListComprehension::new(
                            Some(element_type.clone().into()),
                            element_type.clone(),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![],
                                        element_type.clone(),
                                        Position::fake(),
                                    )
                                    .into(),
                                ),
                                Variable::new("x", Position::fake()),
                                vec![],
                                Position::fake(),
                            ),
                            "x",
                            List::new(element_type, vec![], Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_list_in_list_comprehension() {
            let element_type = types::None::new(Position::fake());

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::List::new(element_type.clone(), Position::fake()),
                            ListComprehension::new(
                                Some(element_type.clone().into()),
                                element_type.clone(),
                                Call::new(
                                    Some(
                                        types::Function::new(
                                            vec![],
                                            element_type.clone(),
                                            Position::fake(),
                                        )
                                        .into(),
                                    ),
                                    Variable::new("x", Position::fake()),
                                    vec![],
                                    Position::fake(),
                                ),
                                "x",
                                List::new(
                                    element_type,
                                    vec![ListElement::Single(
                                        Number::new(42.0, Position::fake()).into(),
                                    )],
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }
    }

    mod map {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn check_no_element() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Ok(()),
            );
        }

        #[test]
        fn check_single_element() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapEntry::new(
                                    None::new(Position::fake()),
                                    None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into()],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Ok(()),
            );
        }

        #[test]
        fn fail_to_check_key_type() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapEntry::new(
                                    Number::new(42.0, Position::fake()),
                                    None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into()],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                )),
            );
        }

        #[test]
        fn fail_to_check_value_type() {
            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapEntry::new(
                                    None::new(Position::fake()),
                                    Number::new(42.0, Position::fake()),
                                    Position::fake(),
                                )
                                .into()],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                )),
            );
        }

        #[test]
        fn check_multiple_elements() {
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", map_type.clone())],
                            map_type,
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapElement::Map(Variable::new("x", Position::fake()).into())],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Ok(()),
            );
        }

        #[test]
        fn check_multiple_elements_with_wrong_key_type() {
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Map::new(
                                    types::Number::new(Position::fake()),
                                    types::None::new(Position::fake()),
                                    Position::fake(),
                                )
                            )],
                            map_type,
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapElement::Map(Variable::new("x", Position::fake()).into())],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                )),
            );
        }

        #[test]
        fn check_multiple_elements_with_wrong_value_type() {
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Map::new(
                                    types::None::new(Position::fake()),
                                    types::Number::new(Position::fake()),
                                    Position::fake(),
                                )
                            )],
                            map_type,
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapElement::Map(Variable::new("x", Position::fake()).into())],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                )),
            );
        }

        #[test]
        fn check_key_removal() {
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            map_type,
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapElement::Removal(None::new(Position::fake()).into())],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Ok(()),
            );
        }

        #[test]
        fn check_key_removal_with_wrong_key_type() {
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            map_type,
                            Map::new(
                                types::None::new(Position::fake()),
                                types::None::new(Position::fake()),
                                vec![MapElement::Removal(
                                    Number::new(42.0, Position::fake()).into()
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                )),
            );
        }
    }

    mod if_list {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn check_first_variable() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::None::new(Position::fake()),
                        IfList::new(
                            Some(types::None::new(Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![],
                                        types::None::new(Position::fake()),
                                        Position::fake(),
                                    )
                                    .into(),
                                ),
                                Variable::new("y", Position::fake()),
                                vec![],
                                Position::fake(),
                            ),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_rest_variable() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type.clone())],
                        list_type,
                        IfList::new(
                            Some(types::None::new(Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Variable::new("ys", Position::fake()),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_union_type_result() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", list_type)],
                        types::Union::new(
                            types::None::new(Position::fake()),
                            types::Number::new(Position::fake()),
                            Position::fake(),
                        ),
                        IfList::new(
                            Some(types::None::new(Position::fake()).into()),
                            Variable::new("x", Position::fake()),
                            "y",
                            "ys",
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![],
                                        types::None::new(Position::fake()),
                                        Position::fake(),
                                    )
                                    .into(),
                                ),
                                Variable::new("y", Position::fake()),
                                vec![],
                                Position::fake(),
                            ),
                            Number::new(42.0, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_argument() {
            let list_type =
                types::List::new(types::Number::new(Position::fake()), Position::fake());

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type)],
                            types::None::new(Position::fake()),
                            IfList::new(
                                Some(types::None::new(Position::fake()).into()),
                                Variable::new("x", Position::fake()),
                                "y",
                                "ys",
                                Variable::new("y", Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }

        #[test]
        fn fail_to_check_result() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type)],
                            types::None::new(Position::fake()),
                            IfList::new(
                                Some(types::None::new(Position::fake()).into()),
                                Variable::new("x", Position::fake()),
                                "y",
                                "ys",
                                Variable::new("y", Position::fake()),
                                Number::new(42.0, Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }
    }

    mod if_map {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn check() {
            let map_type = types::Map::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", map_type.clone())],
                        types::None::new(Position::fake()),
                        IfMap::new(
                            Some(map_type.key().clone()),
                            Some(map_type.value().clone()),
                            "y",
                            Variable::new("x", Position::fake()),
                            Boolean::new(true, Position::fake()),
                            Variable::new("y", Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_union_type_result() {
            let map_type = types::Map::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", map_type.clone())],
                        types::Union::new(
                            types::Number::new(Position::fake()),
                            types::None::new(Position::fake()),
                            Position::fake(),
                        ),
                        IfMap::new(
                            Some(map_type.key().clone()),
                            Some(map_type.value().clone()),
                            "y",
                            Variable::new("x", Position::fake()),
                            Boolean::new(true, Position::fake()),
                            Variable::new("y", Position::fake()),
                            Number::new(42.0, Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn fail_to_check_map() {
            let map_type = types::Map::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Map::new(
                                    types::Number::new(Position::fake()),
                                    types::None::new(Position::fake()),
                                    Position::fake(),
                                )
                            )],
                            types::None::new(Position::fake()),
                            IfMap::new(
                                Some(map_type.key().clone()),
                                Some(map_type.value().clone()),
                                "y",
                                Variable::new("x", Position::fake()),
                                Boolean::new(true, Position::fake()),
                                None::new(Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }

        #[test]
        fn fail_to_check_key() {
            let map_type = types::Map::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", map_type.clone())],
                            types::None::new(Position::fake()),
                            IfMap::new(
                                Some(map_type.key().clone()),
                                Some(map_type.value().clone()),
                                "y",
                                Variable::new("x", Position::fake()),
                                Number::new(42.0, Position::fake()),
                                None::new(Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )]),
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }

        #[test]
        fn fail_to_check_result() {
            let map_type = types::Map::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                check_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", map_type.clone())],
                            types::None::new(Position::fake()),
                            IfMap::new(
                                Some(map_type.key().clone()),
                                Some(map_type.value().clone()),
                                "y",
                                Variable::new("x", Position::fake()),
                                Boolean::new(true, Position::fake()),
                                Number::new(42.0, Position::fake()),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                ),
                Err(AnalysisError::TypesNotMatched(
                    Position::fake(),
                    Position::fake()
                ))
            );
        }
    }

    mod type_coercion {
        use super::*;

        #[test]
        fn check_union() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        union_type.clone(),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_any() {
            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::Any::new(Position::fake()),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            types::Any::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_list() {
            let none_list_type =
                types::List::new(types::None::new(Position::fake()), Position::fake());
            let any_list_type =
                types::List::new(types::Any::new(Position::fake()), Position::fake());

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", none_list_type.clone())],
                        any_list_type.clone(),
                        TypeCoercion::new(
                            none_list_type,
                            any_list_type,
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }

        #[test]
        fn check_map() {
            let key_type = types::Union::new(
                types::ByteString::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let value_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let none_map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let union_map_type = types::Map::new(key_type, value_type, Position::fake());

            check_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", none_map_type.clone())],
                        union_map_type.clone(),
                        TypeCoercion::new(
                            none_map_type,
                            union_map_type,
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]),
            )
            .unwrap();
        }
    }
}
