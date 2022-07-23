use super::count;
use crate::CompileError;

pub fn compile_static(
    expression: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record(vec![count::compile_static()?, expression.into()]).into())
}

pub fn compile_type(type_: impl Into<fmm::types::Type>) -> fmm::types::Record {
    fmm::types::Record::new(vec![count::compile_type().into(), type_.into()])
}

pub fn compile_count_pointer(
    block_pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::pointer_address(
        fmm::build::bit_cast(
            fmm::types::Pointer::new(count::compile_type()),
            block_pointer.clone(),
        ),
        fmm::ir::Primitive::PointerInteger(-1),
    )?
    .into())
}
