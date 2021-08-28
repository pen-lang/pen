use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use crate::infrastructure;
use std::sync::Arc;

pub fn compile(
    source_file: &str,
    object_file: &str,
    interface_file: &str,
    target_triple: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::module_compiler::compile_prelude(
        &infrastructure::create(file_path_converter.clone())?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter.convert_to_file_path(interface_file)?,
        target_triple,
        &COMPILE_CONFIGURATION.instruction,
    )?;

    Ok(())
}
