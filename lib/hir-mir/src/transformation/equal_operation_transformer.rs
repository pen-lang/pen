use super::super::error::CompileError;
use crate::{transformation::record_type_information_compiler, compile_context::CompileContext};
use hir::{
    analysis::types::{
        type_canonicalizer, type_comparability_checker, type_equality_checker, type_resolver,
        union_type_member_calculator,
    },
    ir::*,
    types::{self, Type},
};
use position::Position;

const LHS_NAME: &str = "$lhs";
const RHS_NAME: &str = "$rhs";

pub fn transform(
    operation: &EqualityOperation,
    compile_context: &CompileContext,
) -> Result<Expression, CompileError> {
    Ok(if operation.operator() == EqualityOperator::Equal {
        transform_equal_operation(
            &type_canonicalizer::canonicalize(
                operation
                    .type_()
                    .ok_or_else(|| CompileError::TypeNotInferred(operation.position().clone()))?,
                compile_context.types(),
            )?,
            operation.lhs(),
            operation.rhs(),
            operation.position(),
            compile_context,
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
    compile_context: &CompileContext,
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
                &compile_context.list_type_configuration().list_type_name,
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
                    &compile_context.list_type_configuration().equal_function_name,
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
                                            compile_context,
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
                compile_context.types(),
                compile_context.records(),
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
                union_type_member_calculator::calculate(type_, compile_context.types())?;

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
                                                compile_context.types(),
                                            )? {
                                                transform_equal_operation(
                                                    rhs_type,
                                                    &Variable::new(LHS_NAME, position.clone())
                                                        .into(),
                                                    &Variable::new(RHS_NAME, position.clone())
                                                        .into(),
                                                    position,
                                                    compile_context,
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
            &type_resolver::resolve(reference, compile_context.types())?,
            lhs,
            rhs,
            position,
            compile_context,
        )?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{test::PositionFake, Position};
    use pretty_assertions::assert_eq;

    #[test]
    fn transform_with_union() {
        let union_type = types::Union::new(
            types::Number::new(Position::fake()),
            types::None::new(Position::fake()),
            Position::fake(),
        );

        assert_eq!(
            transform(
                &EqualityOperation::new(
                    Some(union_type.into()),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
                &CompileContext::dummy(Default::default(), Default::default())
            ),
            Ok(IfType::new(
                LHS_NAME,
                Variable::new("x", Position::fake()),
                vec![
                    IfTypeBranch::new(
                        types::None::new(Position::fake()),
                        IfType::new(
                            RHS_NAME,
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::None::new(Position::fake()),
                                    Boolean::new(true, Position::fake()),
                                ),
                                IfTypeBranch::new(
                                    types::Number::new(Position::fake()),
                                    Boolean::new(false, Position::fake()),
                                ),
                            ],
                            None,
                            Position::fake(),
                        ),
                    ),
                    IfTypeBranch::new(
                        types::Number::new(Position::fake()),
                        IfType::new(
                            RHS_NAME,
                            Variable::new("y", Position::fake()),
                            vec![
                                IfTypeBranch::new(
                                    types::None::new(Position::fake()),
                                    Boolean::new(false, Position::fake()),
                                ),
                                IfTypeBranch::new(
                                    types::Number::new(Position::fake()),
                                    EqualityOperation::new(
                                        Some(types::Number::new(Position::fake()).into()),
                                        EqualityOperator::Equal,
                                        Variable::new(LHS_NAME, Position::fake()),
                                        Variable::new(RHS_NAME, Position::fake()),
                                        Position::fake(),
                                    ),
                                ),
                            ],
                            None,
                            Position::fake(),
                        ),
                    ),
                ],
                None,
                Position::fake(),
            )
            .into())
        );
    }

    #[test]
    fn transform_with_record() {
        let record_type = types::Record::new("foo", Position::fake());

        assert_eq!(
            transform(
                &EqualityOperation::new(
                    Some(record_type.clone().into()),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
                &CompileContext::dummy(
                    Default::default(),
                    vec![(
                        "foo".into(),
                        vec![types::RecordField::new(
                            "x",
                            types::None::new(Position::fake())
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
                        types::Boolean::new(Position::fake()),
                        Position::fake(),
                    )
                    .into(),
                ),
                Variable::new(
                    record_type_information_compiler::compile_equal_function_name(&record_type),
                    Position::fake(),
                ),
                vec![
                    Variable::new("x", Position::fake()).into(),
                    Variable::new("y", Position::fake()).into(),
                ],
                Position::fake()
            )
            .into())
        );
    }

    #[test]
    fn fail_to_transform_with_any() {
        assert_eq!(
            transform(
                &EqualityOperation::new(
                    Some(types::Any::new(Position::fake()).into()),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
                &CompileContext::dummy(Default::default(), Default::default())
            ),
            Err(CompileError::AnyEqualOperation(Position::fake()))
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
                            types::None::new(Position::fake()),
                            Position::fake()
                        )
                        .into()
                    ),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
                &CompileContext::dummy(Default::default(), Default::default())
            ),
            Err(CompileError::FunctionEqualOperation(Position::fake()))
        );
    }
}
