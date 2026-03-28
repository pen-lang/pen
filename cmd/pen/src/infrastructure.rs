use crate::file_path_configuration::{
    BIT_CODE_FILE_EXTENSION, BUILD_CONFIGURATION_FILENAME, DEPENDENCY_FILE_EXTENSION,
    FFI_BUILD_SCRIPT_BASENAME, FILE_PATH_CONFIGURATION, LANGUAGE_ROOT_ENVIRONMENT_VARIABLE,
    LANGUAGE_ROOT_SCHEME, LINK_SCRIPT_BASENAME, NINJA_DYNAMIC_DEPENDENCY_FILE_EXTENSION,
    PACKAGES_DIRECTORY,
};
use std::{error::Error, path::Path, rc::Rc};

pub fn create(
    file_path_converter: Rc<infra::FilePathConverter>,
    main_package_directory: impl AsRef<Path>,
) -> Result<app::infra::Infrastructure, Box<dyn Error>> {
    let file_system = Rc::new(infra::FileSystem::new(file_path_converter.clone()));

    let build_script_compiler = Rc::new(infra::NinjaBuildScriptCompiler::new(
        file_path_converter.clone(),
        BIT_CODE_FILE_EXTENSION,
        DEPENDENCY_FILE_EXTENSION,
        NINJA_DYNAMIC_DEPENDENCY_FILE_EXTENSION,
        FFI_BUILD_SCRIPT_BASENAME,
        LINK_SCRIPT_BASENAME,
    ));

    Ok(app::infra::Infrastructure {
        build_script_dependency_compiler: Rc::new(infra::NinjaBuildScriptDependencyCompiler::new(
            file_path_converter.clone(),
        )),
        external_package_initializer: Rc::new(infra::ExternalPackageInitializer::new(
            file_system.clone(),
            file_path_converter.clone(),
            LANGUAGE_ROOT_SCHEME,
            LANGUAGE_ROOT_ENVIRONMENT_VARIABLE,
            PACKAGES_DIRECTORY,
        )),
        file_system: file_system.clone(),
        file_path_displayer: Rc::new(infra::FilePathDisplayer::new(
            file_path_converter.clone(),
            main_package_directory,
        )),
        file_path_configuration: FILE_PATH_CONFIGURATION.clone().into(),
        build_script_runner: Rc::new(infra::NinjaBuildScriptRunner::new(
            file_path_converter.clone(),
        )),
        build_script_compiler,
        package_configuration_reader: Rc::new(infra::JsonPackageConfigurationReader::new(
            file_system.clone(),
            file_path_converter.clone(),
            BUILD_CONFIGURATION_FILENAME,
        )),
        package_configuration_writer: Rc::new(infra::JsonPackageConfigurationWriter::new(
            file_system,
            BUILD_CONFIGURATION_FILENAME,
        )),
        command_runner: Rc::new(infra::CommandRunner::new(file_path_converter.clone())),
        test_linker: Rc::new(infra::TestLinker::new(
            file_path_converter,
            LANGUAGE_ROOT_ENVIRONMENT_VARIABLE,
        )),
    })
}
