pub mod foreign;
pub mod variant;

use crate::context::Context;
use std::cell::LazyCell;

pub const FUNCTION_ARGUMENT_OFFSET: usize = 1;

pub fn compile(context: &Context, type_: &mir::types::Type) -> fmm::types::Type {
    if let Some(type_) = context.fmm_types().borrow().get(type_) {
        return type_.clone();
    }

    let fmm_type = match type_ {
        mir::types::Type::Boolean => fmm::types::Primitive::Boolean.into(),
        mir::types::Type::Function(function) => compile_function(context, function),
        mir::types::Type::None => compile_none(),
        mir::types::Type::Number => fmm::types::Primitive::Float64.into(),
        mir::types::Type::Record(record) => compile_record(context, record),
        mir::types::Type::ByteString => compile_string().into(),
        mir::types::Type::Variant => compile_variant().into(),
    };

    context
        .fmm_types()
        .borrow_mut()
        .insert(type_.clone(), fmm_type.clone());

    fmm_type
}

pub fn compile_function(context: &Context, function: &mir::types::Function) -> fmm::types::Type {
    fmm::types::Pointer::new(compile_unsized_closure(context, function)).into()
}

pub fn compile_none() -> fmm::types::Type {
    fmm::types::void_type().into()
}

pub fn compile_string() -> fmm::types::Pointer {
    thread_local! {
        static TYPE: LazyCell<fmm::types::Pointer> = LazyCell::new(|| {
            fmm::types::Pointer::new(fmm::types::Record::new(vec![
                fmm::types::Primitive::PointerInteger.into(),
                // The first byte of a string
                fmm::types::Primitive::Integer8.into(),
            ]))
        });
    }

    TYPE.with(|type_| (*type_).clone())
}

fn compile_variant() -> fmm::types::Record {
    thread_local! {
        static TYPE: LazyCell<fmm::types::Record> = LazyCell::new(|| {
            fmm::types::Record::new(vec![
                compile_variant_tag().into(),
                compile_variant_payload().into(),
            ])
        });
    }

    TYPE.with(|type_| (*type_).clone())
}

pub fn compile_variant_tag() -> fmm::types::Pointer {
    thread_local! {
        static TYPE: LazyCell<fmm::types::Pointer> = LazyCell::new(|| {
            let payload = fmm::types::Type::from(compile_variant_payload());

            fmm::types::Pointer::new(fmm::types::Record::new(vec![
                // clone function
                fmm::types::Function::new(
                    vec![payload.clone()],
                    payload.clone(),
                    fmm::types::CallingConvention::Target,
                )
                .into(),
                // drop function
                fmm::types::Function::new(
                    vec![payload.clone()],
                    fmm::types::void_type(),
                    fmm::types::CallingConvention::Target,
                )
                .into(),
                // synchronize function
                fmm::types::Function::new(
                    vec![payload],
                    fmm::types::void_type(),
                    fmm::types::CallingConvention::Target,
                )
                .into(),
                fmm::types::generic_pointer_type(),
            ]))
        });
    }

    TYPE.with(|type_| (*type_).clone())
}

pub fn compile_variant_payload() -> fmm::types::Primitive {
    fmm::types::Primitive::Integer64
}

pub fn compile_record(context: &Context, record: &mir::types::Record) -> fmm::types::Type {
    if is_record_boxed(context, record) {
        compile_boxed_record()
    } else {
        compile_unboxed_record(context, record).into()
    }
}

// Box large records. This logic needs to be simple because we also use exactly
// the same one for FFI.
pub fn is_record_boxed(context: &Context, record: &mir::types::Record) -> bool {
    // TODO Unbox small records.
    !context.types()[record.name()].fields().is_empty()
}

pub fn compile_boxed_record() -> fmm::types::Type {
    fmm::types::Pointer::new(fmm::types::Record::new(vec![])).into()
}

pub fn compile_unboxed_record(
    context: &Context,
    record: &mir::types::Record,
) -> fmm::types::Record {
    fmm::types::Record::new(
        context.types()[record.name()]
            .fields()
            .iter()
            .map(|type_| compile(context, type_))
            .collect(),
    )
}

pub fn compile_sized_closure(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> fmm::types::Record {
    compile_raw_closure(
        compile_entry_function(context, definition.type_()),
        compile_closure_payload(context, definition),
    )
}

pub fn compile_closure_payload(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> fmm::types::Type {
    if definition.is_thunk() {
        compile_thunk_payload(context, definition).into()
    } else {
        compile_environment(context, definition).into()
    }
}

pub fn compile_thunk_payload(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> fmm::types::Union {
    fmm::types::Union::new(vec![
        compile_environment(context, definition).into(),
        compile(context, definition.result_type()),
    ])
}

pub fn compile_unsized_closure(
    context: &Context,
    function: &mir::types::Function,
) -> fmm::types::Record {
    compile_raw_closure(
        compile_entry_function(context, function),
        compile_unsized_environment(),
    )
}

fn compile_raw_closure(
    entry_function: fmm::types::Function,
    environment: impl Into<fmm::types::Type>,
) -> fmm::types::Record {
    fmm::types::Record::new(vec![
        entry_function.into(),
        compile_closure_metadata().into(),
        environment.into(),
    ])
}

pub fn compile_environment(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> fmm::types::Record {
    fmm::types::Record::new(
        definition
            .environment()
            .iter()
            .map(|argument| compile(context, argument.type_()))
            .collect(),
    )
}

pub fn compile_unsized_environment() -> fmm::types::Record {
    fmm::types::Record::new(vec![])
}

pub fn compile_entry_function(
    context: &Context,
    type_: &mir::types::Function,
) -> fmm::types::Function {
    fmm::types::Function::new(
        [compile_untyped_closure_pointer().into()]
            .into_iter()
            .chain(
                type_
                    .arguments()
                    .iter()
                    .map(|type_| compile(context, type_)),
            )
            .collect(),
        compile(context, type_.result()),
        fmm::types::CallingConvention::Source,
    )
}

// We can't type this strongly as F-- doesn't support recursive types.
pub fn compile_untyped_closure_pointer() -> fmm::types::Pointer {
    fmm::types::Pointer::new(fmm::types::Record::new(vec![]))
}

fn compile_calling_convention(
    calling_convention: mir::ir::CallingConvention,
) -> fmm::types::CallingConvention {
    match calling_convention {
        mir::ir::CallingConvention::Source => fmm::types::CallingConvention::Source,
        mir::ir::CallingConvention::Target => fmm::types::CallingConvention::Target,
    }
}

pub fn compile_closure_metadata() -> fmm::types::Pointer {
    fmm::types::Pointer::new(fmm::types::Record::new(vec![
        compile_closure_drop_function().into(),
        compile_closure_sync_function().into(),
    ]))
}

fn compile_closure_drop_function() -> fmm::types::Function {
    // The argument is a closure pointer.
    fmm::types::Function::new(
        vec![fmm::types::Primitive::PointerInteger.into()],
        fmm::types::void_type(),
        fmm::types::CallingConvention::Target,
    )
}

fn compile_closure_sync_function() -> fmm::types::Function {
    // The argument is a closure pointer.
    fmm::types::Function::new(
        vec![fmm::types::Primitive::PointerInteger.into()],
        fmm::types::void_type(),
        fmm::types::CallingConvention::Target,
    )
}
