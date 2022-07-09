use crate::{context::Context, error::CompileError, reference_count, type_};

pub fn compile_global_variable(
    context: &Context,
    type_: &mir::types::Type,
) -> Result<(), CompileError> {
    context.module_builder().define_variable(
        type_::compile_id(type_),
        fmm::build::record(vec![
            reference_count::variant::compile_clone_function(context, type_)?,
            reference_count::variant::compile_drop_function(context, type_)?,
            reference_count::variant::compile_synchronize_function(context, type_)?,
        ]),
        false,
        fmm::ir::Linkage::Weak,
        None,
    );

    Ok(())
}

pub fn get_clone_function(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_function(builder, tag, 0)
}

pub fn get_drop_function(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_function(builder, tag, 1)
}

pub fn get_synchronize_function(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    get_function(builder, tag, 2)
}

fn get_function(
    builder: &fmm::build::InstructionBuilder,
    tag: impl Into<fmm::build::TypedExpression>,
    index: usize,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.deconstruct_record(builder.load(tag)?, index)?)
}
