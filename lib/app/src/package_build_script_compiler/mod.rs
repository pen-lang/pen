mod module_target_collector;

use crate::{
    infra::{FilePath, Infrastructure},
    module_finder, prelude_interface_file_finder,
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
        prelude_interface_file_finder::find(infrastructure, output_directory, prelude_package_url)?;

    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .build_script_compiler
            .compile(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    package_directory,
                    output_directory,
                )?,
                child_build_script_files,
                &prelude_interface_files,
                package_directory,
            )?
            .as_bytes(),
    )?;

    Ok(())
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
            .build_script_compiler
            .compile_prelude(&module_target_collector::collect_module_targets(
                infrastructure,
                package_directory,
                output_directory,
            )?)?
            .as_bytes(),
    )?;

    Ok(())
}
