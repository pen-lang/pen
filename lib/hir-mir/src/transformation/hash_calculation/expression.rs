use super::function;
use crate::{
    CompileError,
    context::Context,
    transformation::{collection_type, map_context, record_type_information},
};
use hir::{
    analysis::{
        AnalysisError, type_comparability_checker, type_resolver, union_type_member_calculator,
    },
    ir::*,
    types::{self, Type},
};
use position::Position;

const NONE_HASH: f64 = 0.0;
const BOOLEAN_TRUE_HASH: f64 = 1.0;
const BOOLEAN_FALSE_HASH: f64 = 2.0;

pub fn transform(
    context: &Context,
    value: &Expression,
    type_: &Type,
    position: &Position,
) -> Result<Expression, CompileError> {
    let configuration = context.configuration()?;

    Ok(match type_ {
        Type::Boolean(_) => If::new(
            value.clone(),
            Number::new(BOOLEAN_TRUE_HASH, position.clone()),
            Number::new(BOOLEAN_FALSE_HASH, position.clone()),
            position.clone(),
        )
        .into(),
        Type::List(list_type) => Call::new(
            Some(
                types::Function::new(
                    vec![
                        compile_any_function_type(position).into(),
                        collection_type::transform_list(context, position)?,
                    ],
                    types::Number::new(position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(
                &configuration.map_type.hash.list_hash_function_name,
                position.clone(),
            ),
            vec![
                function::transform(context, list_type.element())?,
                value.clone(),
            ],
            position.clone(),
        )
        .into(),
        Type::Map(map_type) => Call::new(
            Some(
                types::Function::new(
                    vec![
                        collection_type::transform_map_context(context, position)?,
                        collection_type::transform_map(context, position)?,
                    ],
                    types::Number::new(position.clone()),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(
                &configuration.map_type.hash.map_hash_function_name,
                position.clone(),
            ),
            vec![
                map_context::expression::transform(context, map_type)?,
                value.clone(),
            ],
            position.clone(),
        )
        .into(),
        Type::None(_) => Number::new(NONE_HASH, position.clone()).into(),
        Type::Number(_) => compile_concrete_hash_function_call(
            &configuration.map_type.hash.number_hash_function_name,
            value,
            type_,
            position,
        ),
        Type::Record(record_type) => {
            if !type_comparability_checker::check(type_, context.types(), context.records())? {
                return Err(CompileError::InvalidRecordEqualOperation(position.clone()));
            }

            compile_concrete_hash_function_call(
                record_type_information::compile_hash_function_name(record_type),
                value,
                type_,
                position,
            )
        }
        Type::String(_) => compile_concrete_hash_function_call(
            &configuration.map_type.hash.string_hash_function_name,
            value,
            type_,
            position,
        ),
        Type::Union(_) => {
            const VALUE_NAME: &str = "$x";
            let member_types = union_type_member_calculator::calculate(type_, context.types())?;

            IfType::new(
                VALUE_NAME,
                value.clone(),
                member_types
                    .iter()
                    .map(|type_| {
                        Ok(IfTypeBranch::new(
                            type_.clone(),
                            transform(
                                context,
                                &Variable::new(VALUE_NAME, position.clone()).into(),
                                type_,
                                position,
                            )?,
                        ))
                    })
                    .collect::<Result<_, CompileError>>()?,
                None,
                position.clone(),
            )
            .into()
        }
        Type::Reference(reference) => transform(
            context,
            value,
            &type_resolver::resolve(reference, context.types())?,
            position,
        )?,
        Type::Any(_) | Type::Error(_) | Type::Function(_) => {
            return Err(AnalysisError::TypeNotComparable(position.clone(), type_.clone()).into());
        }
    })
}

fn compile_concrete_hash_function_call(
    name: impl Into<String>,
    value: &Expression,
    type_: &Type,
    position: &Position,
) -> Expression {
    Call::new(
        Some(
            types::Function::new(
                vec![type_.clone()],
                types::Number::new(position.clone()),
                position.clone(),
            )
            .into(),
        ),
        Variable::new(name.into(), position.clone()),
        vec![value.clone()],
        position.clone(),
    )
    .into()
}

fn compile_any_function_type(position: &Position) -> types::Function {
    types::Function::new(
        vec![types::Any::new(position.clone()).into()],
        types::Number::new(position.clone()),
        position.clone(),
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::map_type_configuration::HASH_CONFIGURATION;
    use hir::test::RecordFake;
    use position::{Position, test::PositionFake};
    use pretty_assertions::assert_eq;

    fn hash_type() -> Type {
        types::Number::new(Position::fake()).into()
    }

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
                &Variable::new("x", Position::fake()).into(),
                &union_type.into(),
                &Position::fake(),
            ),
            Ok(IfType::new(
                "$x",
                Variable::new("x", Position::fake()),
                vec![
                    IfTypeBranch::new(
                        types::None::new(Position::fake()),
                        Number::new(0.0, Position::fake()),
                    ),
                    IfTypeBranch::new(
                        types::Number::new(Position::fake()),
                        Call::new(
                            Some(
                                types::Function::new(
                                    vec![types::Number::new(Position::fake()).into()],
                                    hash_type(),
                                    Position::fake()
                                )
                                .into()
                            ),
                            Variable::new(
                                &HASH_CONFIGURATION.number_hash_function_name,
                                Position::fake()
                            ),
                            vec![Variable::new("$x", Position::fake()).into()],
                            Position::fake()
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
                &Variable::new("x", Position::fake()).into(),
                &record_type.clone().into(),
                &Position::fake(),
            ),
            Ok(Call::new(
                Some(
                    types::Function::new(
                        vec![record_type.clone().into()],
                        hash_type(),
                        Position::fake(),
                    )
                    .into(),
                ),
                Variable::new(
                    record_type_information::compile_hash_function_name(&record_type),
                    Position::fake(),
                ),
                vec![Variable::new("x", Position::fake()).into()],
                Position::fake()
            )
            .into())
        );
    }

    #[test]
    fn fail_to_transform_with_any() {
        let any_type = types::Any::new(Position::fake());

        assert_eq!(
            transform(
                &Context::dummy(Default::default(), Default::default()),
                &Variable::new("x", Position::fake()).into(),
                &any_type.clone().into(),
                &Position::fake(),
            ),
            Err(AnalysisError::TypeNotComparable(Position::fake(), any_type.into()).into())
        );
    }

    #[test]
    fn fail_to_transform_with_function() {
        let function_type =
            types::Function::new(vec![], types::None::new(Position::fake()), Position::fake());

        assert_eq!(
            transform(
                &Context::dummy(Default::default(), Default::default()),
                &Variable::new("x", Position::fake()).into(),
                &function_type.clone().into(),
                &Position::fake(),
            ),
            Err(AnalysisError::TypeNotComparable(Position::fake(), function_type.into()).into())
        );
    }
}
