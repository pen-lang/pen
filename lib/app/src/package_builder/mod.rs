mod package_builder_infrastructure;

use crate::{
    infra::{FilePath, EXTERNAL_PACKAGE_DIRECTORY},
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
        &find_external_package_build_script(infrastructure, output_directory)?,
        &build_script_file,
    )?;

    infrastructure.module_builder.build(&build_script_file)?;

    Ok(())
}

fn find_external_package_build_script(
    infrastructure: &PackageBuilderInfrastructure,
    output_directory: &FilePath,
) -> Result<Vec<FilePath>, Box<dyn std::error::Error>> {
    let external_package_directory =
        output_directory.join(&FilePath::new(vec![EXTERNAL_PACKAGE_DIRECTORY]));

    Ok(
        if infrastructure
            .file_system
            .exists(&external_package_directory)
        {
            infrastructure
                .file_system
                .read_directory(&external_package_directory)?
                .into_iter()
                .filter(|path| {
                    path.has_extension(
                        infrastructure
                            .file_path_configuration
                            .build_script_file_extension,
                    )
                })
                .collect()
        } else {
            vec![]
        },
    )
}
