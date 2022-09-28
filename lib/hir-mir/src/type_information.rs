use fnv::FnvHashMap;

pub const DEBUG_FUNCTION_INDEX: usize = 0;

pub fn compile(
    information: FnvHashMap<mir::types::Type, Vec<String>>,
) -> mir::ir::TypeInformation {
    mir::ir::TypeInformation::new(vec![compile_debug_function_type().into()], information)
}

pub fn compile_debug_function_type() -> mir::types::Function {
    mir::types::Function::new(
        vec![mir::types::Type::Variant],
        mir::types::Type::ByteString,
    )
}
