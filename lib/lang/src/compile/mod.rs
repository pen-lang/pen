mod error;
mod interfaces;
mod type_context;
mod type_extraction;
mod type_inference;
mod type_resolution;
mod union_types;

use crate::{hir::Module, interface};
pub use error::CompileError;

pub fn compile(module: &Module) -> Result<(Vec<u8>, interface::Module), CompileError> {
    let module = type_inference::infer_types(module)?;

    Ok((
        fmm_llvm::compile_to_bit_code(
            &mir_fmm::compile(&mir::ir::Module::new(
                vec![],
                vec![],
                vec![],
                vec![],
                vec![],
            ))?,
            &fmm_llvm::HeapConfiguration {
                allocate_function_name: "malloc".into(),
                reallocate_function_name: "realloc".into(),
                free_function_name: "free".into(),
            },
            None,
        )?,
        interfaces::compile(&module)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn compile_empty_module() -> Result<(), CompileError> {
        compile(&Module::new(vec![], vec![], vec![], vec![]))?;

        Ok(())
    }
}
