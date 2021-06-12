mod error;
mod expressions;
mod interfaces;
mod list_type_configuration;
mod module_compilation;
mod type_canonicalization;
mod type_compilation;
mod type_context;
mod type_equality;
mod type_extraction;
mod type_inference;
mod type_resolution;
mod union_types;

use self::type_context::TypeContext;
use crate::{hir::*, interface};
pub use error::CompileError;
use list_type_configuration::ListTypeConfiguration;

pub fn compile(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
) -> Result<(Vec<u8>, interface::Module), CompileError> {
    let type_context = TypeContext::new(module, list_type_configuration);
    let module = type_inference::infer_types(module, type_context.types())?;

    Ok((
        fmm_llvm::compile_to_bit_code(
            &mir_fmm::compile(&module_compilation::compile(&module, &type_context)?)?,
            &fmm_llvm::HeapConfiguration {
                allocate_function_name: "malloc".into(),
                reallocate_function_name: "realloc".into(),
                free_function_name: "free".into(),
            },
            None,
        )?,
        interfaces::compile(&module)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{list_type_configuration::LIST_TYPE_CONFIGURATION, *};
    use crate::{position::Position, types};

    #[test]
    fn compile_empty_module() -> Result<(), CompileError> {
        compile(
            &Module::new(vec![], vec![], vec![], vec![]),
            &LIST_TYPE_CONFIGURATION,
        )?;

        Ok(())
    }

    #[test]
    fn compile_boolean() -> Result<(), CompileError> {
        compile(
            &Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "x",
                    Boolean::new(false, Position::dummy()),
                    types::Boolean::new(Position::dummy()),
                    false,
                    Position::dummy(),
                )],
            ),
            &LIST_TYPE_CONFIGURATION,
        )?;

        Ok(())
    }

    #[test]
    fn compile_none() -> Result<(), CompileError> {
        compile(
            &Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "x",
                    None::new(Position::dummy()),
                    types::None::new(Position::dummy()),
                    false,
                    Position::dummy(),
                )],
            ),
            &LIST_TYPE_CONFIGURATION,
        )?;

        Ok(())
    }

    #[test]
    fn compile_number() -> Result<(), CompileError> {
        compile(
            &Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "x",
                    Number::new(42.0, Position::dummy()),
                    types::Number::new(Position::dummy()),
                    false,
                    Position::dummy(),
                )],
            ),
            &LIST_TYPE_CONFIGURATION,
        )?;

        Ok(())
    }

    #[test]
    fn compile_string() -> Result<(), CompileError> {
        compile(
            &Module::new(
                vec![],
                vec![],
                vec![],
                vec![Definition::new(
                    "x",
                    ByteString::new("foo", Position::dummy()),
                    types::ByteString::new(Position::dummy()),
                    false,
                    Position::dummy(),
                )],
            ),
            &LIST_TYPE_CONFIGURATION,
        )?;

        Ok(())
    }
}
