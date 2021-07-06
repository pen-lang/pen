mod module_target_collector;

use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    module_finder, prelude_interface_file_finder,
};
use std::error::Error;

pub fn compile_main(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    child_build_script_files: &[FilePath],
    build_script_file: &FilePath,
    prelude_package_url: &url::Url,
    _is_application: bool,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .build_script_compiler
            .compile_main(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    package_directory,
                    output_directory,
                )?,
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
    prelude_package_url: &url::Url,
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
                &prelude_interface_file_finder::find(
                    infrastructure,
                    output_directory,
                    prelude_package_url,
                )?,
                &file_path_resolver::resolve_external_package_ffi_archive_file(
                    output_directory,
                    package_url,
                    &infrastructure.file_path_configuration,
                ),
                &package_directory,
                output_directory,
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
                output_directory,
            )?
            .as_bytes(),
    )?;

    Ok(())
}
