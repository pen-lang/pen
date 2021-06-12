mod error;
mod module_interfaces;

use crate::hir::{Module, ModuleInterface};
pub use error::CompileError;

pub fn compile(module: &Module) -> Result<(Vec<u8>, ModuleInterface), CompileError> {
    Ok((
        fmm_llvm::compile_to_bitcode(
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
        module_interfaces::compile(module)?,
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
