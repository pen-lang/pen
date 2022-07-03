use crate::{context::CompileContext, expression, type_, CompileError};
use hir::{
    analysis::{type_canonicalizer, AnalysisError},
    ir::*,
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
            type_::compile_function(&function_type, context)?,
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
    })
}
