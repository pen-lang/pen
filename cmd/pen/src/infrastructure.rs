use std::{error::Error, sync::Arc};

use crate::file_path_configuration::{
    BIT_CODE_FILE_EXTENSION, BUILD_CONFIGURATION_FILENAME, FILE_PATH_CONFIGURATION,
    LANGUAGE_ROOT_ENVIRONMENT_VARIABLE, LANGUAGE_ROOT_HOST_NAME, OUTPUT_DIRECTORY,
};

pub fn create(
    file_path_converter: Arc<infra::FilePathConverter>,
) -> Result<app::infra::Infrastructure, Box<dyn Error>> {
    let file_system = Arc::new(infra::FileSystem::new(file_path_converter.clone()));

    let build_script_compiler = Arc::new(infra::NinjaBuildScriptCompiler::new(
        file_path_converter.clone(),
        BIT_CODE_FILE_EXTENSION,
        OUTPUT_DIRECTORY,
    ));

    Ok(app::infra::Infrastructure {
        build_script_dependency_compiler: Arc::new(infra::NinjaBuildScriptDependencyCompiler::new(
            file_path_converter.clone(),
        )),
        external_package_initializer: Arc::new(infra::ExternalPackageInitializer::new(
            file_system.clone(),
            file_path_converter.clone(),
            LANGUAGE_ROOT_HOST_NAME,
            LANGUAGE_ROOT_ENVIRONMENT_VARIABLE,
        )),
        file_system: file_system.clone(),
        file_path_displayer: Arc::new(infra::FilePathDisplayer::new(file_path_converter.clone())),
        file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        module_builder: Arc::new(infra::NinjaModuleBuilder::new(file_path_converter)),
        build_script_compiler,
        package_configuration_reader: Arc::new(infra::JsonPackageConfigurationReader::new(
            file_system,
            BUILD_CONFIGURATION_FILENAME,
        )),
    })
}
