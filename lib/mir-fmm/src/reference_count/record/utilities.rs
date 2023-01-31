use crate::{context::Context, type_};

pub fn get_clone_function_name(name: &str) -> String {
    format!("mir:clone:{name}")
}

pub fn get_clone_unboxed_function_name(name: &str) -> String {
    format!("mir:clone:unboxed:{name}")
}

pub fn get_drop_function_name(name: &str) -> String {
    format!("mir:drop:{name}")
}

pub fn get_synchronize_function_name(name: &str) -> String {
    format!("mir:synchronize:{name}")
}

pub fn compile_clone_function_type(
    context: &Context,
    record: &mir::types::Record,
) -> fmm::types::Function {
    let record = type_::compile_record(context, record);

    fmm::types::Function::new(
        vec![record.clone()],
        record,
        fmm::types::CallingConvention::Target,
    )
}

pub fn compile_clone_unboxed_function_type(
    context: &Context,
    record: &mir::types::Record,
) -> fmm::types::Function {
    let record = type_::compile_unboxed_record(context, record);

    fmm::types::Function::new(
        vec![record.clone().into()],
        record,
        fmm::types::CallingConvention::Target,
    )
}

pub fn compile_drop_function_type(
    context: &Context,
    record: &mir::types::Record,
) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![type_::compile_record(context, record)],
        fmm::types::void_type(),
        fmm::types::CallingConvention::Target,
    )
}

pub fn compile_synchronize_function_type(
    context: &Context,
    record: &mir::types::Record,
) -> fmm::types::Function {
    fmm::types::Function::new(
        vec![type_::compile_record(context, record)],
        fmm::types::void_type(),
        fmm::types::CallingConvention::Target,
    )
}
