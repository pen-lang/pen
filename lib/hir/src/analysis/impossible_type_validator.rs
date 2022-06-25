use super::AnalysisContext;
use crate::{analysis::AnalysisError, ir::*};

pub fn validate(_context: &AnalysisContext, module: &Module) -> Result<(), AnalysisError> {
    for definition in module.type_definitions() {
        validate_type_definition(definition)?;
    }

    Ok(())
}

fn validate_type_definition(_definition: &TypeDefinition) -> Result<(), AnalysisError> {
    Ok(())
}
