use super::compile_configuration::COMPILE_CONFIGURATION;
use std::sync::Arc;

const BUILD_CONFIGURATION_FILENAME: &str = "pen.json";

pub fn compile(
    source_path: &str,
    object_path: &str,
    module_prefix: &str,
    package_prefix: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let package_directory = find_package_directory()?;

    let file_path_converter = Arc::new(infra::FilePathConverter::new(package_directory));
    let file_system = Arc::new(infra::FileSystem::new(file_path_converter.clone()));
    let file_path_displayer = Arc::new(infra::FilePathDisplayer::new(file_path_converter.clone()));

    app::compile::compile_module(
        &app::compile::CompileInfrastructure {
            file_system,
            file_path_displayer,
        },
        &file_path_converter.convert_to_file_path(source_path)?,
        &file_path_converter.convert_to_file_path(object_path)?,
        module_prefix,
        package_prefix,
        &COMPILE_CONFIGURATION,
    )?;

    Ok(())
}

fn find_package_directory() -> Result<std::path::PathBuf, Box<dyn std::error::Error>> {
    let mut directory: &std::path::Path = &std::env::current_dir()?;

    while !directory.join(BUILD_CONFIGURATION_FILENAME).exists() {
        directory = directory.parent().ok_or_else(|| {
            std::io::Error::new(
                std::io::ErrorKind::NotFound,
                format!(
                    "file {} not found in any parent directory",
                    BUILD_CONFIGURATION_FILENAME,
                ),
            )
        })?
    }

    Ok(directory.into())
}
