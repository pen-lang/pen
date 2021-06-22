use super::{file_path_configuration::FILE_PATH_CONFIGURATION, main_package_directory_finder};
use crate::file_path_configuration::{BUILD_CONFIGURATION_FILENAME, OUTPUT_DIRECTORY};
use std::sync::Arc;

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
    let module_build_script_compiler = Arc::new(infra::NinjaModuleBuildScriptCompiler::new(
        file_path_converter.clone(),
        OUTPUT_DIRECTORY,
    ));

    app::package_initializer::initialize(
        &app::package_initializer::PackageInitializerInfrastructure {
            external_package_initializer: Arc::new(infra::ExternalPackageInitializer::new(
                file_system.clone(),
                file_path_converter.clone(),
            )),
            package_configuration_reader: Arc::new(infra::JsonPackageConfigurationReader::new(
                file_system.clone(),
                BUILD_CONFIGURATION_FILENAME,
            )),
            module_build_script_compiler: module_build_script_compiler.clone(),
            file_system: file_system.clone(),
            file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        },
        &main_package_directory,
        &output_directory,
    )?;

    app::package_builder::build_main_package(
        &app::package_builder::PackageBuilderInfrastructure {
            module_builder: Arc::new(infra::NinjaModuleBuilder::new(file_path_converter)),
            module_build_script_compiler,
            file_system,
            file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        },
        &main_package_directory,
        &output_directory,
    )?;

    Ok(())
}
