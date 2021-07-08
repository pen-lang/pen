use super::main_package_directory_finder;
use crate::{file_path_configuration::PRELUDE_PACKAGE_URL, infrastructure};
use std::sync::Arc;

pub fn resolve_dependency(
    source_file: &str,
    object_file: &str,
    dependency_file: &str,
    build_script_dependency_file: &str,
    package_directory: &str,
    output_directory: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        &main_package_directory_finder::find()?,
    ));

    app::module_dependency_resolver::resolve(
        &infrastructure::create(file_path_converter.clone())?,
        &file_path_converter.convert_to_file_path(package_directory)?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter.convert_to_file_path(output_directory)?,
        &file_path_converter.convert_to_file_path(dependency_file)?,
        &file_path_converter.convert_to_file_path(build_script_dependency_file)?,
        &url::Url::parse(PRELUDE_PACKAGE_URL)?,
    )?;

    Ok(())
}
