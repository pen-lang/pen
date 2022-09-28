use crate::{context::Context, reference_count, type_};

pub fn compile(context: &Context, declaration: &mir::ir::FunctionDeclaration) {
    context.module_builder().declare_variable(
        declaration.name(),
        reference_count::block::compile_type(type_::compile_unsized_closure(
            context,
            declaration.type_(),
        )),
    );
}
