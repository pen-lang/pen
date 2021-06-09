use std::collections::HashMap;

pub const FUNCTION_ARGUMENT_OFFSET: usize = 1;

pub fn get_arity(type_: &fmm::types::Function) -> usize {
    type_.arguments().len() - FUNCTION_ARGUMENT_OFFSET
}

pub fn compile(
    type_: &mir::types::Type,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Type {
    match type_ {
        mir::types::Type::Boolean => fmm::types::Primitive::Boolean.into(),
        mir::types::Type::Function(function) => {
            fmm::types::Pointer::new(compile_unsized_closure(function, types)).into()
        }
        mir::types::Type::Number => fmm::types::Primitive::Float64.into(),
        mir::types::Type::Record(record) => compile_record(record, types),
        mir::types::Type::ByteString => compile_string().into(),
        mir::types::Type::Variant => compile_variant().into(),
    }
}

pub fn compile_string() -> fmm::types::Pointer {
    fmm::types::Pointer::new(fmm::types::Record::new(vec![
        fmm::types::Primitive::PointerInteger.into(),
        // The first byte of a string
        fmm::types::Primitive::Integer8.into(),
    ]))
}

pub fn compile_variant() -> fmm::types::Record {
    fmm::types::Record::new(vec![
        compile_variant_tag().into(),
        compile_variant_payload().into(),
    ])
}

pub fn compile_variant_tag() -> fmm::types::Pointer {
    fmm::types::Pointer::new(fmm::types::Record::new(vec![
        // clone function
        fmm::types::Function::new(
            vec![compile_variant_payload().into()],
            fmm::types::VOID_TYPE.clone(),
            fmm::types::CallingConvention::Target,
        )
        .into(),
        // drop function
        fmm::types::Function::new(
            vec![compile_variant_payload().into()],
            fmm::types::VOID_TYPE.clone(),
            fmm::types::CallingConvention::Target,
        )
        .into(),
    ]))
}

pub fn compile_variant_payload() -> fmm::types::Primitive {
    fmm::types::Primitive::Integer64
}

pub fn compile_type_id(type_: &mir::types::Type) -> String {
    format!("{:?}", type_)
}

pub fn compile_record(
    record: &mir::types::Record,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Type {
    if is_record_boxed(record, types) {
        fmm::types::Pointer::new(fmm::types::Record::new(vec![])).into()
    } else {
        compile_unboxed_record(record, types).into()
    }
}

// TODO Unbox small non-recursive records.
pub fn is_record_boxed(
    record: &mir::types::Record,
    types: &HashMap<String, mir::types::RecordBody>,
) -> bool {
    !types[record.name()].elements().is_empty()
}

pub fn compile_unboxed_record(
    record: &mir::types::Record,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Record {
    fmm::types::Record::new(
        types[record.name()]
            .elements()
            .iter()
            .map(|type_| compile(type_, types))
            .collect(),
    )
}

pub fn compile_sized_closure(
    definition: &mir::ir::Definition,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Record {
    compile_raw_closure(
        compile_entry_function(definition.type_(), types),
        compile_closure_payload(definition, types),
    )
}

pub fn compile_closure_payload(
    definition: &mir::ir::Definition,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Type {
    if definition.is_thunk() {
        fmm::types::Union::new(vec![
            compile_environment(definition, types).into(),
            compile(definition.result_type(), types),
        ])
        .into()
    } else {
        compile_environment(definition, types).into()
    }
}

pub fn compile_unsized_closure(
    function: &mir::types::Function,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Record {
    compile_raw_closure(
        compile_entry_function(function, types),
        compile_unsized_environment(),
    )
}

pub fn compile_raw_closure(
    entry_function: fmm::types::Function,
    environment: impl Into<fmm::types::Type>,
) -> fmm::types::Record {
    fmm::types::Record::new(vec![
        entry_function.into(),
        compile_closure_drop_function().into(),
        compile_arity().into(),
        environment.into(),
    ])
}

pub fn compile_environment(
    definition: &mir::ir::Definition,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Record {
    fmm::types::Record::new(
        definition
            .environment()
            .iter()
            .map(|argument| compile(argument.type_(), types))
            .collect(),
    )
}

pub fn compile_unsized_environment() -> fmm::types::Record {
    fmm::types::Record::new(vec![])
}

pub fn compile_entry_function(
    type_: &mir::types::Function,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![compile_untyped_closure_pointer().into()]
            .into_iter()
            .chain(
                type_
                    .arguments()
                    .into_iter()
                    .map(|type_| compile(type_, types)),
            )
            .collect(),
        compile(type_.result(), types),
        fmm::types::CallingConvention::Source,
    )
}

// We can't type this strongly as F-- doesn't support recursive types.
pub fn compile_untyped_closure_pointer() -> fmm::types::Pointer {
    fmm::types::Pointer::new(fmm::types::Record::new(vec![]))
}

pub fn compile_foreign_function(
    function: &mir::types::Function,
    calling_convention: mir::ir::CallingConvention,
    types: &HashMap<String, mir::types::RecordBody>,
) -> fmm::types::Function {
    fmm::types::Function::new(
        function
            .arguments()
            .into_iter()
            .map(|type_| compile(type_, types))
            .collect(),
        compile(function.result(), types),
        compile_calling_convention(calling_convention),
    )
}

fn compile_calling_convention(
    calling_convention: mir::ir::CallingConvention,
) -> fmm::types::CallingConvention {
    match calling_convention {
        mir::ir::CallingConvention::Source => fmm::types::CallingConvention::Source,
        mir::ir::CallingConvention::Target => fmm::types::CallingConvention::Target,
    }
}

pub fn compile_closure_drop_function() -> fmm::types::Function {
    // The argument is a closure pointer.
    fmm::types::Function::new(
        vec![fmm::types::Primitive::PointerInteger.into()],
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
    )
}

pub fn compile_arity() -> fmm::types::Primitive {
    fmm::types::Primitive::PointerInteger
}
