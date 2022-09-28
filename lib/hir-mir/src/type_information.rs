use crate::{context::CompileContext, type_, CompileError};
use hir::{analysis::type_collector, ir::*};

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
        type_collector::collect(module)
            .values()
            .map(|type_| Ok((type_::compile(context, &type_)?, vec![])))
            .collect::<Result<_, CompileError>>()?,
    ))
}
