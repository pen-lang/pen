use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use crate::{
    file_path_configuration::{OUTPUT_DIRECTORY, PRELUDE_PACKAGE_URL},
    infrastructure,
};
use std::sync::Arc;

pub fn compile(
    source_file: &str,
    dependency_file: &str,
    object_file: &str,
    interface_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));

    app::module_compiler::compile(
        &infrastructure::create(file_path_converter.clone())?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(dependency_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter.convert_to_file_path(interface_file)?,
        &COMPILE_CONFIGURATION,
        &file_path_converter
            .convert_to_file_path(main_package_directory)?
            .join(&app::infra::FilePath::new(vec![OUTPUT_DIRECTORY])),
        &url::Url::parse(PRELUDE_PACKAGE_URL)?,
    )?;

    Ok(())
}
