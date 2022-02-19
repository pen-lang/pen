use once_cell::sync::Lazy;

// TODO Inject this as a configuration.
pub const YIELD_FUNCTION_NAME: &str = "_pen_yield";

pub static YIELD_FUNCTION_TYPE: Lazy<fmm::types::Function> = Lazy::new(|| {
    fmm::types::Function::new(
        vec![],
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Source,
    )
});

pub fn compile_yield_function_declaration(module_builder: &fmm::build::ModuleBuilder) {
    module_builder.declare_function(YIELD_FUNCTION_NAME, YIELD_FUNCTION_TYPE.clone());
}
