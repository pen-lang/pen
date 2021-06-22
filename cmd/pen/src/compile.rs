use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use std::sync::Arc;

pub fn compile(
    source_path: &str,
    dependency_path: &str,
    object_path: &str,
    interface_path: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::module_compiler::compile_module(
        &app::module_compiler::ModuleCompilerInfrastructure {
            file_system: Arc::new(infra::FileSystem::new(file_path_converter.clone())),
            file_path_displayer: Arc::new(infra::FilePathDisplayer::new(
                file_path_converter.clone(),
            )),
        },
        &file_path_converter.convert_to_file_path(source_path)?,
        &file_path_converter.convert_to_file_path(dependency_path)?,
        &file_path_converter.convert_to_file_path(object_path)?,
        &file_path_converter.convert_to_file_path(interface_path)?,
        &COMPILE_CONFIGURATION,
    )?;

    Ok(())
}
