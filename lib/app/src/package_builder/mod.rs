mod package_builder_infrastructure;

use crate::{
    infra::FilePath,
    package_build_script_compiler::{self, PackageBuildScriptCompilerInfrastructure},
};
pub use package_builder_infrastructure::PackageBuilderInfrastructure;
use std::error::Error;

pub fn build_main_package(
    infrastructure: &PackageBuilderInfrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let build_script_file = output_directory.join(
        &FilePath::new(vec!["main"]).with_extension(
            infrastructure
                .file_path_configuration
                .build_script_file_extension,
        ),
    );

    package_build_script_compiler::compile(
        &PackageBuildScriptCompilerInfrastructure {
            module_build_script_compiler: infrastructure.module_build_script_compiler.clone(),
            file_system: infrastructure.file_system.clone(),
            file_path_configuration: infrastructure.file_path_configuration.clone(),
        },
        main_package_directory,
        output_directory,
        &build_script_file,
    )?;

    infrastructure.module_builder.build(&build_script_file)?;

    Ok(())
}
