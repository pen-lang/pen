use super::{context::CompileContext, CompileError};
use fnv::{FnvHashMap, FnvHashSet};
use hir::{
    analysis::types::{type_canonicalizer, type_id_calculator},
    types::{self, Type},
};

pub fn compile(type_: &Type, context: &CompileContext) -> Result<mir::types::Type, CompileError> {
    Ok(
        match type_canonicalizer::canonicalize(type_, context.types())? {
            Type::Boolean(_) => mir::types::Type::Boolean,
            Type::Function(function) => compile_function(&function, context)?.into(),
            Type::List(_) => compile_list(context)?.into(),
            Type::None(_) => mir::types::Type::None,
            Type::Number(_) => mir::types::Type::Number,
            Type::Record(record) => mir::types::Record::new(record.name()).into(),
            Type::String(_) => mir::types::Type::ByteString,
            Type::Any(_) | Type::Union(_) => mir::types::Type::Variant,
            Type::Reference(_) => unreachable!(),
        },
    )
}

pub fn compile_function(
    function: &types::Function,
    context: &CompileContext,
) -> Result<mir::types::Function, CompileError> {
    let compile = |type_| compile(type_, context);

    Ok(mir::types::Function::new(
        function
            .arguments()
            .iter()
            .map(compile)
            .collect::<Result<_, _>>()?,
        compile(function.result())?,
    ))
}

pub fn compile_concrete_function(
    function: &types::Function,
    types: &FnvHashMap<String, Type>,
) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(compile_concrete_function_name(
        function, types,
    )?))
}

pub fn compile_concrete_function_name(
    function: &types::Function,
    types: &FnvHashMap<String, Type>,
) -> Result<String, CompileError> {
    Ok(format!(
        "_function_{}",
        type_id_calculator::calculate(&function.clone().into(), types)?,
    ))
}

pub fn compile_list(context: &CompileContext) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(
        &context.configuration()?.list_type.list_type_name,
    ))
}

pub fn compile_concrete_list(
    list: &types::List,
    types: &FnvHashMap<String, Type>,
) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(compile_concrete_list_name(
        list, types,
    )?))
}

pub fn compile_concrete_list_name(
    list: &types::List,
    types: &FnvHashMap<String, Type>,
) -> Result<String, CompileError> {
    Ok(format!(
        "_list_{}",
        type_id_calculator::calculate(list.element(), types)?
    ))
}

pub fn compile_spawn_function() -> mir::types::Function {
    let thunk_type = mir::types::Function::new(vec![], mir::types::Type::Variant);

    mir::types::Function::new(vec![thunk_type.clone().into()], thunk_type)
}
