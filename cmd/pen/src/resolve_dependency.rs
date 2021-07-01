use super::main_package_directory_finder;
use crate::{file_path_configuration::OUTPUT_DIRECTORY, infrastructure};
use std::sync::Arc;

pub fn resolve_dependency(
    package_directory: &str,
    source_file: &str,
    object_file: &str,
    dependency_file: &str,
    build_script_dependency_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(&main_package_directory));

    app::module_dependency_resolver::resolve(
        &infrastructure::create(file_path_converter.clone())?,
        &file_path_converter.convert_to_file_path(package_directory)?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter
            .convert_to_file_path(main_package_directory)?
            .join(&app::infra::FilePath::new(vec![OUTPUT_DIRECTORY])),
        &file_path_converter.convert_to_file_path(dependency_file)?,
        &file_path_converter.convert_to_file_path(build_script_dependency_file)?,
    )?;

    Ok(())
}
