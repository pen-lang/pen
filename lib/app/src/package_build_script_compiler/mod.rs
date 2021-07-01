mod module_target_collector;

use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    module_finder,
};
use std::error::Error;

pub fn compile(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    child_build_script_files: &[FilePath],
    build_script_file: &FilePath,
    prelude_package_url: &url::Url,
) -> Result<(), Box<dyn Error>> {
    let prelude_interface_files =
        find_prelude_interface_files(infrastructure, output_directory, prelude_package_url)?;

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

fn find_prelude_interface_files(
    infrastructure: &Infrastructure,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
) -> Result<Vec<FilePath>, Box<dyn Error>> {
    Ok(module_finder::find(
        infrastructure,
        &file_path_resolver::resolve_package_directory(output_directory, prelude_package_url),
    )?
    .iter()
    .map(|source_file| {
        let (_, interface_file) = file_path_resolver::resolve_target_files(
            output_directory,
            source_file,
            &infrastructure.file_path_configuration,
        );

        interface_file
    })
    .collect::<Vec<_>>())
}

pub fn compile_prelude(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    build_script_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .module_build_script_compiler
            .compile_prelude(&module_target_collector::collect_module_targets(
                infrastructure,
                package_directory,
                output_directory,
            )?)?
            .as_bytes(),
    )?;

    Ok(())
}
