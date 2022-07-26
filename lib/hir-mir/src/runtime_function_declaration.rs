use crate::{context::CompileContext, type_, CompileError};
use fnv::FnvHashSet;
use hir::ir::*;

pub const LOCAL_DEBUG_FUNCTION_NAME: &str = "__debug";
pub const LOCAL_RACE_FUNCTION_NAME: &str = "__race";
pub const LOCAL_SPAWN_FUNCTION_NAME: &str = "__spawn";

// We cannot use foreign function definitions for those built-in functions
// because they might be defined in the same file. So we first alias them to use
// them in code generation.
pub fn compile(
    context: &CompileContext,
    module: &Module,
) -> Result<Vec<mir::ir::ForeignDeclaration>, CompileError> {
    let configuration = context.configuration()?;
    let foreign_names = module
        .function_definitions()
        .iter()
        .filter(|definition| definition.foreign_definition_configuration().is_some())
        .map(|definition| definition.original_name())
        .collect::<FnvHashSet<_>>();

    Ok([
        if !foreign_names.contains(&*configuration.debug_function_name) {
            Some(mir::ir::ForeignDeclaration::new(
                LOCAL_DEBUG_FUNCTION_NAME,
                &configuration.debug_function_name,
                mir::types::Function::new(
                    vec![mir::types::Type::ByteString],
                    mir::types::Type::None,
                ),
                mir::ir::CallingConvention::Target,
            ))
        } else {
            None
        },
        if !foreign_names.contains(&*configuration.race_function_name) {
            Some(mir::ir::ForeignDeclaration::new(
                LOCAL_RACE_FUNCTION_NAME,
                &configuration.race_function_name,
                type_::compile_race_function(context)?,
                mir::ir::CallingConvention::Source,
            ))
        } else {
            None
        },
        if !foreign_names.contains(&*configuration.spawn_function_name) {
            Some(mir::ir::ForeignDeclaration::new(
                LOCAL_SPAWN_FUNCTION_NAME,
                &configuration.spawn_function_name,
                type_::compile_spawn_function(),
                mir::ir::CallingConvention::Target,
            ))
        } else {
            None
        },
    ]
    .into_iter()
    .flatten()
    .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compile_configuration::COMPILE_CONFIGURATION;
    use hir::{test::ModuleFake, types};
    use position::{test::PositionFake, Position};

    #[test]
    fn declare_runtime_functions() {
        let module = Module::empty();
        let context = CompileContext::new(&module, Some(COMPILE_CONFIGURATION.clone()));
        let declarations = compile(&context, &module).unwrap();

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
            assert!(declarations
                .iter()
                .any(|declaration| declaration.name() == local_name
                    && declaration.foreign_name() == foreign_name));
        }
    }

    #[test]
    fn do_not_declare_runtime_function_if_defined_in_same_module() {
        let module = Module::empty().set_definitions(vec![FunctionDefinition::new(
            "myDebug",
            &COMPILE_CONFIGURATION.debug_function_name,
            Lambda::new(
                vec![],
                types::None::new(Position::fake()),
                None::new(Position::fake()),
                Position::fake(),
            ),
            None,
            false,
            Position::fake(),
        )]);
        let context = CompileContext::new(&module, Some(COMPILE_CONFIGURATION.clone()));
        let declarations = compile(&context, &module).unwrap();

        assert!(declarations.iter().any(|declaration| declaration.name()
            == LOCAL_DEBUG_FUNCTION_NAME
            && declaration.foreign_name() == COMPILE_CONFIGURATION.debug_function_name));
    }
}
