use crate::{context::CompileContext, type_, CompileError};
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
        collect_types(module)
            .iter()
            .map(|type_| Ok((type_::compile(context, &type_)?, vec![])))
            .collect::<Result<_, CompileError>>()?,
    ))
}

fn collect_types(module: &Module) -> FnvHashSet<Type> {
    let position = module.position();

    [
        types::ByteString::new(position.clone()).into(),
        types::None::new(position.clone()).into(),
        types::Number::new(position.clone()).into(),
    ]
    .into_iter()
    .chain(
        type_collector::collect_records(module)
            .into_values()
            .map(Type::from),
    )
    .collect()
}
