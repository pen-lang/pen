pub fn compare_pointers(
    lhs: impl Into<fmm::build::TypedExpression>,
    rhs: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, fmm::build::BuildError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, lhs),
        fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, rhs),
    )?
    .into())
}
