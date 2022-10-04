use crate::type_information;

const DEBUG_FUNCTION_NAME: &str = "_builtin_debug";
const ARGUMENT_NAME: &str = "$x";

pub fn compile_debug_function_definition() -> mir::ir::FunctionDefinition {
    mir::ir::FunctionDefinition::new(
        DEBUG_FUNCTION_NAME,
        vec![mir::ir::Argument::new(
            ARGUMENT_NAME,
            mir::types::Type::Variant,
        )],
        mir::types::Type::ByteString,
        type_information::debug::compile_call(mir::ir::Variable::new(ARGUMENT_NAME)),
    )
}
