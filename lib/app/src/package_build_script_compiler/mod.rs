mod module_finder;
mod module_target_collector;
mod package_build_script_compiler_infrastructure;

use crate::{
    common::module_path_resolver,
    infra::{FilePath, PreludePackageConfiguration},
};
pub use package_build_script_compiler_infrastructure::PackageBuildScriptCompilerInfrastructure;
use std::error::Error;

pub fn compile(
    infrastructure: &PackageBuildScriptCompilerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    child_build_script_files: &[FilePath],
    build_script_file: &FilePath,
    prelude_package_configuration: Option<&PreludePackageConfiguration>,
) -> Result<(), Box<dyn Error>> {
    let prelude_interface_files = prelude_package_configuration
        .iter()
        .flat_map(|configuration| {
            let package_directory = module_path_resolver::resolve_package_directory(
                output_directory,
                &configuration.url,
            );

            configuration
                .module_paths
                .iter()
                .map(|path_components| {
                    let (_, interface_file) = module_path_resolver::resolve_target_files(
                        output_directory,
                        &module_path_resolver::resolve_source_file(
                            &package_directory,
                            path_components,
                            &infrastructure.file_path_configuration,
                        ),
                        &infrastructure.file_path_configuration,
                    );

                    interface_file
                })
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .module_build_script_compiler
            .compile(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    package_directory,
                    output_directory,
                )?,
                child_build_script_files,
                &prelude_interface_files,
            )?
            .as_bytes(),
    )?;

    Ok(())
}
