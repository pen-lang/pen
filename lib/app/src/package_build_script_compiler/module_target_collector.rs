use super::{
    module_finder,
    package_build_script_compiler_infrastructure::PackageBuildScriptCompilerInfrastructure,
};
use crate::{
    common::file_path_resolver,
    infra::{FilePath, ModuleTarget},
};
use std::error::Error;

pub fn collect_module_targets(
    infrastructure: &PackageBuildScriptCompilerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<Vec<ModuleTarget>, Box<dyn Error>> {
    Ok(
        module_finder::find_modules(infrastructure, package_directory)?
            .iter()
            .map(|source_file| {
                let (object_file, interface_file) = file_path_resolver::resolve_target_files(
                    output_directory,
                    source_file,
                    &infrastructure.file_path_configuration,
                );

                ModuleTarget::new(
                    package_directory.clone(),
                    source_file.clone(),
                    object_file,
                    interface_file,
                )
            })
            .collect::<Vec<_>>(),
    )
}
