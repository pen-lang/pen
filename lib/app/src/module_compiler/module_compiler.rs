use super::{
    compile_configuration::CompileConfiguration, compile_infrastructure::CompileInfrastructure,
};
use crate::infra::FilePath;
use std::error::Error;

// TODO Pass a package configuration file path.
pub fn compile_module(
    infrastructure: &CompileInfrastructure,
    source_file: &FilePath,
    object_file: &FilePath,
    interface_file: &FilePath,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn Error>> {
    // TODO Compile module imports.
    let (module, module_interface) = lang::hir_mir::compile(
        &lang::ast_hir::compile(
            &lang::parse::parse(
                &infrastructure.file_system.read_to_string(source_file)?,
                &infrastructure.file_path_displayer.display(source_file),
            )?,
            &format!(
                "{}:",
                infrastructure.file_path_displayer.display(source_file)
            ),
            &[],
        )?,
        &compile_configuration.list_type,
    )?;

    infrastructure.file_system.write(
        object_file,
        &fmm_llvm::compile_to_bit_code(
            &fmm::analysis::transform_to_cps(
                &mir_fmm::compile(&module)?,
                fmm::types::VOID_TYPE.clone(),
            )?,
            &compile_configuration.heap,
            None,
        )?,
    )?;
    infrastructure.file_system.write(
        interface_file,
        serde_json::to_string(&module_interface)?.as_bytes(),
    )?;

    Ok(())
}
