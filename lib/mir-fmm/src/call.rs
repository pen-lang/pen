use crate::{CompileError, closure, type_};

pub fn compile(
    instruction_builder: &fmm::build::InstructionBuilder,
    closure_pointer: &fmm::build::TypedExpression,
    arguments: &[fmm::build::TypedExpression],
) -> Result<fmm::build::TypedExpression, CompileError> {
    let entry_function_pointer = closure::get_entry_function_pointer(closure_pointer.clone())?;

    Ok(instruction_builder.call(
        if arguments.is_empty() {
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
