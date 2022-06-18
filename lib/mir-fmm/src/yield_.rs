use crate::context::Context;
use once_cell::sync::Lazy;

static YIELD_FUNCTION_TYPE: Lazy<fmm::types::Function> = Lazy::new(|| {
    fmm::types::Function::new(
        vec![],
        fmm::types::void_type(),
        fmm::types::CallingConvention::Source,
    )
});

pub fn yield_function_type() -> fmm::types::Function {
    YIELD_FUNCTION_TYPE.clone()
}

pub fn compile_yield_function_declaration(context: &Context) {
    context.module_builder().declare_function(
        &context.configuration().yield_function_name,
        yield_function_type(),
    );
}
