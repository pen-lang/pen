use super::super::error::CompileError;
use crate::{transformation::record_type_information_compiler, type_context::TypeContext};
use hir::types::{self, Type};
use hir::{
    analysis::types::{
        type_canonicalizer, type_comparability_checker, type_equality_checker, type_resolver,
        union_type_member_calculator,
    },
    ir::*,
};
use position::Position;

const LHS_NAME: &str = "$lhs";
const RHS_NAME: &str = "$rhs";

pub fn transform(
    operation: &EqualityOperation,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    Ok(if operation.operator() == EqualityOperator::Equal {
        transform_equal_operation(
            &type_canonicalizer::canonicalize(
                operation
                    .type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(operation.position().clone()))?,
                type_context.types(),
            )?,
            operation.lhs(),
            operation.rhs(),
            operation.position(),
            type_context,
        )?
    } else {
        operation.clone().into()
    })
}

fn transform_equal_operation(
    type_: &Type,
    lhs: &Expression,
    rhs: &Expression,
    position: &Position,
    type_context: &TypeContext,
) -> Result<Expression, CompileError> {
    Ok(match type_ {
        Type::Any(_) => return Err(CompileError::AnyEqualOperation(position.clone())),
        Type::Boolean(_) => If::new(
            lhs.clone(),
            If::new(
                rhs.clone(),
                Boolean::new(true, position.clone()),
                Boolean::new(false, position.clone()),
                position.clone(),
            ),
            If::new(
                rhs.clone(),
                Boolean::new(false, position.clone()),
                Boolean::new(true, position.clone()),
                position.clone(),
            ),
            position.clone(),
        )
        .into(),
        Type::Function(_) => return Err(CompileError::FunctionEqualOperation(position.clone())),
        Type::List(list_type) => {
            let element_type = list_type.element();
            let any_list_type = types::Reference::new(
                &type_context.list_type_configuration().list_type_name,
                position.clone(),
            );

            Call::new(
                Some(
                    types::Function::new(
                        vec![
                            types::Function::new(
                                vec![
                                    types::Any::new(position.clone()).into(),
                                    types::Any::new(position.clone()).into(),
                                ],
                                types::Boolean::new(position.clone()),
                                position.clone(),
                            )
                            .into(),
                            any_list_type.clone().into(),
                            any_list_type.into(),
                        ],
                        types::Boolean::new(position.clone()),
                        position.clone(),
                    )
                    .into(),
                ),
                Variable::new(
                    &type_context.list_type_configuration().equal_function_name,
                    position.clone(),
                ),
                vec![
                    Lambda::new(
                        vec![
                            Argument::new(LHS_NAME, types::Any::new(position.clone())),
                            Argument::new(RHS_NAME, types::Any::new(position.clone())),
                        ],
                        types::Boolean::new(position.clone()),
                        IfType::new(
                            LHS_NAME,
                            Variable::new(LHS_NAME, position.clone()),
                            vec![IfTypeBranch::new(
                                element_type.clone(),
                                IfType::new(
                                    RHS_NAME,
                                    Variable::new(RHS_NAME, position.clone()),
                                    vec![IfTypeBranch::new(
                                        element_type.clone(),
                                        transform_equal_operation(
                                            element_type,
                                            &Variable::new(LHS_NAME, position.clone()).into(),
                                            &Variable::new(RHS_NAME, position.clone()).into(),
                                            position,
                                            type_context,
                                        )?,
                                    )],
                                    None,
                                    position.clone(),
                                ),
                            )],
                            None,
                            position.clone(),
                        ),
                        position.clone(),
                    )
                    .into(),
                    lhs.clone(),
                    rhs.clone(),
                ],
                position.clone(),
            )
            .into()
        }
        Type::None(_) => Boolean::new(true, position.clone()).into(),
        Type::Number(_) => EqualityOperation::new(
            Some(type_.clone()),
            EqualityOperator::Equal,
            lhs.clone(),
            rhs.clone(),
            position.clone(),
        )
        .into(),
        Type::Record(record_type) => {
            if !type_comparability_checker::check(
                type_,
                type_context.types(),
                type_context.records(),
            )? {
                return Err(CompileError::InvalidRecordEqualOperation(position.clone()));
            }

            Call::new(
                Some(
                    types::Function::new(
                        vec![record_type.clone().into(), record_type.clone().into()],
                        types::Boolean::new(position.clone()),
                        position.clone(),
                    )
                    .into(),
                ),
                Variable::new(
                    record_type_information_compiler::compile_equal_function_name(record_type),
                    position.clone(),
                ),
                vec![lhs.clone(), rhs.clone()],
                position.clone(),
            )
            .into()
        }
        Type::String(_) => EqualityOperation::new(
            Some(type_.clone()),
            EqualityOperator::Equal,
            lhs.clone(),
            rhs.clone(),
            position.clone(),
        )
        .into(),
        Type::Union(_) => {
            let member_types =
                union_type_member_calculator::calculate(type_, type_context.types())?;

            IfType::new(
                LHS_NAME,
                lhs.clone(),
                member_types
                    .iter()
                    .map(|lhs_type| {
                        Ok(IfTypeBranch::new(
                            lhs_type.clone(),
                            IfType::new(
                                RHS_NAME,
                                rhs.clone(),
                                member_types
                                    .iter()
                                    .map(|rhs_type| {
                                        Ok(IfTypeBranch::new(
                                            rhs_type.clone(),
                                            if type_equality_checker::check(
                                                lhs_type,
                                                rhs_type,
                                                type_context.types(),
                                            )? {
                                                transform_equal_operation(
                                                    rhs_type,
                                                    &Variable::new(LHS_NAME, position.clone())
                                                        .into(),
                                                    &Variable::new(RHS_NAME, position.clone())
                                                        .into(),
                                                    position,
                                                    type_context,
                                                )?
                                            } else {
                                                Boolean::new(false, position.clone()).into()
                                            },
                                        ))
                                    })
                                    .collect::<Result<_, CompileError>>()?,
                                None,
                                position.clone(),
                            ),
                        ))
                    })
                    .collect::<Result<_, CompileError>>()?,
                None,
                position.clone(),
            )
            .into()
        }
        Type::Reference(reference) => transform_equal_operation(
            &type_resolver::resolve(reference, type_context.types())?,
            lhs,
            rhs,
            position,
            type_context,
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test;
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_with_union() {
        let union_type = types::Union::new(
            types::Number::new(test::position()),
            types::None::new(test::position()),
            test::position(),
        );

        assert_eq!(
            transform(
                &EqualityOperation::new(
                    Some(union_type.into()),
                    EqualityOperator::Equal,
                    Variable::new("x", test::position()),
                    Variable::new("y", test::position()),
                    test::position()
                ),
                &TypeContext::dummy(Default::default(), Default::default())
            ),
            Ok(IfType::new(
                LHS_NAME,
                Variable::new("x", test::position()),
                vec![
                    IfTypeBranch::new(
                        types::None::new(test::position()),
                        IfType::new(
                            RHS_NAME,
                            Variable::new("y", test::position()),
                            vec![
                                IfTypeBranch::new(
                                    types::None::new(test::position()),
                                    Boolean::new(true, test::position()),
                                ),
                                IfTypeBranch::new(
                                    types::Number::new(test::position()),
                                    Boolean::new(false, test::position()),
                                ),
                            ],
                            None,
                            test::position(),
                        ),
                    ),
                    IfTypeBranch::new(
                        types::Number::new(test::position()),
                        IfType::new(
                            RHS_NAME,
                            Variable::new("y", test::position()),
                            vec![
                                IfTypeBranch::new(
                                    types::None::new(test::position()),
                                    Boolean::new(false, test::position()),
                                ),
                                IfTypeBranch::new(
                                    types::Number::new(test::position()),
                                    EqualityOperation::new(
                                        Some(types::Number::new(test::position()).into()),
                                        EqualityOperator::Equal,
                                        Variable::new(LHS_NAME, test::position()),
                                        Variable::new(RHS_NAME, test::position()),
                                        test::position(),
                                    ),
                                ),
                            ],
                            None,
                            test::position(),
                        ),
                    ),
                ],
                None,
                test::position(),
            )
            .into())
        );
    }

    #[test]
    fn transform_with_record() {
        let record_type = types::Record::new("foo", test::position());

        assert_eq!(
            transform(
                &EqualityOperation::new(
                    Some(record_type.clone().into()),
                    EqualityOperator::Equal,
                    Variable::new("x", test::position()),
                    Variable::new("y", test::position()),
                    test::position()
                ),
                &TypeContext::dummy(
                    Default::default(),
                    vec![(
                        "foo".into(),
                        vec![types::RecordElement::new(
                            "x",
                            types::None::new(test::position())
                        )]
                    )]
                    .into_iter()
                    .collect()
                )
            ),
            Ok(Call::new(
                Some(
                    types::Function::new(
                        vec![record_type.clone().into(), record_type.clone().into()],
                        types::Boolean::new(test::position()),
                        test::position(),
                    )
                    .into(),
                ),
                Variable::new(
                    record_type_information_compiler::compile_equal_function_name(&record_type),
                    test::position(),
                ),
                vec![
                    Variable::new("x", test::position()).into(),
                    Variable::new("y", test::position()).into(),
                ],
                test::position()
            )
            .into())
        );
    }

    #[test]
    fn fail_to_transform_with_any() {
        assert_eq!(
            transform(
                &EqualityOperation::new(
                    Some(types::Any::new(test::position()).into()),
                    EqualityOperator::Equal,
                    Variable::new("x", test::position()),
                    Variable::new("y", test::position()),
                    test::position()
                ),
                &TypeContext::dummy(Default::default(), Default::default())
            ),
            Err(CompileError::AnyEqualOperation(test::position()))
        );
    }

    #[test]
    fn fail_to_transform_with_function() {
        assert_eq!(
            transform(
                &EqualityOperation::new(
                    Some(
                        types::Function::new(
                            vec![],
                            types::None::new(test::position()),
                            test::position()
                        )
                        .into()
                    ),
                    EqualityOperator::Equal,
                    Variable::new("x", test::position()),
                    Variable::new("y", test::position()),
                    test::position()
                ),
                &TypeContext::dummy(Default::default(), Default::default())
            ),
            Err(CompileError::FunctionEqualOperation(test::position()))
        );
    }
}
