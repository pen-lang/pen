use super::{super::error::CompileError, pointer};
use crate::{
    context::Context, reference_count::REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS, type_,
};

const DROP_FUNCTION_NAME: &str = "mir:drop:string";
const SYNCHRONIZE_FUNCTION_NAME: &str = "mir:synchronize:string";

pub fn compile_drop_function(context: &Context) -> Result<(), CompileError> {
    const ARGUMENT_NAME: &str = "x";
    let string_type = type_::compile_string();

    context.module_builder().define_function(
        DROP_FUNCTION_NAME,
        vec![fmm::ir::Argument::new(ARGUMENT_NAME, string_type.clone())],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            pointer::drop(
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, string_type.clone()),
                |_| Ok(()),
            )?;

            Ok(builder.return_(fmm::build::TypedExpression::new(
                fmm::ir::void_value(),
                fmm::types::void_type(),
            )))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )?;

    Ok(())
}

pub fn compile_synchronize_function(context: &Context) -> Result<(), CompileError> {
    const ARGUMENT_NAME: &str = "x";
    let string_type = type_::compile_string();

    context.module_builder().define_function(
        SYNCHRONIZE_FUNCTION_NAME,
        vec![fmm::ir::Argument::new(ARGUMENT_NAME, string_type.clone())],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            pointer::synchronize(
                &builder,
                &fmm::build::variable(ARGUMENT_NAME, string_type.clone()),
                |_| Ok(()),
            )?;

            Ok(builder.return_(fmm::build::TypedExpression::new(
                fmm::ir::void_value(),
                fmm::types::void_type(),
            )))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )?;

    Ok(())
}

pub fn clone(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    pointer::clone(builder, expression)
}

pub fn drop(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    builder.call(
        fmm::build::variable(
            DROP_FUNCTION_NAME,
            fmm::types::Function::new(
                vec![type_::compile_string().into()],
                fmm::types::void_type(),
                fmm::types::CallingConvention::Target,
            ),
        ),
        vec![expression.clone()],
    )?;

    Ok(())
}

pub fn synchronize(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    builder.call(
        fmm::build::variable(
            SYNCHRONIZE_FUNCTION_NAME,
            fmm::types::Function::new(
                vec![type_::compile_string().into()],
                fmm::types::void_type(),
                fmm::types::CallingConvention::Target,
            ),
        ),
        vec![expression.clone()],
    )?;

    Ok(())
}
