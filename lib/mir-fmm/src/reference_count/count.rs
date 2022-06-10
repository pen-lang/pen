use crate::CompileError;

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

pub fn is_synchronized(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::LessThan,
        count.clone(),
        compile_initial(),
    )?
    .into())
}

pub fn is_initial(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        count.clone(),
        compile_initial(),
    )?
    .into())
}
