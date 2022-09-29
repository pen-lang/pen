use crate::{context::CompileContext, generic_type_collection, type_, CompileError};
use fnv::FnvHashSet;
use hir::{
    analysis::type_collector,
    ir::*,
    types::{self, Type},
};
use position::Position;

pub const DEBUG_FUNCTION_INDEX: usize = 0;

pub fn compile_debug_function_type() -> mir::types::Function {
    mir::types::Function::new(
        vec![mir::types::Type::Variant],
        mir::types::Type::ByteString,
    )
}

// TODO Compile type information.
pub fn compile(
    context: &CompileContext,
    module: &Module,
) -> Result<mir::ir::TypeInformation, CompileError> {
    Ok(mir::ir::TypeInformation::new(
        vec![compile_debug_function_type().into()],
        collect_types(context, module)?
            .iter()
            .map(|type_| Ok((type_::compile(context, type_)?, vec![])))
            .collect::<Result<_, CompileError>>()?,
    ))
}

fn collect_types(
    context: &CompileContext,
    module: &Module,
) -> Result<FnvHashSet<Type>, CompileError> {
    let position = module.position();

    Ok([
        types::Boolean::new(position.clone()).into(),
        types::ByteString::new(position.clone()).into(),
        types::Error::new(position.clone()).into(),
        types::None::new(position.clone()).into(),
        types::Number::new(position.clone()).into(),
    ]
    .into_iter()
    .chain(generic_type_collection::collect(context, module)?)
    .chain(
        type_collector::collect_records(module)
            .into_values()
            .map(Type::from),
    )
    .collect())
}
