pub const DEBUG_FUNCTION_INDEX: usize = 0;

pub fn compile() -> mir::types::TypeInformation {
    mir::types::TypeInformation::new(vec![compile_debug_function_type().into()])
}

pub fn compile_debug_function_type() -> mir::types::Function {
    mir::types::Function::new(
        vec![mir::types::Type::Variant],
        mir::types::Type::ByteString,
    )
}
