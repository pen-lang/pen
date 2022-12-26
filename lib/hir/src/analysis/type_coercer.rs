use super::{context::AnalysisContext, AnalysisError};
use crate::{
    analysis::{
        module_environment, record_field_resolver, type_canonicalizer, type_equality_checker,
        type_extractor,
    },
    ir::*,
    types::{self, Type},
};

pub fn coerce_types(context: &AnalysisContext, module: &Module) -> Result<Module, AnalysisError> {
    let variables = plist::FlailMap::new(module_environment::create(module));

    Ok(Module::new(
        module.type_definitions().to_vec(),
        module.type_aliases().to_vec(),
        module.foreign_declarations().to_vec(),
        module.function_declarations().to_vec(),
        module
            .function_definitions()
            .iter()
            .map(|definition| transform_function_definition(context, definition, &variables))
            .collect::<Result<_, _>>()?,
        module.position().clone(),
    ))
}

fn transform_function_definition(
    context: &AnalysisContext,
    definition: &FunctionDefinition,
    variables: &plist::FlailMap<String, Type>,
) -> Result<FunctionDefinition, AnalysisError> {
    Ok(FunctionDefinition::new(
        definition.name(),
        definition.original_name(),
        transform_lambda(definition.lambda(), variables, context)?,
        definition.foreign_definition_configuration().cloned(),
        definition.is_public(),
        definition.position().clone(),
    ))
}

fn transform_lambda(
    lambda: &Lambda,
    variables: &plist::FlailMap<String, Type>,
    context: &AnalysisContext,
) -> Result<Lambda, AnalysisError> {
    let variables = variables.insert_iter(
        lambda
            .arguments()
            .iter()
            .map(|argument| (argument.name().into(), argument.type_().clone())),
    );

    Ok(Lambda::new(
        lambda.arguments().to_vec(),
        lambda.result_type().clone(),
        coerce_expression(
            context,
            &transform_expression(context, lambda.body(), &variables)?,
            lambda.result_type(),
            &variables,
        )?,
        lambda.position().clone(),
    ))
}

fn transform_expression(
    context: &AnalysisContext,
    expression: &Expression,
    variables: &plist::FlailMap<String, Type>,
) -> Result<Expression, AnalysisError> {
    let transform_expression =
        |expression, variables: &_| transform_expression(context, expression, variables);
    let transform_and_coerce_expression = |expression, type_: &_, variables: &_| {
        coerce_expression(
            context,
            &transform_expression(expression, variables)?,
            type_,
            variables,
        )
    };
    let extract_type = |expression, variables: &plist::FlailMap<String, Type>| {
        type_extractor::extract_from_expression(context, expression, variables)
    };

    Ok(match expression {
        Expression::Call(call) => {
            let type_ = call
                .function_type()
                .ok_or_else(|| AnalysisError::TypeNotInferred(call.position().clone()))?;
            let function_type = type_canonicalizer::canonicalize_function(type_, context.types())?
                .ok_or_else(|| AnalysisError::FunctionExpected(type_.clone()))?;

            Call::new(
                call.function_type().cloned(),
                transform_expression(call.function(), variables)?,
                call.arguments()
                    .iter()
                    .zip(function_type.arguments())
                    .map(|(argument, type_)| {
                        transform_and_coerce_expression(argument, type_, variables)
                    })
                    .collect::<Result<_, _>>()?,
                call.position().clone(),
            )
            .into()
        }
        Expression::If(if_) => {
            let type_ = extract_type(expression, variables)?;

            If::new(
                transform_expression(if_.condition(), variables)?,
                transform_and_coerce_expression(if_.then(), &type_, variables)?,
                transform_and_coerce_expression(if_.else_(), &type_, variables)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfList(if_) => {
            let list_type = types::List::new(
                if_.type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(if_.list().position().clone()))?
                    .clone(),
                if_.list().position().clone(),
            );
            let result_type = extract_type(expression, variables)?;

            IfList::new(
                if_.type_().cloned(),
                transform_expression(if_.list(), variables)?,
                if_.first_name(),
                if_.rest_name(),
                transform_and_coerce_expression(
                    if_.then(),
                    &result_type,
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
                )?,
                transform_and_coerce_expression(if_.else_(), &result_type, variables)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfMap(if_) => {
            let map_type = types::Map::new(
                if_.key_type()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(if_.map().position().clone()))?
                    .clone(),
                if_.value_type()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(if_.map().position().clone()))?
                    .clone(),
                if_.map().position().clone(),
            );
            let result_type = extract_type(expression, variables)?;

            IfMap::new(
                if_.key_type().cloned(),
                if_.value_type().cloned(),
                if_.name(),
                transform_expression(if_.map(), variables)?,
                transform_and_coerce_expression(if_.key(), map_type.key(), variables)?,
                transform_and_coerce_expression(
                    if_.then(),
                    &result_type,
                    &variables.insert(if_.name().into(), map_type.value().clone()),
                )?,
                transform_and_coerce_expression(if_.else_(), &result_type, variables)?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::IfType(if_) => {
            let result_type = extract_type(expression, variables)?;

            IfType::new(
                if_.name(),
                transform_expression(if_.argument(), variables)?,
                if_.branches()
                    .iter()
                    .map(|branch| {
                        Ok(IfTypeBranch::new(
                            branch.type_().clone(),
                            transform_and_coerce_expression(
                                branch.expression(),
                                &result_type,
                                &variables.insert(if_.name().into(), branch.type_().clone()),
                            )?,
                        ))
                    })
                    .collect::<Result<_, _>>()?,
                if_.else_()
                    .map(|branch| {
                        Ok(ElseBranch::new(
                            branch.type_().cloned(),
                            transform_and_coerce_expression(
                                branch.expression(),
                                &result_type,
                                &variables.insert(
                                    if_.name().into(),
                                    branch
                                        .type_()
                                        .ok_or_else(|| {
                                            AnalysisError::TypeNotInferred(
                                                branch.position().clone(),
                                            )
                                        })?
                                        .clone(),
                                ),
                            )?,
                            branch.position().clone(),
                        ))
                    })
                    .transpose()?,
                if_.position().clone(),
            )
            .into()
        }
        Expression::Lambda(lambda) => transform_lambda(lambda, variables, context)?.into(),
        Expression::Let(let_) => Let::new(
            let_.name().map(String::from),
            let_.type_().cloned(),
            transform_expression(let_.bound_expression(), variables)?,
            transform_expression(
                let_.expression(),
                &variables.insert_iter(
                    let_.name()
                        .map(|name| {
                            Ok((
                                name.into(),
                                let_.type_()
                                    .ok_or_else(|| {
                                        AnalysisError::TypeNotInferred(
                                            let_.bound_expression().position().clone(),
                                        )
                                    })?
                                    .clone(),
                            ))
                        })
                        .transpose()?,
                ),
            )?,
            let_.position().clone(),
        )
        .into(),
        Expression::List(list) => List::new(
            list.type_().clone(),
            list.elements()
                .iter()
                .map(|element| {
                    Ok(match element {
                        ListElement::Multiple(element) => {
                            ListElement::Multiple(transform_and_coerce_expression(
                                element,
                                &types::List::new(list.type_().clone(), element.position().clone())
                                    .into(),
                                variables,
                            )?)
                        }
                        ListElement::Single(element) => ListElement::Single(
                            transform_and_coerce_expression(element, list.type_(), variables)?,
                        ),
                    })
                })
                .collect::<Result<_, _>>()?,
            list.position().clone(),
        )
        .into(),
        Expression::ListComprehension(comprehension) => {
            let position = comprehension.position();
            let mut branches = vec![];
            let mut variables = variables.clone();

            for branch in comprehension.branches() {
                let iteratees = branch
                    .iteratees()
                    .iter()
                    .map(|iteratee| {
                        Ok(ListComprehensionIteratee::new(
                            iteratee.type_().cloned(),
                            transform_expression(iteratee.expression(), &variables)?,
                        ))
                    })
                    .collect::<Result<_, _>>()?;

                variables = variables.insert_iter(
                    branch
                        .names()
                        .iter()
                        .zip(branch.iteratees())
                        .map(|(name, iteratee)| {
                            let type_ = iteratee
                                .type_()
                                .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;

                            Ok((
                                name.into(),
                                types::Function::new(
                                    vec![],
                                    type_canonicalizer::canonicalize_list(type_, context.types())?
                                        .ok_or_else(|| AnalysisError::ListExpected(type_.clone()))?
                                        .element()
                                        .clone(),
                                    position.clone(),
                                )
                                .into(),
                            ))
                        })
                        .collect::<Result<Vec<_>, _>>()?,
                );

                branches.push(ListComprehensionBranch::new(
                    branch.names().to_vec(),
                    iteratees,
                    branch
                        .condition()
                        .map(|expression| transform_expression(expression, &variables))
                        .transpose()?,
                    position.clone(),
                ));
            }

            ListComprehension::new(
                comprehension.type_().clone(),
                transform_and_coerce_expression(
                    comprehension.element(),
                    comprehension.type_(),
                    &variables,
                )?,
                branches,
                position.clone(),
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
                            transform_and_coerce_expression(
                                entry.key(),
                                map.key_type(),
                                variables,
                            )?,
                            transform_and_coerce_expression(
                                entry.value(),
                                map.value_type(),
                                variables,
                            )?,
                            entry.position().clone(),
                        )),
                        MapElement::Map(other) => MapElement::Map(transform_and_coerce_expression(
                            other,
                            &types::Map::new(
                                map.key_type().clone(),
                                map.value_type().clone(),
                                map.position().clone(),
                            )
                            .into(),
                            variables,
                        )?),
                    })
                })
                .collect::<Result<_, _>>()?,
            map.position().clone(),
        )
        .into(),
        Expression::Operation(operation) => match operation {
            Operation::Addition(operation) => AdditionOperation::new(
                operation.type_().cloned(),
                transform_expression(operation.lhs(), variables)?,
                transform_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Arithmetic(operation) => ArithmeticOperation::new(
                operation.operator(),
                transform_expression(operation.lhs(), variables)?,
                transform_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Boolean(operation) => BooleanOperation::new(
                operation.operator(),
                transform_expression(operation.lhs(), variables)?,
                transform_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Equality(operation) => {
                let type_ = operation
                    .type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?;

                EqualityOperation::new(
                    operation.type_().cloned(),
                    operation.operator(),
                    transform_and_coerce_expression(operation.lhs(), type_, variables)?,
                    transform_and_coerce_expression(operation.rhs(), type_, variables)?,
                    operation.position().clone(),
                )
                .into()
            }
            Operation::Not(operation) => NotOperation::new(
                transform_expression(operation.expression(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Order(operation) => OrderOperation::new(
                operation.operator(),
                transform_expression(operation.lhs(), variables)?,
                transform_expression(operation.rhs(), variables)?,
                operation.position().clone(),
            )
            .into(),
            Operation::Try(operation) => TryOperation::new(
                operation.type_().cloned(),
                transform_expression(operation.expression(), variables)?,
                operation.position().clone(),
            )
            .into(),
        },
        Expression::RecordConstruction(construction) => RecordConstruction::new(
            construction.type_().clone(),
            transform_record_fields(
                construction.fields(),
                construction.type_(),
                variables,
                context,
            )?,
            construction.position().clone(),
        )
        .into(),
        Expression::RecordDeconstruction(deconstruction) => RecordDeconstruction::new(
            deconstruction.type_().cloned(),
            transform_expression(deconstruction.record(), variables)?,
            deconstruction.field_name(),
            deconstruction.position().clone(),
        )
        .into(),
        Expression::RecordUpdate(update) => RecordUpdate::new(
            update.type_().clone(),
            transform_expression(update.record(), variables)?,
            transform_record_fields(update.fields(), update.type_(), variables, context)?,
            update.position().clone(),
        )
        .into(),
        Expression::Thunk(thunk) => Thunk::new(
            thunk.type_().cloned(),
            transform_and_coerce_expression(
                thunk.expression(),
                thunk
                    .type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(thunk.position().clone()))?,
                variables,
            )?,
            thunk.position().clone(),
        )
        .into(),
        Expression::TypeCoercion(coercion) => TypeCoercion::new(
            coercion.from().clone(),
            coercion.to().clone(),
            transform_expression(coercion.argument(), variables)?,
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

fn transform_record_fields(
    fields: &[RecordField],
    record_type: &Type,
    variables: &plist::FlailMap<String, Type>,
    context: &AnalysisContext,
) -> Result<Vec<RecordField>, AnalysisError> {
    let field_types =
        record_field_resolver::resolve(record_type, context.types(), context.records())?;

    fields
        .iter()
        .map(|field| {
            Ok(RecordField::new(
                field.name(),
                coerce_expression(
                    context,
                    &transform_expression(context, field.expression(), variables)?,
                    field_types
                        .iter()
                        .find(|field_type| field_type.name() == field.name())
                        .ok_or_else(|| AnalysisError::UnknownRecordField(field.position().clone()))?
                        .type_(),
                    variables,
                )?,
                field.position().clone(),
            ))
        })
        .collect::<Result<_, _>>()
}

fn coerce_expression(
    context: &AnalysisContext,
    expression: &Expression,
    upper_type: &Type,
    variables: &plist::FlailMap<String, Type>,
) -> Result<Expression, AnalysisError> {
    let lower_type = type_extractor::extract_from_expression(context, expression, variables)?;

    Ok(
        if type_equality_checker::check(&lower_type, upper_type, context.types())? {
            expression.clone()
        } else {
            TypeCoercion::new(
                lower_type,
                upper_type.clone(),
                expression.clone(),
                expression.position().clone(),
            )
            .into()
        },
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        analysis::type_collector,
        test::{FunctionDefinitionFake, ModuleFake, RecordFake, TypeDefinitionFake},
    };
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    fn coerce_module(module: &Module) -> Result<Module, AnalysisError> {
        coerce_types(
            &AnalysisContext::new(
                type_collector::collect(module),
                type_collector::collect_record_fields(module),
            ),
            module,
        )
    }

    #[test]
    fn coerce_function_result() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        union_type.clone(),
                        None::new(Position::fake()),
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
                        union_type.clone(),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
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
    fn coerce_function_result_of_variable() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::fake()))],
                        union_type.clone(),
                        Variable::new("x", Position::fake()),
                        Position::fake(),
                    ),
                    false,
                )
            ],)),
            Ok(
                Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", types::None::new(Position::fake()))],
                        union_type.clone(),
                        TypeCoercion::new(
                            types::None::new(Position::fake()),
                            union_type,
                            Variable::new("x", Position::fake()),
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
    fn coerce_built_in_call() {
        let list_type = types::List::new(types::None::new(Position::fake()), Position::fake());
        let module = Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
            "f",
            Lambda::new(
                vec![Argument::new("x", list_type.clone())],
                types::Number::new(Position::fake()),
                Call::new(
                    Some(
                        types::Function::new(
                            vec![list_type.into()],
                            types::Number::new(Position::fake()),
                            Position::fake(),
                        )
                        .into(),
                    ),
                    BuiltInFunction::new(BuiltInFunctionName::Size, Position::fake()),
                    vec![Variable::new("x", Position::fake()).into()],
                    Position::fake(),
                ),
                Position::fake(),
            ),
            false,
        )]);

        assert_eq!(coerce_module(&module), Ok(module));
    }

    #[test]
    fn coerce_if() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        union_type.clone(),
                        If::new(
                            Boolean::new(true, Position::fake()),
                            Number::new(42.0, Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
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
                        union_type.clone(),
                        If::new(
                            Boolean::new(true, Position::fake()),
                            TypeCoercion::new(
                                types::Number::new(Position::fake()),
                                union_type.clone(),
                                Number::new(42.0, Position::fake()),
                                Position::fake(),
                            ),
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
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
    fn coerce_if_list() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let list_type = types::List::new(types::Number::new(Position::fake()), Position::fake());
        let element_call = Call::new(
            Some(
                types::Function::new(
                    vec![],
                    types::Number::new(Position::fake()),
                    Position::fake(),
                )
                .into(),
            ),
            Variable::new("x", Position::fake()),
            vec![],
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("xs", list_type.clone())],
                        union_type.clone(),
                        IfList::new(
                            Some(types::Number::new(Position::fake()).into()),
                            Variable::new("xs", Position::fake()),
                            "x",
                            "xs",
                            element_call.clone(),
                            None::new(Position::fake()),
                            Position::fake(),
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
                        vec![Argument::new("xs", list_type)],
                        union_type.clone(),
                        IfList::new(
                            Some(types::Number::new(Position::fake()).into()),
                            Variable::new("xs", Position::fake()),
                            "x",
                            "xs",
                            TypeCoercion::new(
                                types::Number::new(Position::fake()),
                                union_type.clone(),
                                element_call,
                                Position::fake(),
                            ),
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
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
    fn coerce_if_list_with_function_union_type() {
        let number_thunk_type = types::Function::new(
            vec![],
            types::Number::new(Position::fake()),
            Position::fake(),
        );
        let union_type = types::Union::new(
            number_thunk_type.clone(),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let list_type = types::List::new(types::Number::new(Position::fake()), Position::fake());

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("xs", list_type.clone())],
                        union_type.clone(),
                        IfList::new(
                            Some(types::Number::new(Position::fake()).into()),
                            Variable::new("xs", Position::fake()),
                            "x",
                            "xs",
                            Variable::new("x", Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
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
                        vec![Argument::new("xs", list_type)],
                        union_type.clone(),
                        IfList::new(
                            Some(types::Number::new(Position::fake()).into()),
                            Variable::new("xs", Position::fake()),
                            "x",
                            "xs",
                            TypeCoercion::new(
                                number_thunk_type,
                                union_type.clone(),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
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
    fn coerce_if_map() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let map_type = types::Map::new(
            types::Boolean::new(Position::fake()),
            types::Number::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("xs", map_type.clone())],
                        union_type.clone(),
                        IfMap::new(
                            Some(map_type.key().clone()),
                            Some(map_type.value().clone()),
                            "x",
                            Variable::new("xs", Position::fake()),
                            Boolean::new(true, Position::fake()),
                            Variable::new("x", Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
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
                        vec![Argument::new("xs", map_type.clone())],
                        union_type.clone(),
                        IfMap::new(
                            Some(map_type.key().clone()),
                            Some(map_type.value().clone()),
                            "x",
                            Variable::new("xs", Position::fake()),
                            Boolean::new(true, Position::fake()),
                            TypeCoercion::new(
                                types::Number::new(Position::fake()),
                                union_type.clone(),
                                Variable::new("x", Position::fake()),
                                Position::fake(),
                            ),
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])
            )
        );
    }

    #[test]
    fn coerce_key_in_if_map() {
        let union_type = types::Union::new(
            types::Boolean::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let map_type = types::Map::new(
            union_type.clone(),
            types::Number::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("xs", map_type.clone())],
                        types::None::new(Position::fake()),
                        IfMap::new(
                            Some(map_type.key().clone()),
                            Some(map_type.value().clone()),
                            "x",
                            Variable::new("xs", Position::fake()),
                            Boolean::new(true, Position::fake()),
                            None::new(Position::fake()),
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
                        vec![Argument::new("xs", map_type.clone())],
                        types::None::new(Position::fake()),
                        IfMap::new(
                            Some(map_type.key().clone()),
                            Some(map_type.value().clone()),
                            "x",
                            Variable::new("xs", Position::fake()),
                            TypeCoercion::new(
                                types::Boolean::new(Position::fake()),
                                union_type,
                                Boolean::new(true, Position::fake()),
                                Position::fake(),
                            ),
                            None::new(Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )])
            )
        );
    }

    #[test]
    fn coerce_if_type() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("x", union_type.clone())],
                        union_type.clone(),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::Number::new(Position::fake()),
                                Variable::new("y", Position::fake()),
                            )],
                            Some(ElseBranch::new(
                                Some(types::None::new(Position::fake()).into()),
                                None::new(Position::fake()),
                                Position::fake(),
                            )),
                            Position::fake(),
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
                        vec![Argument::new("x", union_type.clone())],
                        union_type.clone(),
                        IfType::new(
                            "y",
                            Variable::new("x", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::Number::new(Position::fake()),
                                TypeCoercion::new(
                                    types::Number::new(Position::fake()),
                                    union_type.clone(),
                                    Variable::new("y", Position::fake()),
                                    Position::fake(),
                                ),
                            )],
                            Some(ElseBranch::new(
                                Some(types::None::new(Position::fake()).into()),
                                TypeCoercion::new(
                                    types::None::new(Position::fake()),
                                    union_type,
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake()
                            )),
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
    fn coerce_equality_operation() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::fake()),
                        EqualityOperation::new(
                            Some(union_type.clone().into()),
                            EqualityOperator::Equal,
                            Number::new(42.0, Position::fake()),
                            None::new(Position::fake()),
                            Position::fake(),
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
                        types::Boolean::new(Position::fake()),
                        EqualityOperation::new(
                            Some(union_type.clone().into()),
                            EqualityOperator::Equal,
                            TypeCoercion::new(
                                types::Number::new(Position::fake()),
                                union_type.clone(),
                                Number::new(42.0, Position::fake()),
                                Position::fake(),
                            ),
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake(),
                            ),
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
    fn coerce_record_construction() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let type_definition = TypeDefinition::fake(
            "r",
            vec![types::RecordField::new("x", union_type.clone())],
            false,
            false,
            false,
        );
        let record_type = types::Record::fake("r");

        assert_eq!(
            coerce_module(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            record_type.clone(),
                            RecordConstruction::new(
                                record_type.clone(),
                                vec![RecordField::new(
                                    "x",
                                    None::new(Position::fake()),
                                    Position::fake()
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        record_type.clone(),
                        RecordConstruction::new(
                            record_type,
                            vec![RecordField::new(
                                "x",
                                TypeCoercion::new(
                                    types::None::new(Position::fake()),
                                    union_type,
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    #[test]
    fn coerce_record_update() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );
        let type_definition = TypeDefinition::fake(
            "r",
            vec![types::RecordField::new("x", union_type.clone())],
            false,
            false,
            false,
        );
        let record_type = types::Record::fake("r");

        assert_eq!(
            coerce_module(
                &Module::empty()
                    .set_type_definitions(vec![type_definition.clone()])
                    .set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("r", record_type.clone())],
                            record_type.clone(),
                            RecordUpdate::new(
                                record_type.clone(),
                                Variable::new("r", Position::fake()),
                                vec![RecordField::new(
                                    "x",
                                    None::new(Position::fake()),
                                    Position::fake()
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
            ),
            Ok(Module::empty()
                .set_type_definitions(vec![type_definition])
                .set_function_definitions(vec![FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![Argument::new("r", record_type.clone())],
                        record_type.clone(),
                        RecordUpdate::new(
                            record_type,
                            Variable::new("r", Position::fake()),
                            vec![RecordField::new(
                                "x",
                                TypeCoercion::new(
                                    types::None::new(Position::fake()),
                                    union_type,
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ),
                                Position::fake(),
                            )],
                            Position::fake(),
                        ),
                        Position::fake(),
                    ),
                    false,
                )]))
        );
    }

    #[test]
    fn coerce_thunk() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            coerce_module(&Module::empty().set_function_definitions(vec![
                FunctionDefinition::fake(
                    "f",
                    Lambda::new(
                        vec![],
                        types::Function::new(vec![], union_type.clone(), Position::fake()),
                        Thunk::new(
                            Some(union_type.clone().into()),
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
                    "f",
                    Lambda::new(
                        vec![],
                        types::Function::new(vec![], union_type.clone(), Position::fake()),
                        Thunk::new(
                            Some(union_type.clone().into()),
                            TypeCoercion::new(
                                types::None::new(Position::fake()),
                                union_type,
                                None::new(Position::fake()),
                                Position::fake()
                            ),
                            Position::fake()
                        ),
                        Position::fake(),
                    ),
                    false,
                )],)
            )
        );
    }

    mod list {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn coerce_single_element_in_list() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let list_type = types::List::new(union_type.clone(), Position::fake());

            assert_eq!(
                coerce_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            List::new(
                                union_type.clone(),
                                vec![ListElement::Single(None::new(Position::fake()).into())],
                                Position::fake(),
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
                            list_type,
                            List::new(
                                union_type.clone(),
                                vec![ListElement::Single(
                                    TypeCoercion::new(
                                        types::None::new(Position::fake()),
                                        union_type,
                                        None::new(Position::fake()),
                                        Position::fake(),
                                    )
                                    .into()
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
        fn coerce_multiple_element_in_list() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let list_type = types::List::new(union_type.clone(), Position::fake());

            assert_eq!(
                coerce_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            list_type.clone(),
                            List::new(
                                union_type.clone(),
                                vec![ListElement::Multiple(
                                    List::new(
                                        types::None::new(Position::fake()),
                                        vec![],
                                        Position::fake()
                                    )
                                    .into()
                                )],
                                Position::fake(),
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
                            list_type.clone(),
                            List::new(
                                union_type,
                                vec![ListElement::Multiple(
                                    TypeCoercion::new(
                                        types::List::new(
                                            types::None::new(Position::fake()),
                                            Position::fake()
                                        ),
                                        list_type,
                                        List::new(
                                            types::None::new(Position::fake()),
                                            vec![],
                                            Position::fake()
                                        ),
                                        Position::fake(),
                                    )
                                    .into()
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

        mod list_comprehension {
            use super::*;
            use pretty_assertions::assert_eq;

            #[test]
            fn coerce_output_element() {
                let union_type = types::Union::new(
                    types::Number::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake(),
                );
                let input_list_type =
                    types::List::new(types::None::new(Position::fake()), Position::fake());
                let output_list_type = types::List::new(union_type.clone(), Position::fake());
                let empty_list =
                    List::new(types::None::new(Position::fake()), vec![], Position::fake());

                assert_eq!(
                    coerce_module(&Module::empty().set_function_definitions(vec![
                        FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![],
                                output_list_type.clone(),
                                ListComprehension::new(
                                    union_type.clone(),
                                    None::new(Position::fake()),
                                    vec![ListComprehensionBranch::new(
                                        vec!["x".into()],
                                        vec![ListComprehensionIteratee::new(
                                            Some(input_list_type.clone().into()),
                                            empty_list.clone(),
                                        )],
                                        None,
                                        Position::fake(),
                                    )],
                                    Position::fake(),
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
                                output_list_type,
                                ListComprehension::new(
                                    union_type.clone(),
                                    TypeCoercion::new(
                                        types::None::new(Position::fake()),
                                        union_type,
                                        None::new(Position::fake()),
                                        Position::fake(),
                                    ),
                                    vec![ListComprehensionBranch::new(
                                        vec!["x".into()],
                                        vec![ListComprehensionIteratee::new(
                                            Some(input_list_type.into()),
                                            empty_list,
                                        )],
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
            fn coerce_input_element() {
                let union_type = types::Union::new(
                    types::Number::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake(),
                );
                let input_list_type =
                    types::List::new(types::None::new(Position::fake()), Position::fake());
                let output_list_type = types::List::new(union_type.clone(), Position::fake());
                let empty_list =
                    List::new(types::None::new(Position::fake()), vec![], Position::fake());

                assert_eq!(
                    coerce_module(&Module::empty().set_function_definitions(
                        vec![FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![],
                                output_list_type.clone(),
                                ListComprehension::new(
                                    union_type.clone(),
                                    Call::new(
                                        Some(
                                            types::Function::new(
                                                vec![],
                                                types::None::new(Position::fake()),
                                                Position::fake(),
                                            )
                                            .into()
                                        ),
                                        Variable::new("x", Position::fake()),
                                        vec![],
                                        Position::fake(),
                                    ),
                                    vec![ListComprehensionBranch::new(
                                        vec!["x".into()],
                                        vec![ListComprehensionIteratee::new(
                                            Some(input_list_type.clone().into()),
                                            empty_list.clone(),
                                        )],
                                        None,
                                        Position::fake(),
                                    )],
                                    Position::fake(),
                                ),
                                Position::fake(),
                            ),
                            false,
                        )],
                    )),
                    Ok(
                        Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![],
                                output_list_type,
                                ListComprehension::new(
                                    union_type.clone(),
                                    TypeCoercion::new(
                                        types::None::new(Position::fake()),
                                        union_type,
                                        Call::new(
                                            Some(
                                                types::Function::new(
                                                    vec![],
                                                    types::None::new(Position::fake()),
                                                    Position::fake(),
                                                )
                                                .into()
                                            ),
                                            Variable::new("x", Position::fake()),
                                            vec![],
                                            Position::fake(),
                                        ),
                                        Position::fake(),
                                    ),
                                    vec![ListComprehensionBranch::new(
                                        vec!["x".into()],
                                        vec![ListComprehensionIteratee::new(
                                            Some(input_list_type.into()),
                                            empty_list,
                                        )],
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
            fn coerce_condition() {
                let union_type = types::Union::new(
                    types::Number::new(Position::fake()),
                    types::None::new(Position::fake()),
                    Position::fake(),
                );
                let list_type =
                    types::List::new(types::None::new(Position::fake()), Position::fake());
                let empty_list =
                    List::new(types::None::new(Position::fake()), vec![], Position::fake());

                assert_eq!(
                    coerce_module(&Module::empty().set_function_definitions(vec![
                        FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![Argument::new("a", union_type.clone())],
                                list_type.clone(),
                                ListComprehension::new(
                                    types::None::new(Position::fake()),
                                    None::new(Position::fake()),
                                    vec![ListComprehensionBranch::new(
                                        vec!["x".into()],
                                        vec![ListComprehensionIteratee::new(
                                            Some(list_type.clone().into()),
                                            empty_list.clone(),
                                        )],
                                        Some(
                                            EqualityOperation::new(
                                                Some(union_type.clone().into()),
                                                EqualityOperator::Equal,
                                                Variable::new("a", Position::fake()),
                                                None::new(Position::fake()),
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
                        )
                    ],)),
                    Ok(
                        Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                            "f",
                            Lambda::new(
                                vec![Argument::new("a", union_type.clone())],
                                list_type.clone(),
                                ListComprehension::new(
                                    types::None::new(Position::fake()),
                                    None::new(Position::fake()),
                                    vec![ListComprehensionBranch::new(
                                        vec!["x".into()],
                                        vec![ListComprehensionIteratee::new(
                                            Some(list_type.into()),
                                            empty_list,
                                        )],
                                        Some(
                                            EqualityOperation::new(
                                                Some(union_type.clone().into()),
                                                EqualityOperator::Equal,
                                                Variable::new("a", Position::fake()),
                                                TypeCoercion::new(
                                                    types::None::new(Position::fake()),
                                                    union_type,
                                                    None::new(Position::fake()),
                                                    Position::fake(),
                                                ),
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
                        )])
                    )
                );
            }
        }
    }

    mod map {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn coerce_key() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let map_type = types::Map::new(
                union_type.clone(),
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                coerce_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            map_type.clone(),
                            Map::new(
                                union_type.clone(),
                                types::None::new(Position::fake()),
                                vec![MapElement::Insertion(MapEntry::new(
                                    None::new(Position::fake()),
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ))],
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
                            map_type,
                            Map::new(
                                union_type.clone(),
                                types::None::new(Position::fake()),
                                vec![MapElement::Insertion(MapEntry::new(
                                    TypeCoercion::new(
                                        types::None::new(Position::fake()),
                                        union_type,
                                        None::new(Position::fake()),
                                        Position::fake(),
                                    ),
                                    None::new(Position::fake()),
                                    Position::fake(),
                                ))],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn coerce_value() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let map_type = types::Map::new(
                types::None::new(Position::fake()),
                union_type.clone(),
                Position::fake(),
            );

            assert_eq!(
                coerce_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            map_type.clone(),
                            Map::new(
                                types::None::new(Position::fake()),
                                union_type.clone(),
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
                    )
                ])),
                Ok(
                    Module::empty().set_function_definitions(vec![FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![],
                            map_type,
                            Map::new(
                                types::None::new(Position::fake()),
                                union_type.clone(),
                                vec![MapEntry::new(
                                    None::new(Position::fake()),
                                    TypeCoercion::new(
                                        types::None::new(Position::fake()),
                                        union_type,
                                        None::new(Position::fake()),
                                        Position::fake(),
                                    ),
                                    Position::fake(),
                                )
                                .into()],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn coerce_map_keys() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let from_map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let to_map_type = types::Map::new(
                union_type,
                types::None::new(Position::fake()),
                Position::fake(),
            );

            assert_eq!(
                coerce_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", from_map_type.clone())],
                            to_map_type.clone(),
                            Map::new(
                                to_map_type.key().clone(),
                                to_map_type.value().clone(),
                                vec![MapElement::Map(Variable::new("x", Position::fake()).into())],
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
                            vec![Argument::new("x", from_map_type.clone())],
                            to_map_type.clone(),
                            Map::new(
                                to_map_type.key().clone(),
                                to_map_type.value().clone(),
                                vec![MapElement::Map(
                                    TypeCoercion::new(
                                        from_map_type,
                                        to_map_type,
                                        Variable::new("x", Position::fake()),
                                        Position::fake(),
                                    )
                                    .into()
                                )],
                                Position::fake(),
                            ),
                            Position::fake(),
                        ),
                        false,
                    )])
                )
            );
        }

        #[test]
        fn coerce_map_values() {
            let union_type = types::Union::new(
                types::Number::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let from_map_type = types::Map::new(
                types::None::new(Position::fake()),
                types::None::new(Position::fake()),
                Position::fake(),
            );
            let to_map_type = types::Map::new(
                types::None::new(Position::fake()),
                union_type,
                Position::fake(),
            );

            assert_eq!(
                coerce_module(&Module::empty().set_function_definitions(vec![
                    FunctionDefinition::fake(
                        "f",
                        Lambda::new(
                            vec![Argument::new("x", from_map_type.clone())],
                            to_map_type.clone(),
                            Map::new(
                                to_map_type.key().clone(),
                                to_map_type.value().clone(),
                                vec![MapElement::Map(Variable::new("x", Position::fake()).into())],
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
                            vec![Argument::new("x", from_map_type.clone())],
                            to_map_type.clone(),
                            Map::new(
                                to_map_type.key().clone(),
                                to_map_type.value().clone(),
                                vec![MapElement::Map(
                                    TypeCoercion::new(
                                        from_map_type,
                                        to_map_type,
                                        Variable::new("x", Position::fake()),
                                        Position::fake(),
                                    )
                                    .into()
                                )],
                                Position::fake(),
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
