use super::{super::error::CompileError, collection_type, map_context};
use crate::{context::Context, downcast};
use hir::{
    analysis::AnalysisError,
    ir::*,
    types::{self, Type},
};

pub fn transform(context: &Context, if_: &IfMap) -> Result<Expression, CompileError> {
    let configuration = &context.configuration()?.map_type;
    let position = if_.position();

    let key_type = if_
        .key_type()
        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
    let value_type = if_
        .value_type()
        .ok_or_else(|| AnalysisError::TypeNotInferred(position.clone()))?;
    let any_type = Type::from(types::Any::new(position.clone()));

    Ok(IfType::new(
        if_.name(),
        Call::new(
            Some(
                types::Function::new(
                    vec![
                        collection_type::transform_map_context(context, position)?,
                        collection_type::transform_map(context, position)?,
                        any_type.clone(),
                    ],
                    any_type.clone(),
                    position.clone(),
                )
                .into(),
            ),
            Variable::new(&configuration.get_function_name, position.clone()),
            vec![
                map_context::expression::transform(
                    context,
                    &types::Map::new(key_type.clone(), value_type.clone(), position.clone()),
                )?,
                if_.map().clone(),
                TypeCoercion::new(
                    key_type.clone(),
                    any_type.clone(),
                    if_.key().clone(),
                    position.clone(),
                )
                .into(),
            ],
            position.clone(),
        ),
        vec![IfTypeBranch::new(
            types::Reference::new(&configuration.empty_type_name, position.clone()),
            if_.else_().clone(),
        )],
        Some(ElseBranch::new(
            Some(types::Any::new(position.clone()).into()),
            Let::new(
                Some(if_.name().into()),
                Some(value_type.clone()),
                downcast::compile(
                    context,
                    &any_type,
                    value_type,
                    &Variable::new(if_.name(), position.clone()).into(),
                )?,
                if_.then().clone(),
                position.clone(),
            ),
            position.clone(),
        )),
        position.clone(),
    )
    .into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use position::{Position, test::PositionFake};

    #[test]
    fn transform_if_map() {
        insta::assert_debug_snapshot!(transform(
            &Context::dummy(Default::default(), Default::default()),
            &IfMap::new(
                Some(types::Number::new(Position::fake()).into()),
                Some(types::None::new(Position::fake()).into()),
                "x",
                Variable::new("xs", Position::fake()),
                Variable::new("k", Position::fake()),
                Variable::new("x", Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
        ));
    }
}
