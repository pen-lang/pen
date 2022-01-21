use super::{reference_count, types, CompileError};
use fnv::{FnvHashMap, FnvHashSet};
use once_cell::sync::Lazy;

const DROP_FUNCTION_ARGUMENT_NAME: &str = "_closure";
const DROP_FUNCTION_ARGUMENT_TYPE: fmm::types::Primitive = fmm::types::Primitive::PointerInteger;

static DUMMY_FUNCTION_TYPE: Lazy<mir::types::Function> = Lazy::new(|| {
    mir::types::Function::new(vec![mir::types::Type::Number], mir::types::Type::Number)
});

pub fn compile_entry_function_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record_address(
        reference_count::compile_untagged_pointer(&closure_pointer.into())?,
        0,
    )?
    .into())
}

pub fn compile_load_entry_function(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    // Entry functions of thunks need to be loaded atomically
    // to make thunk update thread-safe.
    Ok(builder.atomic_load(
        compile_entry_function_pointer(closure_pointer)?,
        fmm::ir::AtomicOrdering::Acquire,
    )?)
}

pub fn compile_drop_function_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record_address(
        reference_count::compile_untagged_pointer(&closure_pointer.into())?,
        1,
    )?
    .into())
}

pub fn compile_load_drop_function(
    builder: &fmm::build::InstructionBuilder,
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(builder.load(compile_drop_function_pointer(closure_pointer)?)?)
}

pub fn compile_payload_pointer(
    closure_pointer: impl Into<fmm::build::TypedExpression>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::record_address(
        reference_count::compile_untagged_pointer(&closure_pointer.into())?,
        2,
    )?
    .into())
}

pub fn compile_closure_content(
    entry_function: impl Into<fmm::build::TypedExpression>,
    drop_function: impl Into<fmm::build::TypedExpression>,
    payload: impl Into<fmm::build::TypedExpression>,
) -> fmm::build::TypedExpression {
    fmm::build::record(vec![
        entry_function.into(),
        drop_function.into(),
        payload.into(),
    ])
    .into()
}

pub fn compile_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_drop_function_with_builder(
        module_builder,
        types,
        |builder, environment_pointer| -> Result<_, CompileError> {
            let environment = builder.load(fmm::build::bit_cast(
                fmm::types::Pointer::new(types::compile_environment(definition, types)),
                environment_pointer.clone(),
            ))?;

            for (index, free_variable) in definition.environment().iter().enumerate() {
                reference_count::drop_expression(
                    builder,
                    &builder.deconstruct_record(environment.clone(), index)?,
                    free_variable.type_(),
                    types,
                )?;
            }

            Ok(())
        },
    )
}

pub fn compile_normal_thunk_drop_function(
    module_builder: &fmm::build::ModuleBuilder,
    definition: &mir::ir::Definition,
    types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_drop_function_with_builder(
        module_builder,
        types,
        |builder, environment_pointer| -> Result<_, CompileError> {
            reference_count::drop_expression(
                builder,
                &builder.load(fmm::build::union_address(
                    fmm::build::bit_cast(
                        fmm::types::Pointer::new(types::compile_closure_payload(definition, types)),
                        environment_pointer.clone(),
                    ),
                    1,
                )?)?,
                definition.result_type(),
                types,
            )?;

            Ok(())
        },
    )
}

fn compile_drop_function_with_builder(
    module_builder: &fmm::build::ModuleBuilder,
    types: &FnvHashMap<String, mir::types::RecordBody>,
    compile_body: impl Fn(
        &fmm::build::InstructionBuilder,
        &fmm::build::TypedExpression,
    ) -> Result<(), CompileError>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    module_builder.define_anonymous_function(
        vec![fmm::ir::Argument::new(
            DROP_FUNCTION_ARGUMENT_NAME,
            DROP_FUNCTION_ARGUMENT_TYPE,
        )],
        |builder| -> Result<_, CompileError> {
            compile_body(
                &builder,
                &compile_payload_pointer(fmm::build::bit_cast(
                    fmm::types::Pointer::new(types::compile_unsized_closure(
                        &DUMMY_FUNCTION_TYPE,
                        types,
                    )),
                    fmm::build::variable(DROP_FUNCTION_ARGUMENT_NAME, DROP_FUNCTION_ARGUMENT_TYPE),
                ))?,
            )?;

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
    )
}
