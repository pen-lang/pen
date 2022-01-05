use super::{type_context::TypeContext, CompileError};
use hir::{
    analysis::types::{type_canonicalizer, type_id_calculator},
    types::{self, Type},
};
use std::collections::BTreeMap;

pub fn compile(type_: &Type, type_context: &TypeContext) -> Result<mir::types::Type, CompileError> {
    Ok(
        match type_canonicalizer::canonicalize(type_, type_context.types())? {
            Type::Boolean(_) => mir::types::Type::Boolean,
            Type::Function(function) => compile_function(&function, type_context)?.into(),
            Type::List(_) => {
                mir::types::Record::new(&type_context.list_type_configuration().list_type_name)
                    .into()
            }
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
    type_context: &TypeContext,
) -> Result<mir::types::Function, CompileError> {
    let compile = |type_| compile(type_, type_context);

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
    types: &BTreeMap<String, Type>,
) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(compile_concrete_function_name(
        function, types,
    )?))
}

pub fn compile_concrete_function_name(
    function: &types::Function,
    types: &BTreeMap<String, Type>,
) -> Result<String, CompileError> {
    Ok(format!(
        "_function_{}",
        type_id_calculator::calculate(&function.clone().into(), types)?,
    ))
}

pub fn compile_concrete_list(
    list: &types::List,
    types: &BTreeMap<String, Type>,
) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(compile_concrete_list_name(
        list, types,
    )?))
}

pub fn compile_concrete_list_name(
    list: &types::List,
    types: &BTreeMap<String, Type>,
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
