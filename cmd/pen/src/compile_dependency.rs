use super::main_package_directory_finder;
use crate::file_path_configuration::{
    BUILD_CONFIGURATION_FILENAME, FILE_PATH_CONFIGURATION, OUTPUT_DIRECTORY,
};
use std::sync::Arc;

pub fn compile_dependency(
    package_directory: &str,
    source_file: &str,
    object_file: &str,
    dependency_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Arc::new(infra::FilePathConverter::new(&main_package_directory));
    let file_system = Arc::new(infra::FileSystem::new(file_path_converter.clone()));

    app::module_dependency_resolver::compile_dependency(
        &app::module_dependency_resolver::ModuleDependencyResolverInfrastructure {
            dependency_compiler: Arc::new(infra::NinjaDependencyCompiler::new(
                file_path_converter.clone(),
            )),
            package_configuration_reader: Arc::new(infra::JsonPackageConfigurationReader::new(
                file_system.clone(),
                BUILD_CONFIGURATION_FILENAME,
            )),
            file_system,
            file_path_displayer: Arc::new(infra::FilePathDisplayer::new(
                file_path_converter.clone(),
            )),
            file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        },
        &file_path_converter.convert_to_file_path(package_directory)?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter
            .convert_to_file_path(main_package_directory)?
            .join(&app::infra::FilePath::new(vec![OUTPUT_DIRECTORY])),
        &file_path_converter.convert_to_file_path(dependency_file)?,
    )?;

    Ok(())
}
