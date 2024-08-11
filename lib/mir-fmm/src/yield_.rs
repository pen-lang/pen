use crate::context::Context;
use std::cell::LazyCell;

thread_local! {
    static YIELD_FUNCTION_TYPE: LazyCell<fmm::types::Function> = LazyCell::new(|| {
        fmm::types::Function::new(
            vec![],
            fmm::types::void_type(),
            fmm::types::CallingConvention::Source,
        )
    });
}

pub fn yield_function_type() -> fmm::types::Function {
    YIELD_FUNCTION_TYPE.with(|function| (**function).clone())
}

pub fn compile_function_declaration(context: &Context) {
    context.module_builder().declare_function(
        &context.configuration().yield_function_name,
        yield_function_type(),
    );
}
