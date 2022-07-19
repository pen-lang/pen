use crate::{
    context::CompileContext,
    downcast, expression, type_,
    utility_function_declaration::{LOCAL_DEBUG_FUNCTION_NAME, LOCAL_SPAWN_FUNCTION_NAME},
    CompileError,
};
use hir::{
    analysis::{type_canonicalizer, AnalysisError},
    ir::*,
    types,
    types::Type,
};

pub fn compile(
    context: &CompileContext,
    call: &BuiltInCall,
) -> Result<mir::ir::Expression, CompileError> {
    let position = call.position();
    let function_type = type_canonicalizer::canonicalize_function(
        call.function_type()
            .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?,
        context.types(),
    )?
    .ok_or_else(|| AnalysisError::FunctionExpected(position.clone()))?;
    let arguments = call
        .arguments()
        .iter()
        .map(|argument| expression::compile(context, argument))
        .collect::<Result<Vec<_>, _>>()?;
    let compile_call = |function| -> Result<_, CompileError> {
        Ok(mir::ir::Call::new(
            type_::compile_function(
                context,
                &type_canonicalizer::canonicalize_function(
                    call.function_type()
                        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?,
                    context.types(),
                )?
                .ok_or_else(|| AnalysisError::FunctionExpected(position.clone()))?,
            )?,
            function,
            arguments.clone(),
        ))
    };

    Ok(match call.function() {
        BuiltInFunction::Debug => {
            compile_call(mir::ir::Variable::new(LOCAL_DEBUG_FUNCTION_NAME))?.into()
        }
        BuiltInFunction::Error => compile_call(mir::ir::Variable::new(
            &context.configuration()?.error_type.error_function_name,
        ))?
        .into(),
        BuiltInFunction::Size => mir::ir::Call::new(
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
        BuiltInFunction::Source => compile_call(mir::ir::Variable::new(
            &context.configuration()?.error_type.source_function_name,
        ))?
        .into(),
        BuiltInFunction::Spawn => {
            const ANY_THUNK_NAME: &str = "$any_thunk";
            const THUNK_NAME: &str = "$thunk";

            let spawned_function_type = type_canonicalizer::canonicalize_function(
                &function_type.arguments()[0],
                context.types(),
            )?
            .ok_or_else(|| AnalysisError::FunctionExpected(position.clone()))?;
            let result_type = spawned_function_type.result();
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
                            expression::compile(
                                context,
                                &TypeCoercion::new(
                                    result_type.clone(),
                                    any_type.clone(),
                                    Call::new(
                                        Some(spawned_function_type.clone().into()),
                                        call.arguments()[0].clone(),
                                        vec![],
                                        position.clone(),
                                    ),
                                    position.clone(),
                                )
                                .into(),
                            )?,
                            type_::compile(context, &any_type)?,
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
                        type_::compile(context, result_type)?,
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

    fn compile_call(call: &BuiltInCall) -> Result<mir::ir::Expression, CompileError> {
        compile(
            &CompileContext::dummy(Default::default(), Default::default()),
            call,
        )
    }

    #[test]
    fn compile_debug() {
        assert_eq!(
            compile_call(&BuiltInCall::new(
                Some(
                    types::Function::new(
                        vec![types::ByteString::new(Position::fake()).into()],
                        types::None::new(Position::fake()),
                        Position::fake()
                    )
                    .into()
                ),
                BuiltInFunction::Debug,
                vec![ByteString::new(vec![], Position::fake()).into()],
                Position::fake(),
            ),),
            Ok(mir::ir::Call::new(
                mir::types::Function::new(
                    vec![mir::types::Type::ByteString],
                    mir::types::Type::None
                ),
                mir::ir::Variable::new(LOCAL_DEBUG_FUNCTION_NAME),
                vec![mir::ir::ByteString::new(vec![]).into()],
            )
            .into())
        );
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
                compile_call(&BuiltInCall::new(
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
                                mir::ir::Variant::new(
                                    mir::types::Type::Number,
                                    mir::ir::Call::new(
                                        mir::types::Function::new(vec![], mir::types::Type::Number),
                                        mir::ir::LetRecursive::new(
                                            mir::ir::FunctionDefinition::new(
                                                "$closure",
                                                vec![],
                                                mir::ir::Expression::Number(42.0),
                                                mir::types::Type::Number
                                            ),
                                            mir::ir::Variable::new("$closure")
                                        ),
                                        vec![]
                                    ),
                                ),
                                mir::types::Type::Variant
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
                            mir::ir::Case::new(
                                mir::ir::Call::new(
                                    thunk_type,
                                    mir::ir::Variable::new("$any_thunk"),
                                    vec![]
                                ),
                                vec![mir::ir::Alternative::new(
                                    vec![mir::types::Type::Number.into()],
                                    "$value",
                                    mir::ir::Variable::new("$value")
                                )],
                                None,
                            ),
                            mir::types::Type::Number
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
                compile_call(&BuiltInCall::new(
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
                                mir::ir::Call::new(
                                    thunk_type.clone(),
                                    mir::ir::LetRecursive::new(
                                        mir::ir::FunctionDefinition::new(
                                            "$closure",
                                            vec![],
                                            mir::ir::Variable::new("x"),
                                            mir::types::Type::Variant
                                        ),
                                        mir::ir::Variable::new("$closure")
                                    ),
                                    vec![]
                                ),
                                mir::types::Type::Variant
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
                            mir::ir::Call::new(
                                thunk_type,
                                mir::ir::Variable::new("$any_thunk"),
                                vec![]
                            ),
                            mir::types::Type::Variant
                        ),
                        mir::ir::Variable::new("$thunk"),
                    ),
                )
                .into())
            );
        }
    }
}
