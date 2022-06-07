use super::pointer;
use crate::CompileError;

pub fn mark(
    builder: &fmm::build::InstructionBuilder,
    expression: &fmm::build::TypedExpression,
    type_: &mir::types::Type,
) -> Result<(), CompileError> {
    match type_ {
        mir::types::Type::ByteString | mir::types::Type::Function(_) => {
            pointer::synchronize(builder, expression)?;
        }
        mir::types::Type::Record(_) => todo!(),
        mir::types::Type::Variant => todo!(),
        mir::types::Type::Boolean | mir::types::Type::None | mir::types::Type::Number => {}
    }

    Ok(())
}
