use crate::{CompileError, context::Context, type_};

pub const LOCAL_DEBUG_FUNCTION_NAME: &str = "__debug";
pub const LOCAL_RACE_FUNCTION_NAME: &str = "__race";
pub const LOCAL_SPAWN_FUNCTION_NAME: &str = "__spawn";

// We cannot use foreign function definitions for those built-in functions
// because they might be defined in the same file. So we first alias them to use
// them in code generation.
pub fn compile(context: &Context) -> Result<Vec<mir::ir::ForeignDeclaration>, CompileError> {
    let configuration = context.configuration()?;

    Ok(vec![
        mir::ir::ForeignDeclaration::new(
            LOCAL_DEBUG_FUNCTION_NAME,
            &configuration.debug_function_name,
            mir::types::Function::new(vec![mir::types::Type::ByteString], mir::types::Type::None),
            mir::ir::CallingConvention::Target,
        ),
        mir::ir::ForeignDeclaration::new(
            LOCAL_RACE_FUNCTION_NAME,
            &configuration.race_function_name,
            type_::compile_race_function(context)?,
            mir::ir::CallingConvention::Source,
        ),
        mir::ir::ForeignDeclaration::new(
            LOCAL_SPAWN_FUNCTION_NAME,
            &configuration.spawn_function_name,
            type_::compile_spawn_function(),
            mir::ir::CallingConvention::Source,
        ),
    ])
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::{ir::*, test::ModuleFake};

    #[test]
    fn declare_runtime_functions() {
        let module = Module::empty();
        let declarations =
            compile(&Context::new(&module, Some(COMPILE_CONFIGURATION.clone()))).unwrap();

        for (local_name, foreign_name) in [
            (
                LOCAL_DEBUG_FUNCTION_NAME,
                &COMPILE_CONFIGURATION.debug_function_name,
            ),
            (
                LOCAL_RACE_FUNCTION_NAME,
                &COMPILE_CONFIGURATION.race_function_name,
            ),
            (
                LOCAL_SPAWN_FUNCTION_NAME,
                &COMPILE_CONFIGURATION.spawn_function_name,
            ),
        ] {
            assert!(
                declarations
                    .iter()
                    .any(|declaration| declaration.name() == local_name
                        && declaration.foreign_name() == foreign_name)
            );
        }
    }
}
