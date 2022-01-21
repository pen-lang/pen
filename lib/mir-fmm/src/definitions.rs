use super::error::CompileError;
use crate::{closures, entry_functions, types};
use fnv::{FnvHashMap, FnvHashSet};

pub fn compile_definition(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    global_variables: &FnvHashMap<String, fmm::build::TypedExpression>,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    module_builder.define_variable(
        definition.name(),
        closures::compile_closure_content(
            entry_functions::compile(module_builder, definition, true, global_variables, types)?,
            closures::compile_drop_function(module_builder, definition, types)?,
            fmm::ir::Undefined::new(types::compile_closure_payload(definition, types)),
        ),
        definition.is_thunk(),
        fmm::ir::Linkage::External,
        None,
    );

    Ok(())
}
