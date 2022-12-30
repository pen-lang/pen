use crate::{
    context::Context,
    downcast, error_type, expression,
    runtime_function_declaration::{
        LOCAL_DEBUG_FUNCTION_NAME, LOCAL_RACE_FUNCTION_NAME, LOCAL_SPAWN_FUNCTION_NAME,
    },
    transformation::map_context,
    type_, type_information, CompileError,
};
use hir::{
    analysis::{type_canonicalizer, AnalysisError},
    ir::*,
    types,
    types::Type,
};
use position::Position;

pub fn compile(
    context: &Context,
    call: &Call,
    function: &BuiltInFunction,
) -> Result<mir::ir::Expression, CompileError> {
    let position = call.position();
    let function_type = call
        .function_type()
        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
    let function_type = type_canonicalizer::canonicalize_function(function_type, context.types())?
        .ok_or_else(|| {
            AnalysisError::FunctionExpected(
                call.function().position().clone(),
                function_type.clone(),
            )
        })?;
    let arguments = call
        .arguments()
        .iter()
        .map(|argument| expression::compile(context, argument))
        .collect::<Result<Vec<_>, _>>()?;
    let compile_call = |function, arguments| -> Result<_, CompileError> {
        Ok(mir::ir::Call::new(
            type_::compile_function(context, &function_type)?,
            function,
            arguments,
        ))
    };

    Ok(match function.name() {
        BuiltInFunctionName::Debug => mir::ir::Call::new(
            mir::types::Function::new(vec![mir::types::Type::ByteString], mir::types::Type::None),
            mir::ir::Variable::new(LOCAL_DEBUG_FUNCTION_NAME),
            vec![type_information::debug::compile_call(arguments[0].clone())],
        )
        .into(),
        BuiltInFunctionName::Delete => {
            let map_type =
                type_canonicalizer::canonicalize_map(function_type.result(), context.types())?
                    .ok_or_else(|| {
                        AnalysisError::MapExpected(
                            call.position().clone(),
                            function_type.result().clone(),
                        )
                    })?;
            let mir_map_type = type_::compile_map(context)?;

            mir::ir::Call::new(
                mir::types::Function::new(
                    vec![
                        type_::compile_map_context(context)?.into(),
                        mir_map_type.clone().into(),
                        mir::types::Type::Variant,
                    ],
                    mir_map_type,
                ),
                mir::ir::Variable::new(&context.configuration()?.map_type.delete_function_name),
                vec![
                    expression::compile(
                        context,
                        &map_context::expression::transform(context, &map_type)?,
                    )?,
                    arguments[0].clone(),
                    expression::compile(
                        context,
                        &TypeCoercion::new(
                            function_type.arguments()[1].clone(),
                            types::Any::new(position.clone()),
                            call.arguments()[1].clone(),
                            position.clone(),
                        )
                        .into(),
                    )?,
                ],
            )
            .into()
        }
        BuiltInFunctionName::Error => error_type::compile_error(arguments[0].clone()),
        BuiltInFunctionName::Keys => {
            let argument_type = &function_type.arguments()[0];
            let argument = &call.arguments()[0];

            compile_map_iteration(
                context,
                argument,
                &context
                    .configuration()?
                    .map_type
                    .iteration
                    .key_function_name,
                type_canonicalizer::canonicalize_map(argument_type, context.types())?
                    .ok_or_else(|| {
                        AnalysisError::MapExpected(
                            argument.position().clone(),
                            argument_type.clone(),
                        )
                    })?
                    .key(),
                position,
            )?
        }
        BuiltInFunctionName::Race => {
            const ELEMENT_NAME: &str = "$element";

            let list_type =
                type_canonicalizer::canonicalize_list(function_type.result(), context.types())?
                    .ok_or_else(|| {
                        AnalysisError::ListExpected(
                            call.position().clone(),
                            function_type.result().clone(),
                        )
                    })?;
            let any_list_type =
                types::List::new(types::Any::new(position.clone()), position.clone());

            compile_call(
                mir::ir::Variable::new(LOCAL_RACE_FUNCTION_NAME),
                vec![expression::compile(
                    context,
                    &ListComprehension::new(
                        any_list_type,
                        Call::new(
                            Some(
                                types::Function::new(vec![], list_type.clone(), position.clone())
                                    .into(),
                            ),
                            Variable::new(ELEMENT_NAME, position.clone()),
                            vec![],
                            position.clone(),
                        ),
                        vec![ListComprehensionBranch::new(
                            vec![ELEMENT_NAME.into()],
                            vec![ListComprehensionIteratee::new(
                                Some(types::List::new(list_type, call.position().clone()).into()),
                                call.arguments()[0].clone(),
                            )],
                            None,
                            position.clone(),
                        )],
                        position.clone(),
                    )
                    .into(),
                )?],
            )?
            .into()
        }
        BuiltInFunctionName::ReflectDebug => {
            type_information::debug::compile_call(arguments[0].clone())
        }
        BuiltInFunctionName::ReflectEqual => {
            type_information::equal::compile_call(arguments[0].clone(), arguments[1].clone())
        }
        BuiltInFunctionName::Size => mir::ir::Call::new(
            type_::compile_function(context, &function_type)?,
            match &function_type.arguments()[0] {
                Type::List(_) => {
                    mir::ir::Variable::new(&context.configuration()?.list_type.size_function_name)
                }
                Type::Map(_) => {
                    mir::ir::Variable::new(&context.configuration()?.map_type.size_function_name)
                }
                _ => unreachable!(),
            },
            arguments,
        )
        .into(),
        BuiltInFunctionName::Source => error_type::compile_source(arguments[0].clone()),
        BuiltInFunctionName::Spawn => {
            const ANY_THUNK_NAME: &str = "$any_thunk";
            const THUNK_NAME: &str = "$thunk";

            let function_type = &function_type.arguments()[0];
            let function_type =
                type_canonicalizer::canonicalize_function(function_type, context.types())?
                    .ok_or_else(|| {
                        AnalysisError::FunctionExpected(
                            call.arguments()[0].position().clone(),
                            function_type.clone(),
                        )
                    })?;
            let result_type = function_type.result();
            let any_type = Type::from(types::Any::new(position.clone()));
            let thunk_type =
                types::Function::new(vec![], any_type.clone(), position.clone()).into();
            let mir_thunk_type = type_::compile(context, &thunk_type)?;

            mir::ir::Let::new(
                ANY_THUNK_NAME,
                mir_thunk_type.clone(),
                mir::ir::Call::new(
                    type_::compile_spawn_function(),
                    mir::ir::Variable::new(LOCAL_SPAWN_FUNCTION_NAME),
                    vec![mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::thunk(
                            ANY_THUNK_NAME,
                            type_::compile(context, &any_type)?,
                            expression::compile(
                                context,
                                &TypeCoercion::new(
                                    result_type.clone(),
                                    any_type.clone(),
                                    Call::new(
                                        Some(function_type.clone().into()),
                                        call.arguments()[0].clone(),
                                        vec![],
                                        position.clone(),
                                    ),
                                    position.clone(),
                                )
                                .into(),
                            )?,
                        ),
                        mir::ir::Synchronize::new(
                            mir_thunk_type,
                            mir::ir::Variable::new(ANY_THUNK_NAME),
                        ),
                    )
                    .into()],
                ),
                mir::ir::LetRecursive::new(
                    mir::ir::FunctionDefinition::new(
                        THUNK_NAME,
                        vec![],
                        type_::compile(context, result_type)?,
                        expression::compile(
                            context,
                            &downcast::compile(
                                context,
                                &any_type,
                                result_type,
                                &Call::new(
                                    Some(thunk_type),
                                    Variable::new(ANY_THUNK_NAME, position.clone()),
                                    vec![],
                                    position.clone(),
                                )
                                .into(),
                            )?,
                        )?,
                    ),
                    mir::ir::Variable::new(THUNK_NAME),
                ),
            )
            .into()
        }
        BuiltInFunctionName::Values => {
            let argument_type = &function_type.arguments()[0];
            let argument = &call.arguments()[0];

            compile_map_iteration(
                context,
                argument,
                &context
                    .configuration()?
                    .map_type
                    .iteration
                    .value_function_name,
                type_canonicalizer::canonicalize_map(argument_type, context.types())?
                    .ok_or_else(|| {
                        AnalysisError::MapExpected(
                            argument.position().clone(),
                            argument_type.clone(),
                        )
                    })?
                    .value(),
                position,
            )?
        }
    })
}

fn compile_map_iteration(
    context: &Context,
    argument: &Expression,
    element_function_name: &str,
    element_type: &Type,
    position: &Position,
) -> Result<mir::ir::Expression, CompileError> {
    const CLOSURE_NAME: &str = "$loop";

    let list_type = type_::compile_list(context)?;
    let definition = compile_map_iteration_function_definition(
        context,
        element_function_name,
        element_type,
        position,
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
                            vec![expression::compile(context, argument)?],
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
    element_function_name: &str,
    element_type: &Type,
    position: &Position,
) -> Result<mir::ir::FunctionDefinition, CompileError> {
    const CLOSURE_NAME: &str = "$loop";
    const ITERATOR_NAME: &str = "$iterator";

    let iteration_configuration = &context.configuration()?.map_type.iteration;
    let any_type = Type::from(types::Any::new(position.clone()));
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
                    List::new(
                        element_type.clone(),
                        vec![
                            ListElement::Single(downcast::compile(
                                context,
                                &any_type,
                                element_type,
                                &Call::new(
                                    Some(
                                        types::Function::new(
                                            vec![iterator_type.clone()],
                                            any_type.clone(),
                                            position.clone(),
                                        )
                                        .into(),
                                    ),
                                    Variable::new(element_function_name, position.clone()),
                                    vec![iterator_variable.clone().into()],
                                    position.clone(),
                                )
                                .into(),
                            )?),
                            ListElement::Multiple(
                                Call::new(
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
                                                vec![iterator_type],
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
                                )
                                .into(),
                            ),
                        ],
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use position::{test::PositionFake, Position};

    fn compile_call(call: &Call) -> Result<mir::ir::Expression, CompileError> {
        compile(
            &Context::dummy(
                [
                    (
                        COMPILE_CONFIGURATION.list_type.list_type_name.clone(),
                        types::None::new(Position::fake()).into(),
                    ),
                    (
                        COMPILE_CONFIGURATION.map_type.context_type_name.clone(),
                        types::None::new(Position::fake()).into(),
                    ),
                    (
                        COMPILE_CONFIGURATION.map_type.map_type_name.clone(),
                        types::None::new(Position::fake()).into(),
                    ),
                    (
                        COMPILE_CONFIGURATION
                            .map_type
                            .iteration
                            .iterator_type_name
                            .clone(),
                        types::None::new(Position::fake()).into(),
                    ),
                ]
                .into_iter()
                .collect(),
                Default::default(),
            ),
            call,
            if let Expression::BuiltInFunction(function) = call.function() {
                function
            } else {
                unreachable!()
            },
        )
    }

    #[test]
    fn compile_debug() {
        insta::assert_debug_snapshot!(compile_call(&Call::new(
            Some(
                types::Function::new(
                    vec![types::ByteString::new(Position::fake()).into()],
                    types::None::new(Position::fake()),
                    Position::fake()
                )
                .into()
            ),
            BuiltInFunction::new(BuiltInFunctionName::Debug, Position::fake()),
            vec![TypeCoercion::new(
                types::None::new(Position::fake()),
                types::Any::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake()
            )
            .into()],
            Position::fake(),
        )));
    }

    #[test]
    fn compile_keys() {
        insta::assert_debug_snapshot!(compile_call(&Call::new(
            Some(
                types::Function::new(
                    vec![types::Map::new(
                        types::ByteString::new(Position::fake()),
                        types::Number::new(Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    types::List::new(types::ByteString::new(Position::fake()), Position::fake()),
                    Position::fake()
                )
                .into()
            ),
            BuiltInFunction::new(BuiltInFunctionName::Keys, Position::fake()),
            vec![Map::new(
                types::ByteString::new(Position::fake()),
                types::Number::new(Position::fake()),
                vec![],
                Position::fake()
            )
            .into()],
            Position::fake(),
        )));
    }

    #[test]
    fn compile_values() {
        insta::assert_debug_snapshot!(compile_call(&Call::new(
            Some(
                types::Function::new(
                    vec![types::Map::new(
                        types::ByteString::new(Position::fake()),
                        types::Number::new(Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    types::List::new(types::ByteString::new(Position::fake()), Position::fake()),
                    Position::fake()
                )
                .into()
            ),
            BuiltInFunction::new(BuiltInFunctionName::Values, Position::fake()),
            vec![Map::new(
                types::ByteString::new(Position::fake()),
                types::Number::new(Position::fake()),
                vec![],
                Position::fake()
            )
            .into()],
            Position::fake(),
        )));
    }

    mod spawn {
        use super::*;
        use pretty_assertions::assert_eq;

        #[test]
        fn compile() {
            let function_type = types::Function::new(
                vec![],
                types::Number::new(Position::fake()),
                Position::fake(),
            );
            let thunk_type = mir::types::Function::new(vec![], mir::types::Type::Variant);

            assert_eq!(
                compile_call(&Call::new(
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
                        types::Number::new(Position::fake()),
                        Number::new(42.0, Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    Position::fake(),
                ),),
                Ok(mir::ir::Let::new(
                    "$any_thunk",
                    thunk_type.clone(),
                    mir::ir::Call::new(
                        type_::compile_spawn_function(),
                        mir::ir::Variable::new(LOCAL_SPAWN_FUNCTION_NAME),
                        vec![mir::ir::LetRecursive::new(
                            mir::ir::FunctionDefinition::thunk(
                                "$any_thunk",
                                mir::types::Type::Variant,
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Call::new(
                                        mir::types::Function::new(vec![], mir::types::Type::Number),
                                        mir::ir::LetRecursive::new(
                                            mir::ir::FunctionDefinition::new(
                                                "$closure",
                                                vec![],
                                                mir::types::Type::Number,
                                                mir::ir::Expression::Number(42.0),
                                            ),
                                            mir::ir::Variable::new("$closure")
                                        ),
                                        vec![]
                                    ),
                                ),
                            ),
                            mir::ir::Synchronize::new(
                                thunk_type.clone(),
                                mir::ir::Variable::new("$any_thunk")
                            ),
                        )
                        .into()]
                    ),
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::new(
                            "$thunk",
                            vec![],
                            mir::types::Type::Number,
                            mir::ir::Case::new(
                                mir::ir::Call::new(
                                    thunk_type,
                                    mir::ir::Variable::new("$any_thunk"),
                                    vec![]
                                ),
                                vec![mir::ir::Alternative::new(
                                    vec![mir::types::Type::Number],
                                    "$value",
                                    mir::ir::Variable::new("$value")
                                )],
                                None,
                            ),
                        ),
                        mir::ir::Variable::new("$thunk"),
                    ),
                )
                .into())
            );
        }

        #[test]
        fn compile_with_any_type() {
            let function_type =
                types::Function::new(vec![], types::Any::new(Position::fake()), Position::fake());
            let thunk_type = mir::types::Function::new(vec![], mir::types::Type::Variant);

            assert_eq!(
                compile_call(&Call::new(
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
                        types::Any::new(Position::fake()),
                        Variable::new("x", Position::fake()),
                        Position::fake()
                    )
                    .into()],
                    Position::fake(),
                ),),
                Ok(mir::ir::Let::new(
                    "$any_thunk",
                    thunk_type.clone(),
                    mir::ir::Call::new(
                        type_::compile_spawn_function(),
                        mir::ir::Variable::new(LOCAL_SPAWN_FUNCTION_NAME),
                        vec![mir::ir::LetRecursive::new(
                            mir::ir::FunctionDefinition::thunk(
                                "$any_thunk",
                                mir::types::Type::Variant,
                                mir::ir::Call::new(
                                    thunk_type.clone(),
                                    mir::ir::LetRecursive::new(
                                        mir::ir::FunctionDefinition::new(
                                            "$closure",
                                            vec![],
                                            mir::types::Type::Variant,
                                            mir::ir::Variable::new("x"),
                                        ),
                                        mir::ir::Variable::new("$closure")
                                    ),
                                    vec![]
                                ),
                            ),
                            mir::ir::Synchronize::new(
                                thunk_type.clone(),
                                mir::ir::Variable::new("$any_thunk")
                            ),
                        )
                        .into()]
                    ),
                    mir::ir::LetRecursive::new(
                        mir::ir::FunctionDefinition::new(
                            "$thunk",
                            vec![],
                            mir::types::Type::Variant,
                            mir::ir::Call::new(
                                thunk_type,
                                mir::ir::Variable::new("$any_thunk"),
                                vec![]
                            ),
                        ),
                        mir::ir::Variable::new("$thunk"),
                    ),
                )
                .into())
            );
        }
    }
}
