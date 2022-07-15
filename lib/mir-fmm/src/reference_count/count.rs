use crate::CompileError;

const UNIQUE_COUNT: isize = 0;
const SYNCHRONIZED_UNIQUE_COUNT: isize = -1;

pub const fn compile(count: isize) -> fmm::ir::Primitive {
    fmm::ir::Primitive::PointerInteger(count as i64)
}

pub const fn compile_unique() -> fmm::ir::Primitive {
    compile(UNIQUE_COUNT)
}

const fn compile_synchronized_unique() -> fmm::ir::Primitive {
    compile(SYNCHRONIZED_UNIQUE_COUNT)
}

pub const fn compile_type() -> fmm::types::Primitive {
    fmm::types::Primitive::PointerInteger
}

pub fn synchronize(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::arithmetic_operation(
        fmm::ir::ArithmeticOperator::Subtract,
        compile_synchronized_unique(),
        count.clone(),
    )?
    .into())
}

pub fn is_synchronized(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::LessThan(true),
        count.clone(),
        compile_unique(),
    )?
    .into())
}

pub fn is_unique(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        count.clone(),
        compile_unique(),
    )?
    .into())
}

pub fn is_synchronized_unique(
    count: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        count.clone(),
        compile_synchronized_unique(),
    )?
    .into())
}

pub fn static_count() -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::bitwise_operation(
        fmm::ir::BitwiseOperator::LeftShift,
        fmm::ir::Primitive::PointerInteger(1),
        fmm::build::arithmetic_operation(
            fmm::ir::ArithmeticOperator::Subtract,
            fmm::build::arithmetic_operation(
                fmm::ir::ArithmeticOperator::Multiply,
                fmm::build::size_of(compile_type()),
                fmm::ir::Primitive::PointerInteger(8),
            )?,
            fmm::ir::Primitive::PointerInteger(1),
        )?,
    )?
    .into())
}
