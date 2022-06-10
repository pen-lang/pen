use super::pointer;
use crate::{closure, CompileError};
use fnv::FnvHashMap;

pub fn synchronize(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
    _types: &FnvHashMap<String, mir::types::RecordBody>,
) -> Result<(), CompileError> {
    match type_ {
        mir::types::Type::ByteString => {
            pointer::synchronize(builder, expression)?;
        }
        mir::types::Type::Function(_) => {
            builder.if_(
                pointer::is_synchronized(builder, expression)?,
                |builder| -> Result<_, CompileError> {
                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
                |builder| {
                    pointer::synchronize(&builder, expression)?;

                    builder.call(
                        closure::metadata::load_synchronize_function(
                            &builder,
                            builder.load(closure::get_metadata_pointer(expression.clone())?)?,
                        )?,
                        vec![fmm::build::bit_cast(
                            fmm::types::Primitive::PointerInteger,
                            expression.clone(),
                        )
                        .into()],
                    )?;

                    Ok(builder.branch(fmm::ir::VOID_VALUE.clone()))
                },
            )?;
        }
        mir::types::Type::Record(_) => todo!(),
        mir::types::Type::Variant => todo!(),
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {}
    }

    Ok(())
}
