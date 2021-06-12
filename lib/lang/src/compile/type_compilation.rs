use super::{type_context::TypeContext, CompileError};
use crate::{compile::type_canonicalization, types::Type};

const NONE_RECORD_TYPE_NAME: &str = "_pen_none";

pub fn compile(type_: &Type, type_context: &TypeContext) -> Result<mir::types::Type, CompileError> {
    let compile = |type_| compile(type_, type_context);

    Ok(
        match type_canonicalization::canonicalize(type_, type_context.types())? {
            Type::Boolean(_) => mir::types::Type::Boolean,
            Type::Function(function) => mir::types::Function::new(
                function
                    .arguments()
                    .iter()
                    .map(|type_| compile(type_))
                    .collect::<Result<_, _>>()?,
                compile(function.result())?,
            )
            .into(),
            Type::List(_) => {
                mir::types::Record::new(&type_context.list_type_configuration().list_type_name)
                    .into()
            }
            Type::None(_) => mir::types::Record::new(NONE_RECORD_TYPE_NAME).into(),
            Type::Number(_) => mir::types::Type::Number,
            Type::Record(record) => mir::types::Record::new(record.name()).into(),
            Type::String(_) => mir::types::Type::ByteString,
            Type::Any(_) | Type::Union(_) => mir::types::Type::Variant,
            Type::Reference(_) => unreachable!(),
        },
    )
}
