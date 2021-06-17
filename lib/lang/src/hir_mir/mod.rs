mod environment;
mod error;
mod expression_compilation;
mod interfaces;
mod list_type_configuration;
mod module_compilation;
mod type_check;
mod type_compilation;
mod type_context;
mod type_extraction;
mod type_inference;
mod union_types;

use self::type_context::TypeContext;
use crate::{hir::*, interface};
pub use error::CompileError;
use list_type_configuration::ListTypeConfiguration;

pub fn compile(
    module: &Module,
    list_type_configuration: &ListTypeConfiguration,
) -> Result<(mir::ir::Module, interface::Module), CompileError> {
    let type_context = TypeContext::new(module, list_type_configuration);

    let module = type_inference::infer_types(module, type_context.types())?;
    type_check::check_types(&module, &type_context)?;

    Ok((
        {
            let module = module_compilation::compile(&module, &type_context)?;
            mir::analysis::check_types(&module)?;
            module
        },
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
                    Lambda::new(
                        vec![],
                        types::Boolean::new(Position::dummy()),
                        Block::new(vec![], Boolean::new(false, Position::dummy())),
                        Position::dummy(),
                    ),
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
                    Lambda::new(
                        vec![],
                        types::None::new(Position::dummy()),
                        Block::new(vec![], None::new(Position::dummy())),
                        Position::dummy(),
                    ),
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
                    Lambda::new(
                        vec![],
                        types::Number::new(Position::dummy()),
                        Block::new(vec![], Number::new(42.0, Position::dummy())),
                        Position::dummy(),
                    ),
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
                    Lambda::new(
                        vec![],
                        types::ByteString::new(Position::dummy()),
                        Block::new(vec![], ByteString::new("foo", Position::dummy())),
                        Position::dummy(),
                    ),
                    false,
                    Position::dummy(),
                )],
            ),
            &LIST_TYPE_CONFIGURATION,
        )?;

        Ok(())
    }
}
