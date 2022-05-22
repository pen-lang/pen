use crate::{closure, types, CompileError};

pub fn compile(
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_pointer = closure::compile_entry_function_pointer(closure_pointer.clone())?;

    Ok(instruction_builder.call(
        if arguments.is_empty() {
            // Entry functions of thunks need to be loaded atomically
            // to make thunk update thread-safe.
            instruction_builder
                .atomic_load(entry_function_pointer, fmm::ir::AtomicOrdering::Acquire)?
        } else {
            instruction_builder.load(entry_function_pointer)?
        },
        [fmm::build::bit_cast(
            types::compile_untyped_closure_pointer(),
            closure_pointer.clone(),
        )
        .into()]
        .into_iter()
        .chain(arguments.iter().cloned())
        .collect(),
    )?)
}
