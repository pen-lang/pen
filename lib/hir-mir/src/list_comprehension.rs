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
    let iteratee_type = branch
        .type_()
        .ok_or_else(|| AnalysisError::TypeNotInferred(comprehension.position().clone()))?;
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

    match type_canonicalizer::canonicalize(iteratee_type, context.types())? {
        Type::List(list_type) => compile_list(context, comprehension, branch, &list_type, element),
        Type::Map(map_type) => compile_map(context, comprehension, branch, &map_type, element),
        type_ => Err(AnalysisError::CollectionExpected(
            type_.set_position(branch.iteratee().position().clone()),
        )
        .into()),
    }
}

fn compile_list(
    context: &Context,
    comprehension: &ListComprehension,
    branch: &ListComprehensionBranch,
    input_list_type: &types::List,
    element: ListElement,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$loop";
    const LIST_NAME: &str = "$list";

    let position = comprehension.position();
    let input_element_type = input_list_type.element();
    let output_element_type = comprehension.type_();
    let list_type = type_::compile_list(context)?;

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
                    mir::ir::FunctionDefinition::new(
                        CLOSURE_NAME,
                        vec![mir::ir::Argument::new(LIST_NAME, list_type.clone())],
                        list_type.clone(),
                        expression::compile(
                            context,
                            &IfList::new(
                                Some(input_element_type.clone()),
                                Variable::new(LIST_NAME, position.clone()),
                                branch.primary_name(),
                                LIST_NAME,
                                {
                                    let rest = Call::new(
                                        Some(
                                            types::Function::new(
                                                vec![types::List::new(
                                                    input_element_type.clone(),
                                                    position.clone(),
                                                )
                                                .into()],
                                                types::List::new(
                                                    output_element_type.clone(),
                                                    position.clone(),
                                                ),
                                                position.clone(),
                                            )
                                            .into(),
                                        ),
                                        Variable::new(CLOSURE_NAME, position.clone()),
                                        vec![Variable::new(LIST_NAME, position.clone()).into()],
                                        position.clone(),
                                    );
                                    let list = List::new(
                                        output_element_type.clone(),
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
                                List::new(output_element_type.clone(), vec![], position.clone()),
                                position.clone(),
                            )
                            .into(),
                        )?,
                    ),
                    mir::ir::Call::new(
                        mir::types::Function::new(vec![list_type.clone().into()], list_type),
                        mir::ir::Variable::new(CLOSURE_NAME),
                        vec![expression::compile(context, branch.iteratee())?],
                    ),
                ),
            ),
            mir::ir::Variable::new(CLOSURE_NAME),
        )
        .into()],
    )
    .into())
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
                            vec![expression::compile(context, branch.iteratee())?],
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
                        Some(branch.primary_name().into()),
                        Some(map_type.key().clone()),
                        compile_key_value_function_call(
                            &iteration_configuration.key_function_name,
                            map_type.key(),
                        )?,
                        Let::new(
                            Some(
                                branch
                                    .secondary_name()
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
