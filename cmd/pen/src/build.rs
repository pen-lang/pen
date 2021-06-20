use super::{file_path_configuration::FILE_PATH_CONFIGURATION, main_package_directory_finder};
use crate::file_path_configuration::BUILD_CONFIGURATION_FILENAME;
use std::sync::Arc;

const OUTPUT_DIRECTORY: &str = ".pen";

pub fn build() -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;

    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory.clone(),
    ));
    let file_system = Arc::new(infra::FileSystem::new(file_path_converter.clone()));

    let main_package_directory =
        file_path_converter.convert_to_file_path(&main_package_directory)?;
    let output_directory =
        main_package_directory.join(&app::infra::FilePath::new(vec![OUTPUT_DIRECTORY]));

    app::package_manager::initialize_main_package(
        &app::package_manager::PackageManagerInfrastructure {
            external_package_initializer: Arc::new(infra::ExternalPackageInitializer::new(
                file_system.clone(),
                file_path_converter.clone(),
            )),
            package_configuration_reader: Arc::new(infra::JsonPackageConfigurationReader::new(
                file_system.clone(),
                BUILD_CONFIGURATION_FILENAME,
            )),
            file_system: file_system.clone(),
        },
        &main_package_directory,
        &output_directory.join(&app::infra::FilePath::new(vec!["packages"])),
    )?;

    app::package_builder::build_main_package(
        &app::package_builder::BuildInfrastructure {
            module_builder: Arc::new(infra::NinjaModuleBuilder::new(
                file_system.clone(),
                file_path_converter,
            )),
            file_system,
            file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        },
        &main_package_directory,
        &output_directory,
    )?;

    Ok(())
}
