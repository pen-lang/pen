use super::{CompileError, context::Context};
use crate::error_type;
use fnv::FnvHashMap;
use hir::{
    analysis::{type_canonicalizer, type_id_calculator},
    types::{self, Type},
};

pub fn compile(context: &Context, type_: &Type) -> Result<mir::types::Type, CompileError> {
    Ok(
        match type_canonicalizer::canonicalize(type_, context.types())? {
            Type::Boolean(_) => mir::types::Type::Boolean,
            Type::Error(_) => compile_error().into(),
            Type::Function(function) => compile_function(context, &function)?.into(),
            Type::List(_) => compile_list(context)?.into(),
            Type::Map(_) => compile_map(context)?.into(),
            Type::None(_) => mir::types::Type::None,
            Type::Number(_) => mir::types::Type::Number,
            Type::Record(record) => compile_record(&record).into(),
            Type::String(_) => mir::types::Type::ByteString,
            Type::Any(_) | Type::Union(_) => mir::types::Type::Variant,
            Type::Reference(_) => unreachable!(),
        },
    )
}

pub fn compile_error() -> mir::types::Record {
    error_type::compile_type()
}

pub fn compile_concrete(context: &Context, type_: &Type) -> Result<mir::types::Type, CompileError> {
    Ok(
        match &type_canonicalizer::canonicalize(type_, context.types())? {
            Type::Function(type_) => compile_concrete_function(type_, context.types())?.into(),
            Type::List(type_) => compile_concrete_list(type_, context.types())?.into(),
            Type::Map(type_) => compile_concrete_map(type_, context.types())?.into(),
            Type::Boolean(_)
            | Type::Error(_)
            | Type::None(_)
            | Type::Number(_)
            | Type::Record(_)
            | Type::String(_) => compile(context, type_)?,
            Type::Any(_) | Type::Reference(_) | Type::Union(_) => unreachable!(),
        },
    )
}

pub fn compile_function(
    context: &Context,
    function: &types::Function,
) -> Result<mir::types::Function, CompileError> {
    let compile = |type_| compile(context, type_);

    Ok(mir::types::Function::new(
        function
            .arguments()
            .iter()
            .map(compile)
            .collect::<Result<_, _>>()?,
        compile(function.result())?,
    ))
}

pub fn compile_record(record: &types::Record) -> mir::types::Record {
    mir::types::Record::new(record.name())
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
        "hir:function:{}",
        type_id_calculator::calculate(&function.clone().into(), types)?,
    ))
}

pub fn compile_list(context: &Context) -> Result<mir::types::Record, CompileError> {
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
        "hir:list:{}",
        type_id_calculator::calculate(list.element(), types)?
    ))
}

pub fn compile_map(context: &Context) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(
        &context.configuration()?.map_type.map_type_name,
    ))
}

pub fn compile_map_context(context: &Context) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(
        &context.configuration()?.map_type.context_type_name,
    ))
}

pub fn compile_concrete_map(
    map: &types::Map,
    types: &FnvHashMap<String, Type>,
) -> Result<mir::types::Record, CompileError> {
    Ok(mir::types::Record::new(compile_concrete_map_name(
        map, types,
    )?))
}

pub fn compile_concrete_map_name(
    map: &types::Map,
    types: &FnvHashMap<String, Type>,
) -> Result<String, CompileError> {
    Ok(format!(
        "hir:map:{}:{}",
        type_id_calculator::calculate(map.key(), types)?,
        type_id_calculator::calculate(map.value(), types)?,
    ))
}

pub fn compile_race_function(context: &Context) -> Result<mir::types::Function, CompileError> {
    let list_type = compile_list(context)?;

    Ok(mir::types::Function::new(
        vec![list_type.clone().into()],
        list_type,
    ))
}

pub fn compile_spawn_function() -> mir::types::Function {
    let thunk_type = mir::types::Function::new(vec![], mir::types::Type::Variant);

    mir::types::Function::new(vec![thunk_type.clone().into()], thunk_type)
}
