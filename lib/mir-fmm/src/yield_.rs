use crate::context::Context;
use once_cell::sync::Lazy;

pub static YIELD_FUNCTION_TYPE: Lazy<fmm::types::Function> = Lazy::new(|| {
    fmm::types::Function::new(
        vec![],
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Source,
    )
});

pub fn compile_yield_function_declaration(context: &Context) {
    context.module_builder().declare_function(
        &context.configuration().yield_function_name,
        YIELD_FUNCTION_TYPE.clone(),
    );
}
