use super::{super::error::CompileError, pointer};
use crate::{
    closure, context::Context, reference_count::REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS, type_,
};

const DROP_FUNCTION_NAME: &str = "mir:drop:function";
const SYNCHRONIZE_FUNCTION_NAME: &str = "mir:synchronize:function";

pub fn compile_drop_function(context: &Context) -> Result<(), CompileError> {
    const ARGUMENT_NAME: &str = "x";
    let closure_type = compile_closure_type(context);

    context.module_builder().define_function(
        DROP_FUNCTION_NAME,
        vec![fmm::ir::Argument::new(ARGUMENT_NAME, closure_type.clone())],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let closure_pointer = fmm::build::variable(ARGUMENT_NAME, closure_type.clone());

            pointer::drop(&builder, &closure_pointer, |builder| {
                builder.call(
                    closure::metadata::load_drop_function(
                        builder,
                        builder.load(closure::get_metadata_pointer(closure_pointer.clone())?)?,
                    )?,
                    vec![
                        fmm::build::bit_cast(
                            fmm::types::Primitive::PointerInteger,
                            closure_pointer.clone(),
                        )
                        .into(),
                    ],
                )?;

                Ok(())
            })?;

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
    let closure_type = compile_closure_type(context);

    context.module_builder().define_function(
        SYNCHRONIZE_FUNCTION_NAME,
        vec![fmm::ir::Argument::new(ARGUMENT_NAME, closure_type.clone())],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            let closure_pointer = fmm::build::variable(ARGUMENT_NAME, closure_type.clone());

            pointer::synchronize(&builder, &closure_pointer, |builder| {
                builder.call(
                    closure::metadata::load_synchronize_function(
                        builder,
                        builder.load(closure::get_metadata_pointer(closure_pointer.clone())?)?,
                    )?,
                    vec![
                        fmm::build::bit_cast(
                            fmm::types::Primitive::PointerInteger,
                            closure_pointer.clone(),
                        )
                        .into(),
                    ],
                )?;

                Ok(())
            })?;

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
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    pointer::clone(builder, closure_pointer)
}

pub fn drop(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    let closure_type = compile_closure_type(context);

    builder.call(
        fmm::build::variable(
            DROP_FUNCTION_NAME,
            fmm::types::Function::new(
                vec![closure_type.clone()],
                fmm::types::void_type(),
                fmm::types::CallingConvention::Target,
            ),
        ),
        vec![fmm::build::bit_cast(closure_type, closure_pointer.clone()).into()],
    )?;

    Ok(())
}

pub fn synchronize(
    context: &Context,
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
) -> Result<(), CompileError> {
    let closure_type = compile_closure_type(context);

    builder.call(
        fmm::build::variable(
            SYNCHRONIZE_FUNCTION_NAME,
            fmm::types::Function::new(
                vec![closure_type.clone()],
                fmm::types::void_type(),
                fmm::types::CallingConvention::Target,
            ),
        ),
        vec![fmm::build::bit_cast(closure_type, closure_pointer.clone()).into()],
    )?;

    Ok(())
}

fn compile_closure_type(context: &Context) -> fmm::types::Type {
    type_::compile_function(
        context,
        &mir::types::Function::new(vec![], mir::types::Type::None),
    )
}
