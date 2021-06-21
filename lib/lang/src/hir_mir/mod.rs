mod environment_creator;
mod error;
mod expression_compiler;
mod list_type_configuration;
mod module_compiler;
mod module_interface_compiler;
mod string_type_configuration;
mod transformation;
mod type_checker;
mod type_compiler;
mod type_context;
mod type_extractor;
mod type_inferrer;
mod union_type_creator;

use self::type_context::TypeContext;
use crate::{hir::*, interface};
pub use error::CompileError;
pub use list_type_configuration::ListTypeConfiguration;
pub use string_type_configuration::StringTypeConfiguration;

pub fn compile(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
    string_type_configuration: &StringTypeConfiguration,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    let type_context = TypeContext::new(module, list_type_configuration, string_type_configuration);

    let module = type_inferrer::infer_types(module, type_context.types())?;
    type_checker::check_types(&module, &type_context)?;

    Ok((
        {
            let module = module_compiler::compile(&module, &type_context)?;
            mir::analysis::check_types(&module)?;
            module
        },
        module_interface_compiler::compile(&module)?,
    ))
}

#[cfg(test)]
mod tests {
    use super::{list_type_configuration::LIST_TYPE_CONFIGURATION, *};
    use crate::{
        hir_mir::string_type_configuration::STRING_TYPE_CONFIGURATION, position::Position, types,
    };

    fn compile_module(
        module: &Module,
    ) -> Result<(mir::ir::Module, interface::Module), CompileError> {
        compile(module, &LIST_TYPE_CONFIGURATION, &STRING_TYPE_CONFIGURATION)
    }

    #[test]
    fn compile_empty_module() -> Result<(), CompileError> {
        compile_module(&Module::new(vec![], vec![], vec![], vec![]))?;

        Ok(())
    }

    #[test]
    fn compile_boolean() -> Result<(), CompileError> {
        compile_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::Boolean::new(Position::dummy()),
                    Block::new(vec![], Boolean::new(false, Position::dummy())),
                    Position::dummy(),
                ),
                false,
            )],
        ))?;

        Ok(())
    }

    #[test]
    fn compile_none() -> Result<(), CompileError> {
        compile_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::None::new(Position::dummy()),
                    Block::new(vec![], None::new(Position::dummy())),
                    Position::dummy(),
                ),
                false,
            )],
        ))?;

        Ok(())
    }

    #[test]
    fn compile_number() -> Result<(), CompileError> {
        compile_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::Number::new(Position::dummy()),
                    Block::new(vec![], Number::new(42.0, Position::dummy())),
                    Position::dummy(),
                ),
                false,
            )],
        ))?;

        Ok(())
    }

    #[test]
    fn compile_string() -> Result<(), CompileError> {
        compile_module(&Module::new(
            vec![],
            vec![],
            vec![],
            vec![Definition::without_source(
                "x",
                Lambda::new(
                    vec![],
                    types::ByteString::new(Position::dummy()),
                    Block::new(vec![], ByteString::new("foo", Position::dummy())),
                    Position::dummy(),
                ),
                false,
            )],
        ))?;

        Ok(())
    }
}
