use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use std::sync::Arc;

pub fn compile(
    source_path: &str,
    object_path: &str,
    module_prefix: &str,
    package_prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::compile::compile_module(
        &app::compile::CompileInfrastructure {
            file_system: Arc::new(infra::FileSystem::new(file_path_converter.clone())),
            file_path_displayer: Arc::new(infra::FilePathDisplayer::new(
                file_path_converter.clone(),
            )),
        },
        &file_path_converter.convert_to_file_path(source_path)?,
        &file_path_converter.convert_to_file_path(object_path)?,
        module_prefix,
        package_prefix,
        &COMPILE_CONFIGURATION,
    )?;

    Ok(())
}
