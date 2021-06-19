use super::{file_path_configuration::FILE_PATH_CONFIGURATION, main_package_directory_finder};
use std::sync::Arc;

const OUTPUT_DIRECTORY: &str = ".pen";

pub fn build() -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;

    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));
    let main_package_directory =
        file_path_converter.convert_to_file_path(&main_package_directory)?;
    let file_system = Arc::new(infra::FileSystem::new(file_path_converter.clone()));

    app::build::build_main_package(
        &app::build::BuildInfrastructure {
            module_builder: Arc::new(infra::NinjaModuleBuilder::new(
                file_system.clone(),
                file_path_converter,
            )),
            file_system,
            file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        },
        &main_package_directory,
        &main_package_directory.join(&app::infra::FilePath::new(vec![OUTPUT_DIRECTORY])),
    )?;

    Ok(())
}
