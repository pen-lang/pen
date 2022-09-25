pub const DEBUG_FUNCTION_INDEX: usize = 0;

pub fn compile_types() -> Vec<mir::types::Type> {
    vec![mir::types::Function::new(
        vec![mir::types::Type::Variant],
        mir::types::Type::ByteString,
    )
    .into()]
}
