use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use crate::{application_configuration::APPLICATION_CONFIGURATION, infrastructure};
use std::sync::Arc;

pub fn compile_main(
    source_file: &str,
    dependency_file: &str,
    object_file: &str,
    main_function_interface_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::module_compiler::compile_main(
        &infrastructure::create(file_path_converter.clone())?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(dependency_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter.convert_to_file_path(main_function_interface_file)?,
        &COMPILE_CONFIGURATION,
        &APPLICATION_CONFIGURATION,
    )?;

    Ok(())
}
