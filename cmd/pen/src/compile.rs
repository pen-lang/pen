use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use std::sync::Arc;

pub fn compile(
    source_file: &str,
    dependency_file: &str,
    object_file: &str,
    interface_file: &str,
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
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(dependency_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter.convert_to_file_path(interface_file)?,
        &COMPILE_CONFIGURATION,
    )?;

    Ok(())
}
