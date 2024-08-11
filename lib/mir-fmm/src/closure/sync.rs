use crate::{
    context::Context,
    reference_count::{self, REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS},
    type_, CompileError,
};
use std::sync::LazyLock;

thread_local! {
    static DUMMY_FUNCTION_TYPE: LazyLock<mir::types::Function> =
        LazyLock::new(|| mir::types::Function::new(vec![], mir::types::Type::None));
}

pub fn compile(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_with_builder(context, |builder, environment_pointer| {
        let environment = builder.load(fmm::build::bit_cast(
            fmm::types::Pointer::new(type_::compile_environment(context, definition)),
            environment_pointer.clone(),
        ))?;

        for (index, free_variable) in definition.environment().iter().enumerate() {
            reference_count::synchronize(
                context,
                builder,
                &builder.deconstruct_record(environment.clone(), index)?,
                free_variable.type_(),
            )?;
        }

        Ok(())
    })
}

pub fn compile_normal_thunk(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_with_builder(context, |builder, environment_pointer| {
        reference_count::synchronize(
            context,
            builder,
            &builder.load(fmm::build::union_address(
                fmm::build::bit_cast(
                    fmm::types::Pointer::new(type_::compile_closure_payload(context, definition)),
                    environment_pointer.clone(),
                ),
                1,
            )?)?,
            definition.result_type(),
        )?;

        Ok(())
    })
}

fn compile_with_builder(
    context: &Context,
    compile_body: impl Fn(
        &fmm::build::InstructionBuilder,
        &fmm::build::TypedExpression,
    ) -> Result<(), CompileError>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let argument = fmm::ir::Argument::new("_closure", fmm::types::Primitive::PointerInteger);

    context.module_builder().define_anonymous_function(
        vec![argument.clone()],
        fmm::types::void_type(),
        |builder| -> Result<_, CompileError> {
            compile_body(
                &builder,
                &super::get_payload_pointer(fmm::build::bit_cast(
                    fmm::types::Pointer::new(type_::compile_unsized_closure(
                        context,
                        &DUMMY_FUNCTION_TYPE.with(|function| (*function).clone()),
                    )),
                    fmm::build::variable(argument.name(), argument.type_().clone()),
                ))?,
            )?;

            Ok(builder.return_(fmm::ir::void_value()))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )
}
