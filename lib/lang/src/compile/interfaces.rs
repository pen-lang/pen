use super::{error::CompileError, type_extraction};
use crate::{hir, interface};

pub fn compile(module: &hir::Module) -> Result<interface::Module, CompileError> {
    Ok(interface::Module::new(
        module
            .type_definitions()
            .iter()
            .map(|definition| {
                interface::TypeDefinition::new(
                    definition.name(),
                    definition.elements().to_vec(),
                    definition.is_open(),
                    definition.is_public() && !definition.is_external(),
                    definition.position().clone(),
                )
            })
            .collect(),
        module
            .type_aliases()
            .iter()
            .map(|alias| {
                interface::TypeAlias::new(
                    alias.name(),
                    alias.type_().clone(),
                    alias.is_public() && !alias.is_external(),
                )
            })
            .collect(),
        module
            .definitions()
            .iter()
            .map(|definition| {
                interface::Declaration::new(
                    definition.name(),
                    type_extraction::extract_from_lambda(definition.lambda()),
                )
            })
            .collect(),
    ))
}
