use super::compile_infrastructure::CompileInfrastructure;
use crate::infra::FilePath;
use lang::hir_mir::ListTypeConfiguration;

// TODO Pass a package configuration file path.
pub fn compile_module(
    infrastructure: &CompileInfrastructure,
    source_file_path: &FilePath,
    object_file_path: &FilePath,
    interface_file_path: &FilePath,
    name_prefix: &str,
    list_type_configuration: &ListTypeConfiguration,
) -> Result<(), Box<dyn std::error::Error>> {
    let (mir_module, module_interface) = lang::hir_mir::compile(
        &lang::ast_hir::compile(
            &lang::parse::parse(
                &infrastructure
                    .file_system
                    .read_to_string(source_file_path)?,
                &infrastructure.file_path_displayer.display(source_file_path),
            )?,
            name_prefix,
            &[],
        )?,
        list_type_configuration,
    )?;

    infrastructure.file_system.write(
        object_file_path,
        &fmm_llvm::compile_to_bit_code(
            &fmm::analysis::transform_to_cps(
                &mir_fmm::compile(&mir_module)?,
                fmm::types::VOID_TYPE.clone(),
            )?,
            &fmm_llvm::HeapConfiguration {
                allocate_function_name: "malloc".into(),
                reallocate_function_name: "realloc".into(),
                free_function_name: "free".into(),
            },
            None,
        )?,
    )?;
    infrastructure.file_system.write(
        interface_file_path,
        serde_json::to_string(&module_interface)?.as_bytes(),
    )?;

    Ok(())
}
