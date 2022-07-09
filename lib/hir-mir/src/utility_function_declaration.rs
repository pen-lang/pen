use crate::{type_, CompileConfiguration};

pub const LOCAL_DEBUG_FUNCTION_NAME: &str = "__debug";
pub const LOCAL_SPAWN_FUNCTION_NAME: &str = "__spawn";

pub fn compile(configuration: &CompileConfiguration) -> Vec<mir::ir::ForeignDeclaration> {
    vec![
        mir::ir::ForeignDeclaration::new(
            LOCAL_DEBUG_FUNCTION_NAME,
            &configuration.debug_function_name,
            mir::types::Function::new(vec![mir::types::Type::ByteString], mir::types::Type::None),
            mir::ir::CallingConvention::Target,
        ),
        mir::ir::ForeignDeclaration::new(
            LOCAL_SPAWN_FUNCTION_NAME,
            &configuration.spawn_function_name,
            type_::compile_spawn_function(),
            mir::ir::CallingConvention::Target,
        ),
    ]
}
