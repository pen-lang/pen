use super::{super::error::CompileError, block, count, heap};
use crate::{context::Context, reference_count::REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS};
use std::cell::LazyCell;

// Reference counts are negative for synchronized memory blocks and otherwise
// positive. References to static memory blocks are tagged.

const CLONE_FUNCTION_NAME: &str = "mir:clone:pointer";

thread_local! {
    static CLONE_FUNCTION_VARIABLE: LazyCell<fmm::build::TypedExpression> = LazyCell::new(|| {
        fmm::build::variable(
            CLONE_FUNCTION_NAME,
            fmm::types::Function::new(
                vec![fmm::types::generic_pointer_type()],
                fmm::types::generic_pointer_type(),
                fmm::types::CallingConvention::Target,
            ),
        )
    });
}

pub fn compile_clone_function(context: &Context) -> Result<(), CompileError> {
    const ARGUMENT_NAME: &str = "x";
    let pointer_type = fmm::types::generic_pointer_type();

    context.module_builder().define_function(
        CLONE_FUNCTION_NAME,
        vec![fmm::ir::Argument::new(ARGUMENT_NAME, pointer_type.clone())],
        pointer_type.clone(),
        |builder| -> Result<_, CompileError> {
            let pointer = fmm::build::variable(ARGUMENT_NAME, pointer_type.clone());

            Ok(builder.return_(builder.if_::<CompileError>(
                is_null(&pointer)?,
                |builder| Ok(builder.branch(pointer.clone())),
                |builder| {
                    let count_pointer = block::compile_count_pointer(&pointer)?;
                    let count = builder
                        .atomic_load(count_pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?;

                    builder.if_(
                        count::is_synchronized(&count)?,
                        |builder| -> Result<_, CompileError> {
                            Ok(builder.branch(builder.if_(
                                count::is_static(&count)?,
                                |builder| Ok(builder.branch(fmm::ir::void_value())),
                                |builder| -> Result<_, CompileError> {
                                    builder.atomic_operation(
                                        fmm::ir::AtomicOperator::Subtract,
                                        count_pointer.clone(),
                                        count::compile(1),
                                        fmm::ir::AtomicOrdering::Relaxed,
                                    )?;

                                    Ok(builder.branch(fmm::ir::void_value()))
                                },
                            )?))
                        },
                        |builder| {
                            builder.store(
                                fmm::build::arithmetic_operation(
                                    fmm::ir::ArithmeticOperator::Add,
                                    count.clone(),
                                    count::compile(1),
                                )?,
                                count_pointer.clone(),
                            );

                            Ok(builder.branch(fmm::ir::void_value()))
                        },
                    )?;

                    Ok(builder.branch(pointer.clone()))
                },
            )?))
        },
        REFERENCE_COUNT_FUNCTION_DEFINITION_OPTIONS.clone(),
    )?;

    Ok(())
}

pub fn clone(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::bit_cast(
        pointer.type_().clone(),
        builder.call(
            CLONE_FUNCTION_VARIABLE.with(|variable| (*variable).clone()),
            vec![fmm::build::bit_cast(fmm::types::generic_pointer_type(), pointer.clone()).into()],
        )?,
    )
    .into())
}

pub fn drop(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    drop_content: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    let drop_inner = |builder: &_| {
        drop_content(builder)?;
        heap::free(builder, pointer.clone())
    };

    builder.if_(
        is_null(pointer)?,
        |builder| Ok(builder.branch(fmm::ir::void_value())),
        |builder| -> Result<_, CompileError> {
            let count_pointer = block::compile_count_pointer(pointer)?;
            let count =
                builder.atomic_load(count_pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?;

            Ok(builder.branch(builder.if_(
                count::is_synchronized(&count)?,
                |builder| -> Result<_, CompileError> {
                    Ok(builder.branch(builder.if_(
                        count::is_static(&count)?,
                        |builder| Ok(builder.branch(fmm::ir::void_value())),
                        |builder| -> Result<_, CompileError> {
                            Ok(builder.branch(builder.if_(
                                count::is_synchronized_unique(&builder.atomic_operation(
                                    fmm::ir::AtomicOperator::Add,
                                    count_pointer.clone(),
                                    count::compile(1),
                                    fmm::ir::AtomicOrdering::Release,
                                )?)?,
                                |builder| -> Result<_, CompileError> {
                                    builder.fence(fmm::ir::AtomicOrdering::Acquire);
                                    drop_inner(&builder)?;

                                    Ok(builder.branch(fmm::ir::void_value()))
                                },
                                |builder| Ok(builder.branch(fmm::ir::void_value())),
                            )?))
                        },
                    )?))
                },
                |builder| {
                    Ok(builder.branch(builder.if_(
                        count::is_unique(&count)?,
                        |builder| -> Result<_, CompileError> {
                            drop_inner(&builder)?;

                            Ok(builder.branch(fmm::ir::void_value()))
                        },
                        |builder| {
                            builder.store(
                                fmm::build::arithmetic_operation(
                                    fmm::ir::ArithmeticOperator::Subtract,
                                    count.clone(),
                                    count::compile(1),
                                )?,
                                count_pointer.clone(),
                            );

                            Ok(builder.branch(fmm::ir::void_value()))
                        },
                    )?))
                },
            )?))
        },
    )?;

    Ok(())
}

pub fn synchronize(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
    synchronize_content: impl Fn(&fmm::build::InstructionBuilder) -> Result<(), CompileError>,
) -> Result<(), CompileError> {
    builder.if_(
        is_null(pointer)?,
        |builder| Ok(builder.branch(fmm::ir::void_value())),
        |builder| -> Result<_, CompileError> {
            let count_pointer = block::compile_count_pointer(pointer)?;
            let count =
                builder.atomic_load(count_pointer.clone(), fmm::ir::AtomicOrdering::Relaxed)?;

            Ok(builder.branch(builder.if_(
                count::is_synchronized(&count)?,
                |builder| Ok(builder.branch(fmm::ir::void_value())),
                |builder| -> Result<_, CompileError> {
                    builder.store(count::synchronize(&count)?, count_pointer.clone());

                    synchronize_content(&builder)?;

                    Ok(builder.branch(fmm::ir::void_value()))
                },
            )?))
        },
    )?;

    Ok(())
}

pub fn is_unique(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    builder.if_(
        is_null(pointer)?,
        |builder| Ok(builder.branch(fmm::ir::Primitive::Boolean(false))),
        |builder| {
            // This atomic ordering can be relaxed because blocks get never un-synchronized.
            let count = builder.atomic_load(
                block::compile_count_pointer(pointer)?,
                fmm::ir::AtomicOrdering::Relaxed,
            )?;

            Ok(builder.branch(builder.if_(
                count::is_synchronized(&count)?,
                |builder| -> Result<_, CompileError> {
                    // We need a memory fence of an acquire ordering to synchronize with release by
                    // drops and make a block ready for memory operations.
                    builder.fence(fmm::ir::AtomicOrdering::Acquire);

                    Ok(builder.branch(count::is_synchronized_unique(&count)?))
                },
                |builder| Ok(builder.branch(count::is_unique(&count)?)),
            )?))
        },
    )
}

// Heap blocks are always synchronized by their owners before references are
// shared with other threads. So an ordering to load counts can always be
// relaxed.
pub fn is_synchronized(
    builder: &fmm::build::InstructionBuilder,
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    builder.if_(
        is_null(pointer)?,
        |builder| Ok(builder.branch(fmm::ir::Primitive::Boolean(false))),
        |builder| -> Result<_, CompileError> {
            Ok(builder.branch(count::is_synchronized(&builder.atomic_load(
                block::compile_count_pointer(pointer)?,
                fmm::ir::AtomicOrdering::Relaxed,
            )?)?))
        },
    )
}

fn is_null(
    pointer: &fmm::build::TypedExpression,
) -> Result<fmm::build::TypedExpression, CompileError> {
    Ok(fmm::build::comparison_operation(
        fmm::ir::ComparisonOperator::Equal,
        fmm::build::bit_cast(fmm::types::Primitive::PointerInteger, pointer.clone()),
        fmm::ir::Undefined::new(fmm::types::Primitive::PointerInteger),
    )?
    .into())
}
