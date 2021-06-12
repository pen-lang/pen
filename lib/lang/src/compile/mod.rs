mod error;
mod expressions;
mod interfaces;
mod list_type_configuration;
mod modules;
mod type_canonicalization;
mod type_compilation;
mod type_context;
mod type_equality;
mod type_extraction;
mod type_inference;
mod type_resolution;
mod union_types;

use self::type_context::TypeContext;
use crate::{hir::Module, interface};
pub use error::CompileError;
use list_type_configuration::ListTypeConfiguration;

pub fn compile(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
) -> Result<(Vec<u8>, interface::Module), CompileError> {
    let type_context = TypeContext::new(module, list_type_configuration);
    let module = type_inference::infer_types(module, type_context.types())?;

    Ok((
        fmm_llvm::compile_to_bit_code(
            &mir_fmm::compile(&modules::compile(&module, &type_context)?)?,
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
    use super::{list_type_configuration::LIST_TYPE_CONFIGURATION, *};

    #[test]
    fn compile_empty_module() -> Result<(), CompileError> {
        compile(
            &Module::new(vec![], vec![], vec![], vec![]),
            &LIST_TYPE_CONFIGURATION,
        )?;

        Ok(())
    }
}
