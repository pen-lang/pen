use super::{
    context::AnalysisContext, module_environment_creator, type_canonicalizer,
    type_difference_calculator, type_extractor, union_type_creator, AnalysisError,
};
use crate::{
    ir::*,
    types::{self, Type},
};
use fnv::FnvHashMap;

pub fn infer(context: &AnalysisContext, module: &Module) -> Result<Module, AnalysisError> {
    let variables = module_environment_creator::create(module);

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
    variables: &FnvHashMap<String, Type>,
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
    variables: &FnvHashMap<String, Type>,
) -> Result<Lambda, AnalysisError> {
    Ok(Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        infer_expression(
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
        lambda.position().clone(),
    ))
}

fn infer_expression(
    context: &AnalysisContext,
    expression: &Expression,
    variables: &FnvHashMap<String, Type>,
) -> Result<Expression, AnalysisError> {
    let infer_expression =
        |expression, variables: &_| infer_expression(context, expression, variables);

    Ok(match expression {
        Expression::BuiltInCall(call) => {
            let position = call.position();
            let arguments = call
                .arguments()
                .iter()
                .map(|argument| infer_expression(argument, variables))
                .collect::<Result<Vec<_>, _>>()?;
            let argument_types = arguments
                .iter()
                .map(|argument| {
                    type_extractor::extract_from_expression(context, argument, variables)
                })
                .collect::<Result<Vec<_>, _>>()?;

            BuiltInCall::new(
                Some(
                    match call.function() {
                        BuiltInFunction::Debug => types::Function::new(
                            vec![types::ByteString::new(position.clone()).into()],
                            types::None::new(position.clone()),
                            position.clone(),
                        ),
                        BuiltInFunction::Size => types::Function::new(
                            argument_types,
                            types::Number::new(position.clone()),
                            position.clone(),
                        ),
                        BuiltInFunction::Spawn => {
                            let result_type = argument_types.first().cloned().ok_or_else(|| {
                                AnalysisError::WrongArgumentCount(position.clone())
                            })?;

                            types::Function::new(argument_types, result_type, position.clone())
                        }
                    }
                    .into(),
                ),
                call.function(),
                arguments,
                position.clone(),
            )
            .into()
        }
        Expression::Call(call) => {
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
            let list_type = type_canonicalizer::canonicalize_list(
                &type_extractor::extract_from_expression(context, &list, variables)?,
                context.types(),
            )?
            .ok_or_else(|| AnalysisError::ListExpected(if_.list().position().clone()))?;

            let then = infer_expression(
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
                        (if_.rest_name().into(), list_type.clone().into()),
                    ])
                    .collect(),
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
            let map_type = type_canonicalizer::canonicalize_map(
                &type_extractor::extract_from_expression(context, &map, variables)?,
                context.types(),
            )?
            .ok_or_else(|| AnalysisError::MapExpected(if_.map().position().clone()))?;

            let then = infer_expression(
                if_.then(),
                &variables
                    .clone()
                    .into_iter()
                    .chain([(if_.name().into(), map_type.value().clone())])
                    .collect(),
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
                            &variables
                                .clone()
                                .into_iter()
                                .chain([(if_.name().into(), branch.type_().clone())])
                                .collect(),
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
                            &variables
                                .clone()
                                .into_iter()
                                .chain([(if_.name().into(), type_)])
                                .collect(),
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
                    &variables
                        .clone()
                        .into_iter()
                        .chain(let_.name().map(|name| (name.into(), bound_type)))
                        .collect(),
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
            let list = infer_expression(comprehension.list(), variables)?;
            let list_type = type_canonicalizer::canonicalize_list(
                &type_extractor::extract_from_expression(context, &list, variables)?,
                context.types(),
            )?
            .ok_or_else(|| AnalysisError::ListExpected(comprehension.list().position().clone()))?;

            ListComprehension::new(
                Some(list_type.element().clone()),
                comprehension.output_type().clone(),
                infer_expression(
                    comprehension.element(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([(
                            comprehension.element_name().into(),
                            types::Function::new(
                                vec![],
                                list_type.element().clone(),
                                comprehension.position().clone(),
                            )
                            .into(),
                        )])
                        .collect(),
                )?,
                comprehension.element_name(),
                list,
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
                        MapElement::Removal(key) => {
                            MapElement::Removal(infer_expression(key, variables)?)
                        }
                    })
                })
                .collect::<Result<_, AnalysisError>>()?,
            map.position().clone(),
        )
        .into(),
        Expression::MapIterationComprehension(comprehension) => {
            let map = infer_expression(comprehension.map(), variables)?;
            let map_type = type_canonicalizer::canonicalize_map(
                &type_extractor::extract_from_expression(context, &map, variables)?,
                context.types(),
            )?
            .ok_or_else(|| AnalysisError::MapExpected(comprehension.map().position().clone()))?;

            MapIterationComprehension::new(
                Some(map_type.key().clone()),
                Some(map_type.value().clone()),
                comprehension.element_type().clone(),
                infer_expression(
                    comprehension.element(),
                    &variables
                        .clone()
                        .into_iter()
                        .chain([
                            (comprehension.key_name().into(), map_type.key().clone()),
                            (comprehension.value_name().into(), map_type.value().clone()),
                        ])
                        .collect(),
                )?,
                comprehension.key_name(),
                comprehension.value_name(),
                map,
                comprehension.position().clone(),
            )
            .into()
        }
        Expression::Operation(operation) => match operation {
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

                TryOperation::new(
                    Some(
                        if let Some(type_) = type_difference_calculator::calculate(
                            &type_extractor::extract_from_expression(
                                context,
                                &expression,
                                variables,
                            )?,
                            context.error_type()?,
                            context.types(),
                        )? {
                            if type_.is_any() {
                                return Err(AnalysisError::UnionExpected(
                                    expression.position().clone(),
                                ));
                            } else {
                                type_
                            }
                        } else {
                            return Err(AnalysisError::UnionExpected(
                                expression.position().clone(),
                            ));
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
        | Expression::None(_)
        | Expression::Number(_)
        | Expression::String(_)
        | Expression::Variable(_) => expression.clone(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analysis::type_collector,
        test::{FunctionDefinitionFake, ModuleFake, TypeDefinitionFake},
    };
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn infer_module(module: &Module) -> Result<Module, AnalysisError> {
        infer(
            &AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_records(module),
                Some(types::Record::new("error", Position::fake()).into()),
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
            infer_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                )],)
            ),
            Ok(
                Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
            infer_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                )],)
            ),
            Ok(
                Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
            infer_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                )],)
            ),
            Ok(
                Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    .set_declarations(vec![declaration.clone()])
                    .set_definitions(vec![FunctionDefinition::fake(
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
                .set_declarations(vec![declaration.clone()])
                .set_definitions(vec![FunctionDefinition::fake(
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

    #[test]
    fn infer_list_comprehension() {
        let element_type = types::None::new(Position::fake());
        let list_type = types::List::new(element_type.clone(), Position::fake());

        assert_eq!(
            infer_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        list_type.clone(),
                        ListComprehension::new(
                            None,
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
                            "x",
                            List::new(element_type.clone(), vec![], Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])
            ),
            Ok(
                Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        list_type,
                        ListComprehension::new(
                            Some(element_type.clone().into()),
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
                            "x",
                            List::new(element_type, vec![], Position::fake()),
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
    fn infer_map_comprehension() {
        let key_type = types::None::new(Position::fake());
        let value_type = types::None::new(Position::fake());
        let element_type = types::None::new(Position::fake());
        let list_type = types::List::new(element_type.clone(), Position::fake());
        let empty_map = Map::new(
            key_type.clone(),
            value_type.clone(),
            vec![],
            Position::fake(),
        );

        assert_eq!(
            infer_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        list_type.clone(),
                        MapIterationComprehension::new(
                            None,
                            None,
                            element_type.clone(),
                            Let::new(
                                Some("x".into()),
                                None,
                                Variable::new("k", Position::fake()),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            "k",
                            "v",
                            empty_map.clone(),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])
            ),
            Ok(
                Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        list_type,
                        MapIterationComprehension::new(
                            Some(key_type.clone().into()),
                            Some(value_type.into()),
                            element_type,
                            Let::new(
                                Some("x".into()),
                                Some(key_type.into()),
                                Variable::new("k", Position::fake()),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            "k",
                            "v",
                            empty_map,
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
                    .set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new(
                                "x",
                                types::Record::new("r", Position::fake())
                            )],
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
                .set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![Argument::new(
                            "x",
                            types::Record::new("r", Position::fake())
                        )],
                        types::None::new(Position::fake()),
                        RecordDeconstruction::new(
                            Some(types::Record::new("r", Position::fake()).into()),
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

    #[test]
    fn infer_thunk() {
        let none_type = types::None::new(Position::fake());

        assert_eq!(
            infer_module(
                &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                    "x",
                    Lambda::new(
                        vec![],
                        none_type.clone(),
                        Thunk::new(None, None::new(Position::fake()), Position::fake()),
                        Position::fake(),
                    ),
                    false,
                )])
            ),
            Ok(
                Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )],)
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )],)
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )],)
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )],)
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )],)
                ),
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
                types::Reference::new("error", Position::fake()),
                Position::fake(),
            );
            let module = Module::empty().set_type_definitions(vec![TypeDefinition::fake(
                "error",
                vec![],
                false,
                false,
                false,
            )]);

            assert_eq!(
                infer_module(&module.set_definitions(vec![FunctionDefinition::fake(
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
                )])),
                Ok(module.set_definitions(vec![FunctionDefinition::fake(
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
                )],))
            );
        }

        #[test]
        fn fail_to_infer_with_error() {
            let error_type = types::Reference::new("error", Position::fake());

            assert_eq!(
                infer_module(
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
                                vec![Argument::new("x", error_type.clone())],
                                error_type,
                                TryOperation::new(
                                    None,
                                    Variable::new("x", Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false,
                        )],)
                ),
                Err(AnalysisError::UnionExpected(Position::fake()))
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )])
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )])
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )])
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )])
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
                    )])
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
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
        fn infer_size() {
            let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());

            assert_eq!(
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            types::Number::new(Position::fake()),
                            BuiltInCall::new(
                                None,
                                BuiltInFunction::Size,
                                vec![Variable::new("x", Position::fake()).into()],
                                Position::fake()
                            ),
                            Position::fake(),
                        ),
                        false,
                    )],)
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![Argument::new("x", list_type.clone())],
                            types::Number::new(Position::fake()),
                            BuiltInCall::new(
                                Some(
                                    types::Function::new(
                                        vec![list_type.into()],
                                        types::Number::new(Position::fake()),
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::Size,
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
                infer_module(
                    &Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            function_type.clone(),
                            BuiltInCall::new(
                                None,
                                BuiltInFunction::Spawn,
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
                    )],)
                ),
                Ok(
                    Module::empty().set_definitions(vec![FunctionDefinition::fake(
                        "x",
                        Lambda::new(
                            vec![],
                            function_type.clone(),
                            BuiltInCall::new(
                                Some(
                                    types::Function::new(
                                        vec![function_type.clone().into()],
                                        function_type,
                                        Position::fake()
                                    )
                                    .into()
                                ),
                                BuiltInFunction::Spawn,
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
