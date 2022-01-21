use super::types;
use fnv::{FnvHashMap, FnvHashSet};

pub fn compile_declaration(
    module_builder: &fmm::build::ModuleBuilder,
    declaration: &mir::ir::Declaration,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) {
    module_builder.declare_variable(
        declaration.name(),
        types::compile_unsized_closure(declaration.type_(), types),
    );
}
