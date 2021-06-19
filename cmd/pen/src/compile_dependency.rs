use super::main_package_directory_finder;
use crate::file_path_configuration::FILE_PATH_CONFIGURATION;
use std::sync::Arc;

pub fn compile_dependency(
    package_path: &str,
    source_path: &str,
    object_path: &str,
    dependency_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::compile_dependency::compile_dependency(
        &app::compile_dependency::CompileDependencyInfrastructure {
            dependency_compiler: Arc::new(infra::NinjaDependencyCompiler::new(
                file_path_converter.clone(),
            )),
            file_system: Arc::new(infra::FileSystem::new(file_path_converter.clone())),
            file_path_displayer: Arc::new(infra::FilePathDisplayer::new(
                file_path_converter.clone(),
            )),
            file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        },
        &file_path_converter.convert_to_file_path(package_path)?,
        &file_path_converter.convert_to_file_path(source_path)?,
        &file_path_converter.convert_to_file_path(object_path)?,
        &file_path_converter.convert_to_file_path(dependency_path)?,
    )?;

    Ok(())
}
