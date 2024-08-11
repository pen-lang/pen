use crate::context::Context;
use once_cell::unsync::Lazy;

thread_local! {
    static YIELD_FUNCTION_TYPE: LazyLock<fmm::types::Function> = LazyLock::new(|| {
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
