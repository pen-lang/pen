use super::main_package_directory_finder;
use crate::file_path_configuration::FILE_PATH_CONFIGURATION;
use std::sync::Arc;

pub fn compile_dependency(
    package_directory: &str,
    source_file: &str,
    object_file: &str,
    dependency_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::module_dependency_compiler::compile_dependency(
        &app::module_dependency_compiler::CompileDependencyInfrastructure {
            dependency_compiler: Arc::new(infra::NinjaDependencyCompiler::new(
                file_path_converter.clone(),
            )),
            file_system: Arc::new(infra::FileSystem::new(file_path_converter.clone())),
            file_path_displayer: Arc::new(infra::FilePathDisplayer::new(
                file_path_converter.clone(),
            )),
            file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        },
        &file_path_converter.convert_to_file_path(package_directory)?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter.convert_to_file_path(dependency_file)?,
    )?;

    Ok(())
}
