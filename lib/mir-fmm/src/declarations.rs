use super::types;
use crate::context::Context;

pub fn compile_declaration(context: &Context, declaration: &mir::ir::Declaration) {
    context.module_builder().declare_variable(
        declaration.name(),
        types::compile_unsized_closure(declaration.type_(), context.types()),
    );
}
