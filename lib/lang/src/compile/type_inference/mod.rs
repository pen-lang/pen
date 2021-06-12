use crate::hir::Module;

pub fn infer_types(
    module: &Module,
    type_context: &TypeContext,
) -> Result<Module, TypeInferenceError> {
    Ok(module.clone())
}
