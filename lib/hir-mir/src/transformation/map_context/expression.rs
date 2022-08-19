use super::context_function_name;
use crate::transformation::collection_type;
use crate::{context::CompileContext, CompileError};
use hir::{ir::*, types};

pub fn transform(context: &CompileContext, type_: &types::Map) -> Result<Expression, CompileError> {
    let position = type_.position();

    Ok(Call::new(
        Some(
            types::Function::new(
                vec![],
                collection_type::transform_map_context(context, position)?,
                position.clone(),
            )
            .into(),
        ),
        Variable::new(
            context_function_name(type_, context.types())?,
            position.clone(),
        ),
        vec![],
        position.clone(),
    )
    .into())
}
