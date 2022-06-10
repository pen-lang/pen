use crate::{context::Context, reference_count, type_, CompileError};
use once_cell::sync::Lazy;

static DUMMY_FUNCTION_TYPE: Lazy<mir::types::Function> = Lazy::new(|| {
    mir::types::Function::new(vec![mir::types::Type::Number], mir::types::Type::Number)
});

pub fn compile(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_drop_function_with_builder(
        context,
        |builder, environment_pointer| -> Result<_, CompileError> {
            let environment = builder.load(fmm::build::bit_cast(
                fmm::types::Pointer::new(type_::compile_environment(definition, context.types())),
                environment_pointer.clone(),
            ))?;

            for (index, free_variable) in definition.environment().iter().enumerate() {
                reference_count::drop(
                    builder,
                    &builder.deconstruct_record(environment.clone(), index)?,
                    free_variable.type_(),
                    context.types(),
                )?;
            }

            Ok(())
        },
    )
}

pub fn compile_normal_thunk(
    context: &Context,
    definition: &mir::ir::FunctionDefinition,
) -> Result<fmm::build::TypedExpression, CompileError> {
    compile_drop_function_with_builder(
        context,
        |builder, environment_pointer| -> Result<_, CompileError> {
            reference_count::drop(
                builder,
                &builder.load(fmm::build::union_address(
                    fmm::build::bit_cast(
                        fmm::types::Pointer::new(type_::compile_closure_payload(
                            definition,
                            context.types(),
                        )),
                        environment_pointer.clone(),
                    ),
                    1,
                )?)?,
                definition.result_type(),
                context.types(),
            )?;

            Ok(())
        },
    )
}

fn compile_drop_function_with_builder(
    context: &Context,
    compile_body: impl Fn(
        &fmm::build::InstructionBuilder,
        &fmm::build::TypedExpression,
    ) -> Result<(), CompileError>,
) -> Result<fmm::build::TypedExpression, CompileError> {
    let argument = fmm::ir::Argument::new("_closure", fmm::types::Primitive::PointerInteger);

    context.module_builder().define_anonymous_function(
        vec![argument.clone()],
        |builder| -> Result<_, CompileError> {
            compile_body(
                &builder,
                &super::get_payload_pointer(fmm::build::bit_cast(
                    fmm::types::Pointer::new(type_::compile_unsized_closure(
                        &DUMMY_FUNCTION_TYPE,
                        context.types(),
                    )),
                    fmm::build::variable(argument.name(), argument.type_().clone()),
                ))?,
            )?;

            Ok(builder.return_(fmm::ir::VOID_VALUE.clone()))
        },
        fmm::types::VOID_TYPE.clone(),
        fmm::types::CallingConvention::Target,
    )
}
