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
        .ok_or_else(|| AnalysisError::FunctionExpected(function_type.clone()))?;
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
                    .ok_or_else(|| AnalysisError::MapExpected(function_type.result().clone()))?;
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
        BuiltInFunctionName::Race => {
            const ELEMENT_NAME: &str = "$element";

            let list_type =
                type_canonicalizer::canonicalize_list(function_type.result(), context.types())?
                    .ok_or_else(|| AnalysisError::ListExpected(function_type.result().clone()))?;
            let any_list_type =
                types::List::new(types::Any::new(position.clone()), position.clone());

            compile_call(
                mir::ir::Variable::new(LOCAL_RACE_FUNCTION_NAME),
                vec![expression::compile(
                    context,
                    &ListComprehension::new(
                        any_list_type,
                        Call::new(
                            Some(types::Function::new(vec![], list_type, position.clone()).into()),
                            Variable::new(ELEMENT_NAME, position.clone()),
                            vec![],
                            position.clone(),
                        ),
                        vec![ListComprehensionBranch::new(
                            Some(
                                types::List::new(list_type.clone(), call.position().clone()).into(),
                            ),
                            ELEMENT_NAME,
                            None,
                            call.arguments()[0].clone(),
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
                    .ok_or_else(|| AnalysisError::FunctionExpected(function_type.clone()))?;
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
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{test::PositionFake, Position};

    fn compile_call(call: &Call) -> Result<mir::ir::Expression, CompileError> {
        compile(
            &Context::dummy(Default::default(), Default::default()),
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
