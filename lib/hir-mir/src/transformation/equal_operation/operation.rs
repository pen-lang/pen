use super::function;
use crate::{
    context::Context,
    error::CompileError,
    transformation::{collection_type, map_context, record_type_information},
};
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
    context: &Context,
    type_: &Type,
    lhs: &Expression,
    rhs: &Expression,
    position: &Position,
) -> Result<Expression, CompileError> {
    transform_canonical(
        context,
        &type_canonicalizer::canonicalize(type_, context.types())?,
        lhs,
        rhs,
        position,
    )
}

fn transform_canonical(
    context: &Context,
    type_: &Type,
    lhs: &Expression,
    rhs: &Expression,
    position: &Position,
) -> Result<Expression, CompileError> {
    Ok(match type_ {
        Type::Boolean(_) => If::new(
            lhs.clone(),
            rhs.clone(),
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
                    function::transform(context, list_type.element())?,
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
                    map_context::expression::transform(context, map_type)?,
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
                                    transform_canonical(
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
        Type::Reference(reference) => transform_canonical(
            context,
            &type_resolver::resolve(reference, context.types())?,
            lhs,
            rhs,
            position,
        )?,
        Type::Any(_) | Type::Error(_) | Type::Function(_) => {
            return Err(AnalysisError::TypeNotComparable(position.clone(), type_.clone()).into())
        }
    })
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
    use crate::transformation::record_type_information;
    use hir::{test::RecordFake, types};
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
                &Context::dummy(Default::default(), Default::default()),
                &union_type.into(),
                &Variable::new("x", Position::fake()).into(),
                &Variable::new("y", Position::fake()).into(),
                &Position::fake()
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
        let record_type = types::Record::fake("foo");

        assert_eq!(
            transform(
                &Context::dummy(
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
                &record_type.clone().into(),
                &Variable::new("x", Position::fake()).into(),
                &Variable::new("y", Position::fake()).into(),
                &Position::fake()
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
                &Context::dummy(Default::default(), Default::default()),
                &types::Any::new(Position::fake()).into(),
                &Variable::new("x", Position::fake()).into(),
                &Variable::new("y", Position::fake()).into(),
                &Position::fake()
            ),
            Err(AnalysisError::TypeNotComparable(
                Position::fake(),
                types::Any::new(Position::fake()).into()
            )
            .into())
        );
    }

    #[test]
    fn fail_to_transform_with_function() {
        let function_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            transform(
                &Context::dummy(Default::default(), Default::default()),
                &function_type.clone().into(),
                &Variable::new("x", Position::fake()).into(),
                &Variable::new("y", Position::fake()).into(),
                &Position::fake()
            ),
            Err(AnalysisError::TypeNotComparable(Position::fake(), function_type.into()).into())
        );
    }
}
