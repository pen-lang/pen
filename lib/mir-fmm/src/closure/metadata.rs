use super::{drop, sync};
use crate::{context::Context, reference_count, CompileError};

// We do not need to compile closure metadata for thunks in the middle of
// evaluation because of the following reasons.
//
// - While thunks are evaluated, evaluator threads keep references at least. So
//   drop functions are never called.
//   - Also, they are never expected to be dropped during evaluation.
// - If thunks are already synchronized, we do not call synchronization
//   functions in closure metadata.

pub fn compile(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(context.module_builder().define_anonymous_variable(
        fmm::build::record(vec![
            drop::compile(context, definition)?,
            sync::compile(context, definition)?,
        ]),
        false,
        None,
    ))
}

pub fn compile_normal_thunk(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(context.module_builder().define_anonymous_variable(
        fmm::build::record(vec![
            drop::compile_normal_thunk(context, definition)?,
            sync::compile_normal_thunk(context, definition)?,
        ]),
        false,
        None,
    ))
}

pub fn load_drop_function(
    builder: &fmm::build::InstructionBuilder,
    metadata_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.load(fmm::build::record_address(
        reference_count::pointer::untag(&metadata_pointer.into())?,
        0,
    )?)?)
}

pub fn load_synchronize_function(
    builder: &fmm::build::InstructionBuilder,
    metadata_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.load(fmm::build::record_address(
        reference_count::pointer::untag(&metadata_pointer.into())?,
        1,
    )?)?)
}
