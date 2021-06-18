use super::{
    compile_configuration::CompileConfiguration, compile_infrastructure::CompileInfrastructure,
};
use crate::infra::FilePath;

// TODO Pass a package configuration file path.
pub fn compile_module(
    infrastructure: &CompileInfrastructure,
    source_file_path: &FilePath,
    object_file_path: &FilePath,
    module_prefix: &str,
    package_prefix: &str,
    compile_configuration: &CompileConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let full_prefix = package_prefix.to_owned() + module_prefix;

    // TODO Compile module imports.
    let (module, module_interface) = lang::hir_mir::compile(
        &lang::ast_hir::compile(
            &lang::parse::parse(
                &infrastructure
                    .file_system
                    .read_to_string(source_file_path)?,
                &infrastructure.file_path_displayer.display(source_file_path),
            )?,
            &full_prefix,
            &[],
        )?,
        &compile_configuration.list_type,
    )?;

    infrastructure.file_system.write(
        object_file_path,
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
        &object_file_path.with_extension("json"),
        serde_json::to_string(&module_interface)?.as_bytes(),
    )?;

    Ok(())
}
