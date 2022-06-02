const INITIAL_COUNT: isize = 0;

pub const fn compile(count: isize) -> fmm::ir::Primitive {
    fmm::ir::Primitive::Integer32(count as u32)
}

pub const fn compile_initial() -> fmm::ir::Primitive {
    compile(INITIAL_COUNT)
}

pub const fn compile_type() -> fmm::types::Primitive {
    fmm::types::Primitive::Integer32
}
