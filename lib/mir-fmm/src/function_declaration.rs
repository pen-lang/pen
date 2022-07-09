use super::type_;
use crate::context::Context;

pub fn compile(context: &Context, declaration: &mir::ir::FunctionDeclaration) {
    context.module_builder().declare_variable(
        declaration.name(),
        type_::compile_unsized_closure(declaration.type_(), context.types()),
    );
}
