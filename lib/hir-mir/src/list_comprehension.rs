use crate::{context::Context, downcast, expression, type_, CompileError};
use hir::{
    analysis::{type_canonicalizer, AnalysisError},
    ir::*,
    types::{self, Type},
};

pub fn compile(
    context: &Context,
    comprehension: &ListComprehension,
) -> Result<mir::ir::Expression, CompileError> {
    let [branch, ..] = comprehension.branches() else { unreachable!() };
    let iteratee_types = branch
        .iteratees()
        .iter()
        .map(|iteratee| {
            type_canonicalizer::canonicalize(
                iteratee.type_().ok_or_else(|| {
                    AnalysisError::TypeNotInferred(comprehension.position().clone())
                })?,
                context.types(),
            )
        })
        .collect::<Result<Vec<_>, _>>()?;
    // TODO Define a compile_element function.
    let element = if comprehension.branches().len() == 1 {
        ListElement::Single(comprehension.element().clone())
    } else {
        ListElement::Multiple(
            ListComprehension::new(
                comprehension.type_().clone(),
                comprehension.element().clone(),
                comprehension.branches()[1..].to_vec(),
                comprehension.position().clone(),
            )
            .into(),
        )
    };

    if let [Type::Map(map_type)] = iteratee_types.as_slice() {
        compile_map(context, comprehension, branch, map_type, element)
    } else if iteratee_types
        .iter()
        .all(|type_| matches!(type_, Type::List(_)))
    {
        let list_types = iteratee_types
            .iter()
            .filter_map(|type_| type_.as_list())
            .collect::<Vec<_>>();

        compile_lists(context, comprehension, branch, &list_types, element)
    } else if iteratee_types
        .iter()
        .all(|type_| matches!(type_, Type::Map(_)))
    {
        Err(CompileError::MultipleMapsInListComprehension(
            comprehension.position().clone(),
        ))
    } else if iteratee_types
        .iter()
        .all(|type_| matches!(type_, Type::List(_) | Type::Map(_)))
    {
        Err(CompileError::MixedIterateesInListComprehension(
            comprehension.position().clone(),
        ))
    } else {
        let index = iteratee_types
            .iter()
            .position(|type_| !matches!(type_, Type::List(_) | Type::Map(_)))
            .unwrap();

        Err(AnalysisError::CollectionExpected(
            iteratee_types[index]
                .clone()
                .set_position(branch.iteratees()[index].position().clone()),
        )
        .into())
    }
}

fn compile_lists(
    context: &Context,
    comprehension: &ListComprehension,
    branch: &ListComprehensionBranch,
    iteratee_types: &[&types::List],
    element: ListElement,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$loop";

    let list_type = type_::compile_list(context)?;
    let definition = compile_list_iteration_function_definition(
        context,
        comprehension,
        branch,
        iteratee_types,
        element,
    )?;

    Ok(mir::ir::Call::new(
        mir::types::Function::new(
            vec![mir::types::Function::new(vec![], list_type.clone()).into()],
            list_type.clone(),
        ),
        mir::ir::Variable::new(&context.configuration()?.list_type.lazy_function_name),
        vec![mir::ir::LetRecursive::new(
            mir::ir::FunctionDefinition::new(
                CLOSURE_NAME,
                vec![],
                list_type.clone(),
                mir::ir::LetRecursive::new(
                    definition.clone(),
                    mir::ir::Call::new(
                        mir::types::Function::new(
                            branch
                                .iteratees()
                                .iter()
                                .map(|_| list_type.clone().into())
                                .collect(),
                            list_type,
                        ),
                        mir::ir::Variable::new(definition.name()),
                        branch
                            .iteratees()
                            .iter()
                            .map(|iteratee| expression::compile(context, iteratee.expression()))
                            .collect::<Result<_, _>>()?,
                    ),
                ),
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into()],
    )
    .into())
}

fn compile_list_iteration_function_definition(
    context: &Context,
    comprehension: &ListComprehension,
    branch: &ListComprehensionBranch,
    iteratee_types: &[&types::List],
    element: ListElement,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    const CLOSURE_NAME: &str = "$loop";
    const LIST_NAME: &str = "$list";

    let position = comprehension.position();
    let list_type = type_::compile_list(context)?;
    let iteratee_names = (0..branch.iteratees().len())
        .map(|index| format!("{}_{}", LIST_NAME, index))
        .collect::<Vec<_>>();
    let arguments = iteratee_names
        .iter()
        .map(|name| mir::ir::Argument::new(name, list_type.clone()))
        .collect();

    let mut body = {
        let rest = Call::new(
            Some(
                types::Function::new(
                    iteratee_types
                        .iter()
                        .map(|&type_| type_.clone().into())
                        .collect(),
                    types::List::new(comprehension.type_().clone(), position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(CLOSURE_NAME, position.clone()),
            iteratee_names
                .iter()
                .map(|name| Variable::new(name, position.clone()).into())
                .collect(),
            position.clone(),
        );
        let list = List::new(
            comprehension.type_().clone(),
            vec![element, ListElement::Multiple(rest.clone().into())],
            position.clone(),
        );

        if let Some(condition) = branch.condition() {
            Expression::from(If::new(
                condition.clone(),
                list,
                rest,
                branch.position().clone(),
            ))
        } else {
            list.into()
        }
    };

    for ((element_name, iteratee_name), type_) in branch
        .names()
        .iter()
        .zip(iteratee_names)
        .zip(iteratee_types)
        .rev()
    {
        body = IfList::new(
            Some(type_.element().clone()),
            Variable::new(&iteratee_name, position.clone()),
            element_name,
            iteratee_name,
            body,
            List::new(comprehension.type_().clone(), vec![], position.clone()),
            position.clone(),
        )
        .into()
    }

    Ok(mir::ir::FunctionDefinition::new(
        CLOSURE_NAME,
        arguments,
        list_type,
        expression::compile(context, &body)?,
    ))
}

fn compile_map(
    context: &Context,
    comprehension: &ListComprehension,
    branch: &ListComprehensionBranch,
    map_type: &types::Map,
    element: ListElement,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$loop";

    let list_type = type_::compile_list(context)?;
    let definition = compile_map_iteration_function_definition(
        context,
        comprehension,
        branch,
        map_type,
        element,
    )?;

    Ok(mir::ir::Call::new(
        mir::types::Function::new(
            vec![mir::types::Function::new(vec![], list_type.clone()).into()],
            list_type.clone(),
        ),
        mir::ir::Variable::new(&context.configuration()?.list_type.lazy_function_name),
        vec![mir::ir::LetRecursive::new(
            mir::ir::FunctionDefinition::new(
                CLOSURE_NAME,
                vec![],
                list_type.clone(),
                mir::ir::LetRecursive::new(
                    definition.clone(),
                    mir::ir::Call::new(
                        mir::types::Function::new(vec![mir::types::Type::Variant], list_type),
                        mir::ir::Variable::new(definition.name()),
                        vec![mir::ir::Call::new(
                            mir::types::Function::new(
                                vec![type_::compile_map(context)?.into()],
                                mir::types::Type::Variant,
                            ),
                            mir::ir::Variable::new(
                                &context
                                    .configuration()?
                                    .map_type
                                    .iteration
                                    .iterate_function_name,
                            ),
                            branch
                                .iteratees()
                                .iter()
                                .map(|iteratee| expression::compile(context, iteratee.expression()))
                                .collect::<Result<_, _>>()?,
                        )
                        .into()],
                    ),
                ),
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into()],
    )
    .into())
}

fn compile_map_iteration_function_definition(
    context: &Context,
    comprehension: &ListComprehension,
    branch: &ListComprehensionBranch,
    map_type: &types::Map,
    element: ListElement,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    const CLOSURE_NAME: &str = "$loop";
    const ITERATOR_NAME: &str = "$iterator";

    let iteration_configuration = &context.configuration()?.map_type.iteration;
    let position = comprehension.position();
    let any_type = Type::from(types::Any::new(position.clone()));
    let element_type = comprehension.type_();
    let iterator_type = Type::from(types::Reference::new(
        &iteration_configuration.iterator_type_name,
        position.clone(),
    ));
    let iterator_or_none_type = types::Union::new(
        iterator_type.clone(),
        types::None::new(position.clone()),
        position.clone(),
    );
    let iterator_variable = Variable::new(ITERATOR_NAME, position.clone());
    let compile_key_value_function_call = |name, type_| {
        downcast::compile(
            context,
            &any_type,
            type_,
            &Call::new(
                Some(
                    types::Function::new(
                        vec![iterator_type.clone()],
                        any_type.clone(),
                        position.clone(),
                    )
                    .into(),
                ),
                Variable::new(name, position.clone()),
                vec![iterator_variable.clone().into()],
                position.clone(),
            )
            .into(),
        )
    };

    Ok(mir::ir::FunctionDefinition::new(
        CLOSURE_NAME,
        vec![mir::ir::Argument::new(
            ITERATOR_NAME,
            mir::types::Type::Variant,
        )],
        type_::compile_list(context)?,
        expression::compile(
            context,
            &IfType::new(
                ITERATOR_NAME,
                iterator_variable.clone(),
                vec![IfTypeBranch::new(
                    iterator_type.clone(),
                    Let::new(
                        Some(
                            branch
                                .names()
                                .get(0)
                                .ok_or_else(|| {
                                    AnalysisError::KeyNameNotDefined(
                                        comprehension.position().clone(),
                                    )
                                })?
                                .into(),
                        ),
                        Some(map_type.key().clone()),
                        compile_key_value_function_call(
                            &iteration_configuration.key_function_name,
                            map_type.key(),
                        )?,
                        Let::new(
                            Some(
                                branch
                                    .names()
                                    .get(1)
                                    .ok_or_else(|| {
                                        AnalysisError::ValueNameNotDefined(
                                            comprehension.position().clone(),
                                        )
                                    })?
                                    .into(),
                            ),
                            Some(map_type.value().clone()),
                            compile_key_value_function_call(
                                &iteration_configuration.value_function_name,
                                map_type.value(),
                            )?,
                            {
                                let rest = Call::new(
                                    Some(
                                        types::Function::new(
                                            vec![iterator_or_none_type.clone().into()],
                                            types::List::new(
                                                element_type.clone(),
                                                position.clone(),
                                            ),
                                            position.clone(),
                                        )
                                        .into(),
                                    ),
                                    Variable::new(CLOSURE_NAME, position.clone()),
                                    vec![Call::new(
                                        Some(
                                            types::Function::new(
                                                vec![iterator_type.clone()],
                                                iterator_or_none_type,
                                                position.clone(),
                                            )
                                            .into(),
                                        ),
                                        Variable::new(
                                            &iteration_configuration.rest_function_name,
                                            position.clone(),
                                        ),
                                        vec![iterator_variable.into()],
                                        position.clone(),
                                    )
                                    .into()],
                                    position.clone(),
                                );
                                let list = List::new(
                                    element_type.clone(),
                                    vec![element, ListElement::Multiple(rest.clone().into())],
                                    position.clone(),
                                );

                                if let Some(condition) = branch.condition() {
                                    Expression::from(If::new(
                                        condition.clone(),
                                        list,
                                        rest,
                                        branch.position().clone(),
                                    ))
                                } else {
                                    list.into()
                                }
                            },
                            position.clone(),
                        ),
                        position.clone(),
                    ),
                )],
                Some(ElseBranch::new(
                    Some(types::None::new(position.clone()).into()),
                    List::new(element_type.clone(), vec![], position.clone()),
                    position.clone(),
                )),
                position.clone(),
            )
            .into(),
        )?,
    ))
}
