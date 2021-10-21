use super::types;
use std::collections::BTreeMap;

pub fn compile_declaration(
    module_builder: &fmm::build::ModuleBuilder,
    declaration: &mir::ir::Declaration,
    types: &BTreeMap<String, mir::types::RecordBody>,
) {
    module_builder.declare_variable(
        declaration.name(),
        types::compile_unsized_closure(declaration.type_(), types),
    );
}
