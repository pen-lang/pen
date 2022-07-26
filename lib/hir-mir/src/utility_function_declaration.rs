use crate::{context::CompileContext, type_, CompileError};
use fnv::FnvHashSet;
use hir::ir::Module;

pub const LOCAL_DEBUG_FUNCTION_NAME: &str = "__debug";
pub const LOCAL_RACE_FUNCTION_NAME: &str = "__race";
pub const LOCAL_SPAWN_FUNCTION_NAME: &str = "__spawn";

// We cannot use foreign function definitions for those built-in functions
// because they might be defined in the same file. So we first alias them to use
// them in code generation.
//
// TODO Rename this function runtime_function_declaration.
// TODO Test this.
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
