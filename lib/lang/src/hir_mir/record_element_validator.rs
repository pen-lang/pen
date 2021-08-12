use super::{type_context::TypeContext, CompileError};
use crate::hir::*;
use std::collections::HashMap;

pub fn validate(module: &Module, _type_context: &TypeContext) -> Result<(), CompileError> {
    let _ = are_records_open(module.type_definitions());

    Ok(())
}

fn are_records_open(type_definitions: &[TypeDefinition]) -> HashMap<String, bool> {
    type_definitions
        .iter()
        .map(|definition| (definition.name().into(), is_record_open(definition)))
        .collect()
}

fn is_record_open(definition: &TypeDefinition) -> bool {
    definition.is_open() || !definition.is_external()
}
