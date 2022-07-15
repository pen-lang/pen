use crate::{context::Context, reference_count, type_};

pub fn compile(context: &Context, declaration: &mir::ir::FunctionDeclaration) {
    context.module_builder().declare_variable(
        declaration.name(),
        reference_count::heap::compile_type_with_reference_count(type_::compile_unsized_closure(
            declaration.type_(),
            context.types(),
        )),
    );
}
