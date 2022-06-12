use crate::{closure, type_, CompileError};

pub fn compile(
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_pointer = closure::get_entry_function_pointer(closure_pointer.clone())?;

    Ok(instruction_builder.call(
        if arguments.is_empty() {
            // Entry functions of thunks need to be loaded atomically
            // to make thunk update thread-safe.
            //
            // Relaxed ordering should be fine here since entry functions themselves should
            // guarantee memory operation ordering..
            instruction_builder
                .atomic_load(entry_function_pointer, fmm::ir::AtomicOrdering::Relaxed)?
        } else {
            instruction_builder.load(entry_function_pointer)?
        },
        [fmm::build::bit_cast(
            type_::compile_untyped_closure_pointer(),
            closure_pointer.clone(),
        )
        .into()]
        .into_iter()
        .chain(arguments.iter().cloned())
        .collect(),
    )?)
}
