use super::{drop, sync};
use crate::{context::Context, CompileError};
use std::sync::LazyLock;

static VARIABLE_DEFINITION_OPTIONS: LazyLock<fmm::ir::VariableDefinitionOptions> =
    LazyLock::new(|| {
        fmm::ir::VariableDefinitionOptions::new()
            .set_address_named(false)
            .set_linkage(fmm::ir::Linkage::Internal)
            .set_mutable(false)
    });

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
        VARIABLE_DEFINITION_OPTIONS.clone(),
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
        VARIABLE_DEFINITION_OPTIONS.clone(),
    ))
}

pub fn load_drop_function(
    builder: &fmm::build::InstructionBuilder,
    metadata_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.load(fmm::build::record_address(metadata_pointer, 0)?)?)
}

pub fn load_synchronize_function(
    builder: &fmm::build::InstructionBuilder,
    metadata_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.load(fmm::build::record_address(metadata_pointer, 1)?)?)
}
