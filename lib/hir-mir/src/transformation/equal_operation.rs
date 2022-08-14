use super::{super::error::CompileError, collection_type, map_context};
use crate::{context::CompileContext, transformation::record_type_information};
use hir::{
    analysis::{
        type_canonicalizer, type_comparability_checker, type_equality_checker, type_resolver,
        union_type_creator, union_type_member_calculator, AnalysisError,
    },
    ir::*,
    types::{self, Type},
};
use position::Position;

const LHS_NAME: &str = "$lhs";
const RHS_NAME: &str = "$rhs";

pub fn transform(
    context: &CompileContext,
    operation: &EqualityOperation,
) -> Result<Expression, CompileError> {
    Ok(if operation.operator() == EqualityOperator::Equal {
        transform_equal_operation(
            context,
            &type_canonicalizer::canonicalize(
                operation
                    .type_()
                    .ok_or_else(|| AnalysisError::TypeNotInferred(operation.position().clone()))?,
                context.types(),
            )?,
            operation.lhs(),
            operation.rhs(),
            operation.position(),
        )?
    } else {
        operation.clone().into()
    })
}

fn transform_equal_operation(
    context: &CompileContext,
    type_: &Type,
    lhs: &Expression,
    rhs: &Expression,
    position: &Position,
) -> Result<Expression, CompileError> {
    Ok(match type_ {
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
        Type::List(list_type) => {
            let any_list_type = collection_type::transform_list(context, position)?;

            Call::new(
                Some(
                    types::Function::new(
                        vec![
                            compile_any_function_type(position).into(),
                            any_list_type.clone(),
                            any_list_type,
                        ],
                        types::Boolean::new(position.clone()),
                        position.clone(),
                    )
                    .into(),
                ),
                Variable::new(
                    &context.configuration()?.list_type.equal_function_name,
                    position.clone(),
                ),
                vec![
                    transform_any_function(context, list_type.element(), position)?.into(),
                    lhs.clone(),
                    rhs.clone(),
                ],
                position.clone(),
            )
            .into()
        }
        Type::Map(map_type) => {
            let any_map_type = collection_type::transform_map(context, position)?;

            Call::new(
                Some(
                    types::Function::new(
                        vec![
                            collection_type::transform_map_context(context, position)?,
                            any_map_type.clone(),
                            any_map_type,
                        ],
                        types::Boolean::new(position.clone()),
                        position.clone(),
                    )
                    .into(),
                ),
                Variable::new(
                    &context.configuration()?.map_type.equal_function_name,
                    position.clone(),
                ),
                vec![
                    map_context::transform(context, map_type.key(), map_type.value(), position)?,
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
            if !type_comparability_checker::check(type_, context.types(), context.records())? {
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
                    record_type_information::compile_equal_function_name(record_type),
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
            let member_types = union_type_member_calculator::calculate(type_, context.types())?;

            IfType::new(
                LHS_NAME,
                lhs.clone(),
                member_types
                    .iter()
                    .map(|member_type| {
                        Ok(IfTypeBranch::new(
                            member_type.clone(),
                            IfType::new(
                                RHS_NAME,
                                rhs.clone(),
                                vec![IfTypeBranch::new(
                                    member_type.clone(),
                                    transform_equal_operation(
                                        context,
                                        member_type,
                                        &Variable::new(LHS_NAME, position.clone()).into(),
                                        &Variable::new(RHS_NAME, position.clone()).into(),
                                        position,
                                    )?,
                                )],
                                Some(ElseBranch::new(
                                    Some(
                                        union_type_creator::create(
                                            &member_types
                                                .iter()
                                                .cloned()
                                                .filter_map(|type_| {
                                                    type_equality_checker::check(
                                                        &type_,
                                                        member_type,
                                                        context.types(),
                                                    )
                                                    .map(|equal| (!equal).then_some(type_))
                                                    .transpose()
                                                })
                                                .collect::<Result<Vec<_>, _>>()?,
                                            position,
                                        )
                                        .unwrap(),
                                    ),
                                    Boolean::new(false, position.clone()),
                                    position.clone(),
                                )),
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
            context,
            &type_resolver::resolve(reference, context.types())?,
            lhs,
            rhs,
            position,
        )?,
        Type::Any(_) | Type::Error(_) | Type::Function(_) => {
            return Err(AnalysisError::TypeNotComparable(type_.clone()).into())
        }
    })
}

// TODO Define this as a global function.
pub fn transform_any_function(
    context: &CompileContext,
    type_: &Type,
    position: &Position,
) -> Result<Lambda, CompileError> {
    Ok(Lambda::new(
        vec![
            Argument::new(LHS_NAME, types::Any::new(position.clone())),
            Argument::new(RHS_NAME, types::Any::new(position.clone())),
        ],
        types::Boolean::new(position.clone()),
        IfType::new(
            LHS_NAME,
            Variable::new(LHS_NAME, position.clone()),
            vec![IfTypeBranch::new(
                type_.clone(),
                IfType::new(
                    RHS_NAME,
                    Variable::new(RHS_NAME, position.clone()),
                    vec![IfTypeBranch::new(
                        type_.clone(),
                        transform_equal_operation(
                            context,
                            type_,
                            &Variable::new(LHS_NAME, position.clone()).into(),
                            &Variable::new(RHS_NAME, position.clone()).into(),
                            position,
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
    ))
}

fn compile_any_function_type(position: &Position) -> types::Function {
    types::Function::new(
        vec![
            types::Any::new(position.clone()).into(),
            types::Any::new(position.clone()).into(),
        ],
        types::Boolean::new(position.clone()),
        position.clone(),
    )
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
                &CompileContext::dummy(Default::default(), Default::default()),
                &EqualityOperation::new(
                    Some(union_type.into()),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
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
                            vec![IfTypeBranch::new(
                                types::None::new(Position::fake()),
                                Boolean::new(true, Position::fake()),
                            )],
                            Some(ElseBranch::new(
                                Some(types::Number::new(Position::fake()).into()),
                                Boolean::new(false, Position::fake()),
                                Position::fake()
                            )),
                            Position::fake(),
                        ),
                    ),
                    IfTypeBranch::new(
                        types::Number::new(Position::fake()),
                        IfType::new(
                            RHS_NAME,
                            Variable::new("y", Position::fake()),
                            vec![IfTypeBranch::new(
                                types::Number::new(Position::fake()),
                                EqualityOperation::new(
                                    Some(types::Number::new(Position::fake()).into()),
                                    EqualityOperator::Equal,
                                    Variable::new(LHS_NAME, Position::fake()),
                                    Variable::new(RHS_NAME, Position::fake()),
                                    Position::fake(),
                                ),
                            )],
                            Some(ElseBranch::new(
                                Some(types::None::new(Position::fake()).into()),
                                Boolean::new(false, Position::fake()),
                                Position::fake()
                            )),
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
                &CompileContext::dummy(
                    Default::default(),
                    [(
                        "foo".into(),
                        vec![types::RecordField::new(
                            "x",
                            types::None::new(Position::fake())
                        )]
                    )]
                    .into_iter()
                    .collect()
                ),
                &EqualityOperation::new(
                    Some(record_type.clone().into()),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
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
                    record_type_information::compile_equal_function_name(&record_type),
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
                &CompileContext::dummy(Default::default(), Default::default()),
                &EqualityOperation::new(
                    Some(types::Any::new(Position::fake()).into()),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
            ),
            Err(AnalysisError::TypeNotComparable(types::Any::new(Position::fake()).into()).into())
        );
    }

    #[test]
    fn fail_to_transform_with_function() {
        let function_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            transform(
                &CompileContext::dummy(Default::default(), Default::default()),
                &EqualityOperation::new(
                    Some(function_type.clone().into()),
                    EqualityOperator::Equal,
                    Variable::new("x", Position::fake()),
                    Variable::new("y", Position::fake()),
                    Position::fake()
                ),
            ),
            Err(AnalysisError::TypeNotComparable(function_type.into()).into())
        );
    }
}
