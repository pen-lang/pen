use crate::{concurrency::MODULE_LOCAL_SPAWN_FUNCTION_NAME, type_};

pub fn compile(name: &str) -> mir::ir::ForeignDeclaration {
    mir::ir::ForeignDeclaration::new(
        MODULE_LOCAL_SPAWN_FUNCTION_NAME,
        name,
        type_::compile_spawn_function(),
        mir::ir::CallingConvention::Target,
    )
}
