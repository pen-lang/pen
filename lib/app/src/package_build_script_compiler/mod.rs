mod error;
mod module_target_collector;

use self::error::PackageBuildScriptCompilerError;
use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure, MainModuleTarget},
    module_finder, prelude_interface_file_finder, ApplicationConfiguration,
};
use std::error::Error;

pub fn compile_main(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    child_build_script_files: &[FilePath],
    build_script_file: &FilePath,
    prelude_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let (main_module_targets, module_targets) = module_target_collector::collect_module_targets(
        infrastructure,
        package_directory,
        output_directory,
    )?
    .into_iter()
    .partition::<Vec<_>, _>(|target| {
        target.source_file()
            == &file_path_resolver::resolve_source_file(
                package_directory,
                &[application_configuration.main_module_basename.clone()],
                &infrastructure.file_path_configuration,
            )
    });

    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .build_script_compiler
            .compile_main(
                &module_targets,
                main_module_targets.get(0)
                    .map(|target| -> Result<_, Box<dyn Error>> {
                        let package_configuration = infrastructure
                            .package_configuration_reader
                            .read(package_directory)?;

                        Ok(MainModuleTarget::new(
                            target.source_file().clone(),
                            target.object_file().clone(),
                            {
                                let (_, main_function_interface_file) =
                                    file_path_resolver::resolve_target_files(
                                        output_directory,
                                        &file_path_resolver::resolve_source_file(
                                            &file_path_resolver::resolve_package_directory(
                                                output_directory,
                                                package_configuration
                                                    .dependencies
                                                    .get(&application_configuration.system_package_name).ok_or({
                                                        PackageBuildScriptCompilerError::SystemPackageNotFound
                                                    })?,
                                            ),
                                            &[application_configuration
                                                .main_function_module_basename
                                                .clone()],
                                            &infrastructure.file_path_configuration,
                                        ),
                                        &infrastructure.file_path_configuration,
                                    );

                                main_function_interface_file
                            },
                        ))
                    })
                    .transpose()?
                    .as_ref(),
                child_build_script_files,
                &prelude_interface_file_finder::find(
                    infrastructure,
                    output_directory,
                    prelude_package_url,
                )?,
                &file_path_resolver::resolve_main_package_ffi_archive_file(
                    output_directory,
                    &infrastructure.file_path_configuration,
                ),
                package_directory,
                output_directory,
            )?
            .as_bytes(),
    )?;

    Ok(())
}

pub fn compile_external(
    infrastructure: &Infrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
    build_script_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        file_path_resolver::resolve_package_directory(output_directory, package_url);

    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .build_script_compiler
            .compile_external(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    &package_directory,
                    output_directory,
                )?,
                &file_path_resolver::resolve_external_package_ffi_archive_file(
                    output_directory,
                    package_url,
                    &infrastructure.file_path_configuration,
                ),
                &package_directory,
            )?
            .as_bytes(),
    )?;

    Ok(())
}

pub fn compile_prelude(
    infrastructure: &Infrastructure,
    package_url: &url::Url,
    output_directory: &FilePath,
    build_script_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_directory =
        file_path_resolver::resolve_package_directory(output_directory, package_url);

    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .build_script_compiler
            .compile_prelude(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    &package_directory,
                    output_directory,
                )?,
                &file_path_resolver::resolve_external_package_ffi_archive_file(
                    output_directory,
                    package_url,
                    &infrastructure.file_path_configuration,
                ),
                &package_directory,
            )?
            .as_bytes(),
    )?;

    Ok(())
}
