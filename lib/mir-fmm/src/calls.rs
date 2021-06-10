use crate::{closures, types, CompileError};

pub fn compile(
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(instruction_builder.call(
        closures::compile_load_entry_function(instruction_builder, closure_pointer.clone())?,
        vec![fmm::build::bit_cast(
            types::compile_untyped_closure_pointer(),
            closure_pointer.clone(),
        )
        .into()]
        .into_iter()
        .chain(arguments.iter().cloned())
        .collect(),
    )?)
}
