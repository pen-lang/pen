use crate::{
    concurrency_configuration::MODULE_LOCAL_SPAWN_FUNCTION_NAME, type_, ConcurrencyConfiguration,
};

pub fn compile(
    concurrency_configuration: &ConcurrencyConfiguration,
) -> mir::ir::ForeignDeclaration {
    mir::ir::ForeignDeclaration::new(
        MODULE_LOCAL_SPAWN_FUNCTION_NAME,
        &concurrency_configuration.spawn_function_name,
        type_::compile_spawn_function(),
        mir::ir::CallingConvention::Target,
    )
}
