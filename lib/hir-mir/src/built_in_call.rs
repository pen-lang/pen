use crate::{
    concurrency::MODULE_LOCAL_SPAWN_FUNCTION_NAME, context::CompileContext, downcast, expression,
    type_, CompileError,
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
        .collect::<Result<_, _>>()?;

    Ok(match call.function() {
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
                    mir::ir::Variable::new(MODULE_LOCAL_SPAWN_FUNCTION_NAME),
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
