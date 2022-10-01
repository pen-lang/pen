use crate::{context::Context, type_, CompileError};
use hir::{analysis::type_canonicalizer, types::Type};

pub fn compile(
    context: &Context,
    expression: mir::ir::Expression,
    type_: &Type,
) -> Result<mir::ir::Expression, CompileError> {
    Ok(
        match &type_canonicalizer::canonicalize(type_, context.types())? {
            Type::Boolean(_)
            | Type::Error(_)
            | Type::None(_)
            | Type::Number(_)
            | Type::Record(_)
            | Type::String(_) => expression,
            Type::Function(function_type) => mir::ir::Record::new(
                type_::compile_concrete_function(function_type, context.types())?,
                vec![expression],
            )
            .into(),
            Type::List(list_type) => mir::ir::Record::new(
                type_::compile_concrete_list(list_type, context.types())?,
                vec![expression],
            )
            .into(),
            Type::Map(map_type) => mir::ir::Record::new(
                type_::compile_concrete_map(map_type, context.types())?,
                vec![expression],
            )
            .into(),
            Type::Any(_) | Type::Reference(_) | Type::Union(_) => unreachable!(),
        },
    )
}

// TODO TEST
