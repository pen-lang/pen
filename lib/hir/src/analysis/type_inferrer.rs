use super::{
    context::AnalysisContext, module_environment, type_canonicalizer, type_difference_calculator,
    type_extractor, union_type_creator, AnalysisError,
};
use crate::{
    ir::*,
    types::{self, Type},
};

pub fn infer(context: &AnalysisContext, module: &Module) -> Result<Module, AnalysisError> {
    let variables = plist::FlailMap::new(module_environment::create(module));

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| infer_definition(context, definition, &variables))
            .collect::<Result<_, _>>()?,
        module.position().clone(),
    ))
}

fn infer_definition(
    context: &AnalysisContext,
    definition: &FunctionDefinition,
    variables: &plist::FlailMap<String, Type>,
) -> Result<FunctionDefinition, AnalysisError> {
    Ok(FunctionDefinition::new(
        definition.name(),
        definition.original_name(),
        infer_lambda(context, definition.lambda(), variables)?,
        definition.foreign_definition_configuration().cloned(),
        definition.is_public(),
        definition.position().clone(),
    ))
}

fn infer_lambda(
    context: &AnalysisContext,
    lambda: &Lambda,
    variables: &plist::FlailMap<String, Type>,
) -> Result<Lambda, AnalysisError> {
    Ok(Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        infer_expression(
            context,
            lambda.body(),
            &variables.insert_iter(
                lambda
                    .arguments()
                    .iter()
                    .map(|argument| (argument.name().into(), argument.type_().clone())),
            ),
        )?,
        lambda.position().clone(),
    ))
}

fn infer_expression(
    context: &AnalysisContext,
    expression: &Expression,
    variables: &plist::FlailMap<String, Type>,
) -> Result<Expression, AnalysisError> {
    let infer_expression =
        |expression, variables: &_| infer_expression(context, expression, variables);

    Ok(match expression {
        Expression::Call(call) => {
            if let Expression::BuiltInFunction(function) = call.function() {
                infer_built_in_call(context, call, function, variables)?.into()
            } else {
                let function = infer_expression(call.function(), variables)?;

                Call::new(
                    Some(type_extractor::extract_from_expression(
                        context, &function, variables,
                    )?),
                    function.clone(),
                    call.arguments()
                        .iter()
                        .map(|argument| infer_expression(argument, variables))
                        .collect::<Result<_, _>>()?,
                    call.position().clone(),
                )
                .into()
            }
        }
        Expression::If(if_) => {
            let then = infer_expression(if_.then(), variables)?;
            let else_ = infer_expression(if_.else_(), variables)?;

            If::new(
                infer_expression(if_.condition(), variables)?,
                then,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfList(if_) => {
            let list = infer_expression(if_.list(), variables)?;
            let type_ = type_extractor::extract_from_expression(context, &list, variables)?;
            let list_type = type_canonicalizer::canonicalize_list(&type_, context.types())?
                .ok_or(AnalysisError::ListExpected(type_))?;

            let then = infer_expression(
                if_.then(),
                &variables.insert_iter([
                    (
                        if_.first_name().into(),
                        types::Function::new(
                            vec![],
                            list_type.element().clone(),
                            if_.position().clone(),
                        )
                        .into(),
                    ),
                    (if_.rest_name().into(), list_type.clone().into()),
                ]),
            )?;
            let else_ = infer_expression(if_.else_(), variables)?;

            IfList::new(
                Some(list_type.element().clone()),
                list,
                if_.first_name(),
                if_.rest_name(),
                then,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfMap(if_) => {
            let map = infer_expression(if_.map(), variables)?;
            let key = infer_expression(if_.key(), variables)?;
            let type_ = type_extractor::extract_from_expression(context, &map, variables)?;
            let map_type = type_canonicalizer::canonicalize_map(&type_, context.types())?
                .ok_or(AnalysisError::MapExpected(type_))?;

            let then = infer_expression(
                if_.then(),
                &variables.insert(if_.name().into(), map_type.value().clone()),
            )?;
            let else_ = infer_expression(if_.else_(), variables)?;

            IfMap::new(
                Some(map_type.key().clone()),
                Some(map_type.value().clone()),
                if_.name(),
                map,
                key,
                then,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfType(if_) => {
            let argument = infer_expression(if_.argument(), variables)?;
            let branches = if_
                .branches()
                .iter()
                .map(|branch| -> Result<_, AnalysisError> {
                    Ok(IfTypeBranch::new(
                        branch.type_().clone(),
                        infer_expression(
                            branch.expression(),
                            &variables.insert(if_.name().into(), branch.type_().clone()),
                        )?,
                    ))
                })
                .collect::<Result<Vec<_>, _>>()?;

            let else_ = if_
                .else_()
                .map(|branch| -> Result<_, AnalysisError> {
                    let type_ = type_difference_calculator::calculate(
                        &type_extractor::extract_from_expression(context, &argument, variables)?,
                        &union_type_creator::create(
                            &if_.branches()
                                .iter()
                                .map(|branch| branch.type_().clone())
                                .collect::<Vec<_>>(),
                            if_.position(),
                        )
                        .unwrap(),
                        context.types(),
                    )?
                    .ok_or_else(|| AnalysisError::UnreachableCode(branch.position().clone()))?;

                    Ok(ElseBranch::new(
                        Some(type_.clone()),
                        infer_expression(
                            branch.expression(),
                            &variables.insert(if_.name().into(), type_),
                        )?,
                        branch.position().clone(),
                    ))
                })
                .transpose()?;

            IfType::new(
                if_.name(),
                argument,
                branches,
                else_,
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => infer_lambda(context, lambda, variables)?.into(),
        Expression::Let(let_) => {
            let bound_expression = infer_expression(let_.bound_expression(), variables)?;
            let bound_type =
                type_extractor::extract_from_expression(context, &bound_expression, variables)?;

            Let::new(
                let_.name().map(String::from),
                Some(bound_type.clone()),
                bound_expression,
                infer_expression(
                    let_.expression(),
                    &variables.insert_iter(let_.name().map(|name| (name.into(), bound_type))),
                )?,
                let_.position().clone(),
            )
            .into()
        }
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ListElement::Multiple(element) => {
                            ListElement::Multiple(infer_expression(element, variables)?)
                        }
                        ListElement::Single(element) => {
                            ListElement::Single(infer_expression(element, variables)?)
                        }
                    })
                })
                .collect::<Result<_, AnalysisError>>()?,
            list.position().clone(),
        )
        .into(),
        Expression::ListComprehension(comprehension) => {
            let mut variables = variables.clone();
            let mut branches = vec![];

            for branch in comprehension.branches() {
                let iteratee = infer_expression(branch.iteratee(), &variables)?;
                let type_ =
                    type_extractor::extract_from_expression(context, &iteratee, &variables)?;

                variables = match type_canonicalizer::canonicalize(&type_, context.types())? {
                    Type::List(list_type) => variables.insert(
                        branch.primary_name().into(),
                        types::Function::new(
                            vec![],
                            list_type.element().clone(),
                            comprehension.position().clone(),
                        )
                        .into(),
                    ),
                    Type::Map(map_type) => variables.insert_iter(
                        [(branch.primary_name().into(), map_type.key().clone())]
                            .into_iter()
                            .chain(
                                branch
                                    .secondary_name()
                                    .map(|name| (name.into(), map_type.value().clone())),
                            ),
                    ),
                    _ => return Err(AnalysisError::CollectionExpected(type_.clone())),
                };

                branches.push(ListComprehensionBranch::new(
                    Some(type_.clone()),
                    branch.primary_name(),
                    branch.secondary_name().map(String::from),
                    iteratee,
                    branch
                        .condition()
                        .map(|expression| infer_expression(expression, &variables))
                        .transpose()?,
                    branch.position().clone(),
                ));
            }

            ListComprehension::new(
                comprehension.type_().clone(),
                infer_expression(comprehension.element(), &variables)?,
                branches,
                comprehension.position().clone(),
            )
            .into()
        }
        Expression::Map(map) => Map::new(
            map.key_type().clone(),
            map.value_type().clone(),
            map.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        MapElement::Insertion(entry) => MapElement::Insertion(MapEntry::new(
                            infer_expression(entry.key(), variables)?,
                            infer_expression(entry.value(), variables)?,
                            entry.position().clone(),
                        )),
                        MapElement::Map(map) => MapElement::Map(infer_expression(map, variables)?),
                    })
                })
                .collect::<Result<_, AnalysisError>>()?,
            map.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => match operation {
            Operation::Addition(operation) => {
                let lhs = infer_expression(operation.lhs(), variables)?;

                AdditionOperation::new(
                    Some(type_canonicalizer::canonicalize(
                        &type_extractor::extract_from_expression(context, &lhs, variables)?,
                        context.types(),
                    )?),
                    lhs,
                    infer_expression(operation.rhs(), variables)?,
                    operation.position().clone(),
                )
                .into()
            }
            Operation::Arithmetic(operation) => ArithmeticOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Boolean(operation) => BooleanOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Equality(operation) => {
                let lhs = infer_expression(operation.lhs(), variables)?;
                let rhs = infer_expression(operation.rhs(), variables)?;

                EqualityOperation::new(
                    Some(
                        types::Union::new(
                            type_extractor::extract_from_expression(context, &lhs, variables)?,
                            type_extractor::extract_from_expression(context, &rhs, variables)?,
                            operation.position().clone(),
                        )
                        .into(),
                    ),
                    operation.operator(),
                    lhs,
                    rhs,
                    operation.position().clone(),
                )
                .into()
            }
            Operation::Not(operation) => NotOperation::new(
                infer_expression(operation.expression(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Order(operation) => OrderOperation::new(
                operation.operator(),
                infer_expression(operation.lhs(), variables)?,
                infer_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Try(operation) => {
                let position = operation.position();
                let expression = infer_expression(operation.expression(), variables)?;
                let type_ =
                    type_extractor::extract_from_expression(context, &expression, variables)?;

                TryOperation::new(
                    Some(
                        if let Some(type_) = type_difference_calculator::calculate(
                            &type_,
                            &types::Error::new(position.clone()).into(),
                            context.types(),
                        )? {
                            if type_.is_any() {
                                return Err(AnalysisError::UnionExpected(type_));
                            } else {
                                type_
                            }
                        } else {
                            return Err(AnalysisError::UnionExpected(type_));
                        },
                    ),
                    expression,
                    position.clone(),
                )
                .into()
            }
        },
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            construction
                .fields()
                .iter()
                .map(|field| {
                    Ok(RecordField::new(
                        field.name(),
                        infer_expression(field.expression(), variables)?,
                        field.position().clone(),
                    ))
                })
                .collect::<Result<_, AnalysisError>>()?,
            construction.position().clone(),
        )
        .into(),
        Expression::RecordDeconstruction(deconstruction) => {
            let record = infer_expression(deconstruction.record(), variables)?;

            RecordDeconstruction::new(
                Some(type_extractor::extract_from_expression(
                    context, &record, variables,
                )?),
                record,
                deconstruction.field_name(),
                deconstruction.position().clone(),
            )
            .into()
        }
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            infer_expression(update.record(), variables)?,
            update
                .fields()
                .iter()
                .map(|field| {
                    Ok(RecordField::new(
                        field.name(),
                        infer_expression(field.expression(), variables)?,
                        field.position().clone(),
                    ))
                })
                .collect::<Result<_, AnalysisError>>()?,
            update.position().clone(),
        )
        .into(),
        Expression::Thunk(thunk) => Thunk::new(
            Some(type_extractor::extract_from_expression(
                context,
                thunk.expression(),
                variables,
            )?),
            infer_expression(thunk.expression(), variables)?,
            thunk.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            infer_expression(coercion.argument(), variables)?,
            coercion.position().clone(),
        )
        .into(),
        Expression::Boolean(_)
        | Expression::BuiltInFunction(_)
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => expression.clone(),
    })
}

fn infer_built_in_call(
    context: &AnalysisContext,
    call: &Call,
    function: &BuiltInFunction,
    variables: &plist::FlailMap<String, Type>,
) -> Result<Call, AnalysisError> {
    let position = call.position();
    let arguments = call
        .arguments()
        .iter()
        .map(|argument| infer_expression(context, argument, variables))
        .collect::<Result<Vec<_>, _>>()?;
    let argument_types = arguments
        .iter()
        .map(|argument| type_extractor::extract_from_expression(context, argument, variables))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(Call::new(
        Some(
            match function.name() {
                BuiltInFunctionName::Debug => types::Function::new(
                    vec![types::Any::new(position.clone()).into()],
                    types::None::new(position.clone()),
                    position.clone(),
                ),
                BuiltInFunctionName::Delete => {
                    let Some(result_type) =
                        argument_types.first().cloned() else {
                            return Err(AnalysisError::ArgumentCount(position.clone()));
                        };

                    types::Function::new(argument_types, result_type, position.clone())
                }
                BuiltInFunctionName::Error => types::Function::new(
                    vec![types::Any::new(position.clone()).into()],
                    types::Error::new(position.clone()),
                    position.clone(),
                ),
                BuiltInFunctionName::Race => {
                    let argument_type = argument_types
                        .first()
                        .ok_or_else(|| AnalysisError::ArgumentCount(position.clone()))?;
                    let argument_type =
                        type_canonicalizer::canonicalize_list(argument_type, context.types())?
                            .ok_or_else(|| AnalysisError::ListExpected(argument_type.clone()))?;

                    types::Function::new(
                        argument_types,
                        argument_type.element().clone(),
                        position.clone(),
                    )
                }
                BuiltInFunctionName::ReflectDebug => types::Function::new(
                    vec![types::Any::new(position.clone()).into()],
                    types::ByteString::new(position.clone()),
                    position.clone(),
                ),
                BuiltInFunctionName::ReflectEqual => types::Function::new(
                    vec![
                        types::Any::new(position.clone()).into(),
                        types::Any::new(position.clone()).into(),
                    ],
                    types::Union::new(
                        types::Boolean::new(position.clone()),
                        types::None::new(position.clone()),
                        position.clone(),
                    ),
                    position.clone(),
                ),
                BuiltInFunctionName::Size => types::Function::new(
                    argument_types,
                    types::Number::new(position.clone()),
                    position.clone(),
                ),
                BuiltInFunctionName::Source => types::Function::new(
                    vec![types::Error::new(position.clone()).into()],
                    types::Any::new(position.clone()),
                    position.clone(),
                ),
                BuiltInFunctionName::Spawn => {
                    let result_type = argument_types
                        .first()
                        .cloned()
                        .ok_or_else(|| AnalysisError::ArgumentCount(position.clone()))?;

                    types::Function::new(argument_types, result_type, position.clone())
                }
            }
            .into(),
        ),
        call.function().clone(),
        arguments,
        position.clone(),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analysis::type_collector,
        test::{FunctionDefinitionFake, ModuleFake, RecordFake},
    };
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn infer_module(module: &Module) -> Result<Module, AnalysisError> {
        infer(
            &AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_record_fields(module),
            ),
            module,
        )
    }

    #[test]
    fn infer_empty_module() {
        infer_module(&Module::empty()).unwrap();
    }

    #[test]
    fn infer_call() {
        assert_eq!(
            infer_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Call::new(
                            None,
                            Variable::new("x", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ],)),
            Ok(
                Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    Position::fake()
                                )
                                .into()
                            ),
                            Variable::new("x", Position::fake()),
                            vec![],
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)
            )
        );
    }

    #[test]
    fn infer_equality_operation() {
        assert_eq!(
            infer_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        EqualityOperation::new(
                            None,
                            EqualityOperator::Equal,
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ],)),
            Ok(
                Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        EqualityOperation::new(
                            Some(
                                types::Union::new(
                                    types::None::new(Position::fake()),
                                    types::None::new(Position::fake()),
                                    Position::fake()
                                )
                                .into()
                            ),
                            EqualityOperator::Equal,
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)
            )
        );
    }

    #[test]
    fn infer_let() {
        assert_eq!(
            infer_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Let::new(
                            Some("x".into()),
                            None,
                            None::new(Position::fake()),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )
            ],)),
            Ok(
                Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
                )],)
            )
        );
    }

    #[test]
    fn infer_let_with_call() {
        let declaration = FunctionDeclaration::new(
            "f",
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            infer_module(
                &Module::empty()
                    .set_function_declarations(vec![declaration.clone()])
                    .set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Let::new(
                                Some("x".into()),
                                None,
                                Call::new(
                                    None,
                                    Variable::new("f", Position::fake()),
                                    vec![],
                                    Position::fake()
                                ),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
            ),
            Ok(Module::empty()
                .set_function_declarations(vec![declaration.clone()])
                .set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        types::None::new(Position::fake()),
                        Let::new(
                            Some("x".into()),
                            Some(types::None::new(Position::fake()).into()),
                            Call::new(
                                Some(declaration.type_().clone().into()),
                                Variable::new("f", Position::fake()),
                                vec![],
                                Position::fake()
                            ),
                            Variable::new("x", Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    mod list_comprehension {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer() {
            let element_type = types::None::new(Position::fake());
            let list_type = types::List::new(element_type.clone(), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            ListComprehension::new(
                                element_type.clone(),
                                Let::new(
                                    Some("y".into()),
                                    None,
                                    Call::new(
                                        None,
                                        Variable::new("x", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Variable::new("y", Position::fake()),
                                    Position::fake(),
                                ),
                                vec![ListComprehensionBranch::new(
                                    None,
                                    "x",
                                    None,
                                    List::new(element_type.clone(), vec![], Position::fake()),
                                    None,
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            ListComprehension::new(
                                element_type.clone(),
                                Let::new(
                                    Some("y".into()),
                                    Some(element_type.clone().into()),
                                    Call::new(
                                        Some(
                                            types::Function::new(
                                                vec![],
                                                element_type.clone(),
                                                Position::fake()
                                            )
                                            .into()
                                        ),
                                        Variable::new("x", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Variable::new("y", Position::fake()),
                                    Position::fake(),
                                ),
                                vec![ListComprehensionBranch::new(
                                    Some(list_type.into()),
                                    "x",
                                    None,
                                    List::new(element_type, vec![], Position::fake()),
                                    None,
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_two_branches() {
            let element_type = types::None::new(Position::fake());
            let list_type = types::List::new(element_type.clone(), Position::fake());
            let nested_list_type = types::List::new(list_type.clone(), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            ListComprehension::new(
                                element_type.clone(),
                                Let::new(
                                    Some("y".into()),
                                    None,
                                    Call::new(
                                        None,
                                        Variable::new("x", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Variable::new("y", Position::fake()),
                                    Position::fake(),
                                ),
                                vec![
                                    ListComprehensionBranch::new(
                                        None,
                                        "xs",
                                        None,
                                        List::new(list_type.clone(), vec![], Position::fake()),
                                        None,
                                        Position::fake(),
                                    ),
                                    ListComprehensionBranch::new(
                                        None,
                                        "x",
                                        None,
                                        Call::new(
                                            None,
                                            Variable::new("xs", Position::fake()),
                                            vec![],
                                            Position::fake()
                                        ),
                                        None,
                                        Position::fake(),
                                    ),
                                ],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            ListComprehension::new(
                                element_type.clone(),
                                Let::new(
                                    Some("y".into()),
                                    Some(element_type.clone().into()),
                                    Call::new(
                                        Some(
                                            types::Function::new(
                                                vec![],
                                                element_type,
                                                Position::fake()
                                            )
                                            .into()
                                        ),
                                        Variable::new("x", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Variable::new("y", Position::fake()),
                                    Position::fake(),
                                ),
                                vec![
                                    ListComprehensionBranch::new(
                                        Some(nested_list_type.into()),
                                        "xs",
                                        None,
                                        List::new(list_type.clone(), vec![], Position::fake()),
                                        None,
                                        Position::fake(),
                                    ),
                                    ListComprehensionBranch::new(
                                        Some(list_type.clone().into()),
                                        "x",
                                        None,
                                        Call::new(
                                            Some(
                                                types::Function::new(
                                                    vec![],
                                                    list_type,
                                                    Position::fake()
                                                )
                                                .into()
                                            ),
                                            Variable::new("xs", Position::fake()),
                                            vec![],
                                            Position::fake()
                                        ),
                                        None,
                                        Position::fake(),
                                    ),
                                ],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_condition() {
            let list_type =
                types::List::new(types::Boolean::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            ListComprehension::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                                vec![ListComprehensionBranch::new(
                                    None,
                                    "x",
                                    None,
                                    List::new(
                                        list_type.element().clone(),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Some(Let::new(
                                        Some("y".into()),
                                        None,
                                        Call::new(
                                            None,
                                            Variable::new("x", Position::fake()),
                                            vec![],
                                            Position::fake()
                                        ),
                                        Variable::new("y", Position::fake()),
                                        Position::fake(),
                                    ).into()),
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            ListComprehension::new(
                                types::None::new(Position::fake()),
                                None::new(Position::fake()),
                                vec![ListComprehensionBranch::new(
                                    Some(list_type.clone().into()),
                                    "x",
                                    None,
                                    List::new(
                                        list_type.element().clone(),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Some(
                                        Let::new(
                                            Some("y".into()),
                                            Some(list_type.element().clone()),
                                            Call::new(
                                                None,
                                                Variable::new("x", Position::fake()),
                                                vec![],
                                                Position::fake()
                                            ),
                                            Variable::new("y", Position::fake()),
                                            Position::fake(),
                                        )
                                        .into()
                                    ),
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_map_iteratee() {
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let element_type = types::None::new(Position::fake());
            let list_type = types::List::new(element_type.clone(), Position::fake());
            let empty_map = Map::new(
                map_type.key().clone(),
                map_type.value().clone(),
                vec![],
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            ListComprehension::new(
                                element_type.clone(),
                                Let::new(
                                    Some("x".into()),
                                    None,
                                    Variable::new("k", Position::fake()),
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                ),
                                vec![ListComprehensionBranch::new(
                                    None,
                                    "k",
                                    Some("v".into()),
                                    empty_map.clone(),
                                    None,
                                    Position::fake()
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type,
                            ListComprehension::new(
                                element_type,
                                Let::new(
                                    Some("x".into()),
                                    Some(map_type.key().clone()),
                                    Variable::new("k", Position::fake()),
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                ),
                                vec![ListComprehensionBranch::new(
                                    Some(map_type.clone().into()),
                                    "k",
                                    Some("v".into()),
                                    empty_map,
                                    None,
                                    Position::fake(),
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }
    }

    #[test]
    fn infer_record_deconstruction() {
        let type_definition = TypeDefinition::new(
            "r",
            "",
            vec![types::RecordField::new(
                "x",
                types::None::new(Position::fake()),
            )],
            false,
            false,
            false,
            Position::fake(),
        );

        assert_eq!(
            infer_module(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", types::Record::fake("r"))],
                            types::None::new(Position::fake()),
                            RecordDeconstruction::new(
                                None,
                                Variable::new("x", Position::fake()),
                                "x",
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_function_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new("x", types::Record::fake("r"))],
                        types::None::new(Position::fake()),
                        RecordDeconstruction::new(
                            Some(types::Record::fake("r").into()),
                            Variable::new("x", Position::fake()),
                            "x",
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    mod function_definition {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn check_overridden_function_declaration() {
            let function_declaration = FunctionDeclaration::new(
                "f",
                types::Function::new(
                    vec![types::Number::new(Position::fake()).into()],
                    types::Number::new(Position::fake()),
                    Position::fake(),
                ),
                Position::fake(),
            );

            assert_eq!(
                infer_module(
                    &Module::empty()
                        .set_function_declarations(vec![function_declaration.clone()])
                        .set_function_definitions(vec![FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![],
                                types::None::new(Position::fake()),
                                Call::new(
                                    None,
                                    Variable::new("f", Position::fake()),
                                    vec![],
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false,
                        )]),
                ),
                Ok(Module::empty()
                    .set_function_declarations(vec![function_declaration])
                    .set_function_definitions(vec![FunctionDefinition::fake(
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
                    )]))
            )
        }

        #[test]
        fn infer_thunk() {
            let none_type = types::None::new(Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            none_type.clone(),
                            Thunk::new(None, None::new(Position::fake()), Position::fake()),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            none_type.clone(),
                            Thunk::new(
                                Some(none_type.into()),
                                None::new(Position::fake()),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }
    }

    mod if_type {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer_else_branch_type_of_none() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let branches = vec![IfTypeBranch::new(
                types::Number::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "x",
                                Variable::new("x", Position::fake()),
                                branches.clone(),
                                Some(ElseBranch::new(
                                    None,
                                    None::new(Position::fake()),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type)],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "x",
                                Variable::new("x", Position::fake()),
                                branches,
                                Some(ElseBranch::new(
                                    Some(types::None::new(Position::fake()).into()),
                                    None::new(Position::fake()),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_else_branch_type_of_union() {
            let union_type = types::Union::new(
                types::Union::new(
                    types::Number::new(Position::fake()),
                    types::Boolean::new(Position::fake()),
                    Position::fake(),
                ),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let branches = vec![IfTypeBranch::new(
                types::Number::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "x",
                                Variable::new("x", Position::fake()),
                                branches.clone(),
                                Some(ElseBranch::new(
                                    None,
                                    None::new(Position::fake()),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type)],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "x",
                                Variable::new("x", Position::fake()),
                                branches,
                                Some(ElseBranch::new(
                                    Some(
                                        types::Union::new(
                                            types::Boolean::new(Position::fake()),
                                            types::None::new(Position::fake()),
                                            Position::fake(),
                                        )
                                        .into()
                                    ),
                                    None::new(Position::fake()),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_else_branch_type_with_bound_variable() {
            let function_type =
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());
            let union_type = types::Union::new(
                function_type.clone(),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let branches = vec![IfTypeBranch::new(
                types::None::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "y",
                                Variable::new("x", Position::fake()),
                                branches.clone(),
                                Some(ElseBranch::new(
                                    None,
                                    Call::new(
                                        None,
                                        Variable::new("y", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type)],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "y",
                                Variable::new("x", Position::fake()),
                                branches,
                                Some(ElseBranch::new(
                                    Some(function_type.clone().into()),
                                    Call::new(
                                        Some(function_type.into()),
                                        Variable::new("y", Position::fake()),
                                        vec![],
                                        Position::fake()
                                    ),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_else_branch_type_of_any() {
            let any_type = types::Any::new(Position::fake());
            let branches = vec![IfTypeBranch::new(
                types::Number::new(Position::fake()),
                None::new(Position::fake()),
            )];

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", any_type.clone())],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "x",
                                Variable::new("x", Position::fake()),
                                branches.clone(),
                                Some(ElseBranch::new(
                                    None,
                                    None::new(Position::fake()),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", any_type.clone())],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "x",
                                Variable::new("x", Position::fake()),
                                branches,
                                Some(ElseBranch::new(
                                    Some(any_type.into()),
                                    None::new(Position::fake()),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn fail_to_infer_else_branch_type_due_to_unreachable_code() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", union_type)],
                            types::None::new(Position::fake()),
                            IfType::new(
                                "x",
                                Variable::new("x", Position::fake()),
                                vec![
                                    IfTypeBranch::new(
                                        types::Number::new(Position::fake()),
                                        None::new(Position::fake()),
                                    ),
                                    IfTypeBranch::new(
                                        types::None::new(Position::fake()),
                                        None::new(Position::fake()),
                                    )
                                ],
                                Some(ElseBranch::new(
                                    None,
                                    None::new(Position::fake()),
                                    Position::fake()
                                )),
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Err(AnalysisError::UnreachableCode(Position::fake()))
            );
        }
    }

    mod try_operation {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer() {
            let union_type = types::Union::new(
                types::None::new(Position::fake()),
                types::Error::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", union_type.clone())],
                            union_type.clone(),
                            TryOperation::new(
                                None,
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
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
                    )],)
                )
            );
        }

        #[test]
        fn fail_to_infer_with_error() {
            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", types::Error::new(Position::fake()))],
                            types::Error::new(Position::fake()),
                            TryOperation::new(
                                None,
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Err(AnalysisError::UnionExpected(
                    types::Error::new(Position::fake()).into()
                ))
            );
        }
    }

    mod if_list {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            types::None::new(Position::fake()),
                            IfList::new(
                                None,
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
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
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
                    )],)
                )
            );
        }

        #[test]
        fn infer_with_first_name_in_let() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            types::None::new(Position::fake()),
                            IfList::new(
                                None,
                                Variable::new("x", Position::fake()),
                                "y",
                                "ys",
                                Let::new(
                                    Some("z".into()),
                                    None,
                                    Variable::new("y", Position::fake()),
                                    Variable::new("z", Position::fake()),
                                    Position::fake()
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", list_type)],
                            types::None::new(Position::fake()),
                            IfList::new(
                                Some(types::None::new(Position::fake()).into()),
                                Variable::new("x", Position::fake()),
                                "y",
                                "ys",
                                Let::new(
                                    Some("z".into()),
                                    Some(
                                        types::Function::new(
                                            vec![],
                                            types::None::new(Position::fake()),
                                            Position::fake()
                                        )
                                        .into()
                                    ),
                                    Variable::new("y", Position::fake()),
                                    Variable::new("z", Position::fake()),
                                    Position::fake()
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_with_rest_name_in_let() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            types::None::new(Position::fake()),
                            IfList::new(
                                None,
                                Variable::new("x", Position::fake()),
                                "y",
                                "ys",
                                Let::new(
                                    Some("z".into()),
                                    None,
                                    Variable::new("ys", Position::fake()),
                                    Variable::new("z", Position::fake()),
                                    Position::fake()
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", list_type)],
                            types::None::new(Position::fake()),
                            IfList::new(
                                Some(types::None::new(Position::fake()).into()),
                                Variable::new("x", Position::fake()),
                                "y",
                                "ys",
                                Let::new(
                                    Some("z".into()),
                                    Some(
                                        types::List::new(
                                            types::None::new(Position::fake()),
                                            Position::fake()
                                        )
                                        .into()
                                    ),
                                    Variable::new("ys", Position::fake()),
                                    Variable::new("z", Position::fake()),
                                    Position::fake()
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }
    }

    mod if_map {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer() {
            let map_type = types::Map::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", map_type.clone())],
                            types::None::new(Position::fake()),
                            IfMap::new(
                                None,
                                None,
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
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", map_type)],
                            types::None::new(Position::fake()),
                            IfMap::new(
                                Some(types::Boolean::new(Position::fake()).into()),
                                Some(types::None::new(Position::fake()).into()),
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
                    )],)
                )
            );
        }

        #[test]
        fn infer_with_name() {
            let map_type = types::Map::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", map_type.clone())],
                            types::None::new(Position::fake()),
                            IfMap::new(
                                None,
                                None,
                                "y",
                                Variable::new("x", Position::fake()),
                                Boolean::new(true, Position::fake()),
                                Let::new(
                                    Some("z".into()),
                                    None,
                                    Variable::new("y", Position::fake()),
                                    Variable::new("z", Position::fake()),
                                    Position::fake()
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", map_type)],
                            types::None::new(Position::fake()),
                            IfMap::new(
                                Some(types::Boolean::new(Position::fake()).into()),
                                Some(types::None::new(Position::fake()).into()),
                                "y",
                                Variable::new("x", Position::fake()),
                                Boolean::new(true, Position::fake()),
                                Let::new(
                                    Some("z".into()),
                                    Some(types::None::new(Position::fake()).into()),
                                    Variable::new("y", Position::fake()),
                                    Variable::new("z", Position::fake()),
                                    Position::fake()
                                ),
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }
    }

    mod built_in_call {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn infer_debug() {
            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Call::new(
                                None,
                                BuiltInFunction::new(BuiltInFunctionName::Debug, Position::fake()),
                                vec![None::new(Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::None::new(Position::fake()),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![types::Any::new(Position::fake()).into()],
                                        types::None::new(Position::fake()),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(BuiltInFunctionName::Debug, Position::fake()),
                                vec![None::new(Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn infer_delete() {
            let map_type = types::Map::new(
                types::ByteString::new(Position::fake()),
                types::Number::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            map_type.clone(),
                            Call::new(
                                None,
                                BuiltInFunction::new(BuiltInFunctionName::Delete, Position::fake()),
                                vec![Map::new(
                                    map_type.key().clone(),
                                    map_type.value().clone(),
                                    vec![],
                                    Position::fake()
                                )
                                .into(), ByteString::new("", Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            map_type.clone(),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![
                                            map_type.clone().into(),
                                            types::ByteString::new(Position::fake()).into()
                                        ],
                                        map_type.clone(),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(BuiltInFunctionName::Delete, Position::fake()),
                                vec![
                                    Map::new(
                                        map_type.key().clone(),
                                        map_type.value().clone(),
                                        vec![],
                                        Position::fake()
                                    )
                                    .into(),
                                    ByteString::new("", Position::fake()).into()
                                ],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn infer_delete_key_from_key_argument() {
            let map_type = types::Map::new(
                types::Union::new(
                    types::ByteString::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake(),
                ),
                types::Number::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            map_type.clone(),
                            Call::new(
                                None,
                                BuiltInFunction::new(BuiltInFunctionName::Delete, Position::fake()),
                                vec![Map::new(
                                    map_type.key().clone(),
                                    map_type.value().clone(),
                                    vec![],
                                    Position::fake()
                                )
                                .into(), None::new( Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            map_type.clone(),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![
                                            map_type.clone().into(),
                                            types::None::new(Position::fake()).into()
                                        ],
                                        map_type.clone(),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(BuiltInFunctionName::Delete, Position::fake()),
                                vec![
                                    Map::new(
                                        map_type.key().clone(),
                                        map_type.value().clone(),
                                        vec![],
                                        Position::fake()
                                    )
                                    .into(),
                                    None::new(Position::fake()).into()
                                ],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn infer_race() {
            let list_type = types::List::new(
                types::List::new(types::None::new(Position::fake()), Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            list_type.element().clone(),
                            Call::new(
                                None,
                                BuiltInFunction::new(BuiltInFunctionName::Race, Position::fake()),
                                vec![Variable::new("x", Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            list_type.element().clone(),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![list_type.clone().into()],
                                        list_type.element().clone(),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(BuiltInFunctionName::Race, Position::fake()),
                                vec![Variable::new("x", Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn infer_reflect_debug() {
            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::ByteString::new(Position::fake()),
                            Call::new(
                                None,
                                BuiltInFunction::new(
                                    BuiltInFunctionName::ReflectDebug,
                                    Position::fake()
                                ),
                                vec![None::new(Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            types::ByteString::new(Position::fake()),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![types::Any::new(Position::fake()).into()],
                                        types::ByteString::new(Position::fake()),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(
                                    BuiltInFunctionName::ReflectDebug,
                                    Position::fake()
                                ),
                                vec![None::new(Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn infer_reflect_equal() {
            let result_type = types::Union::new(
                types::Boolean::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            result_type.clone(),
                            Call::new(
                                None,
                                BuiltInFunction::new(
                                    BuiltInFunctionName::ReflectEqual,
                                    Position::fake()
                                ),
                                vec![
                                    None::new(Position::fake()).into(),
                                    None::new(Position::fake()).into()
                                ],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            result_type.clone(),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![
                                            types::Any::new(Position::fake()).into(),
                                            types::Any::new(Position::fake()).into()
                                        ],
                                        result_type,
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(
                                    BuiltInFunctionName::ReflectEqual,
                                    Position::fake()
                                ),
                                vec![
                                    None::new(Position::fake()).into(),
                                    None::new(Position::fake()).into()
                                ],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn infer_size() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            types::Number::new(Position::fake()),
                            Call::new(
                                None,
                                BuiltInFunction::new(BuiltInFunctionName::Size, Position::fake()),
                                vec![Variable::new("x", Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            types::Number::new(Position::fake()),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![list_type.into()],
                                        types::Number::new(Position::fake()),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(BuiltInFunctionName::Size, Position::fake()),
                                vec![Variable::new("x", Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                )
            );
        }

        #[test]
        fn infer_spawn() {
            let function_type =
                types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            function_type.clone(),
                            Call::new(
                                None,
                                BuiltInFunction::new(BuiltInFunctionName::Spawn, Position::fake()),
                                vec![Lambda::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )
                ],)),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            function_type.clone(),
                            Call::new(
                                Some(
                                    types::Function::new(
                                        vec![function_type.clone().into()],
                                        function_type,
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::new(BuiltInFunctionName::Spawn, Position::fake()),
                                vec![Lambda::new(
                                    vec![],
                                    types::None::new(Position::fake()),
                                    None::new(Position::fake()),
                                    Position::fake(),
                                )
                                .into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }
    }
}
