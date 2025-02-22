use super::package_id_calculator;
use crate::{
    ApplicationConfiguration,
    common::module_id_calculator,
    infra::{
        ARCHIVE_DIRECTORY, BUILD_SCRIPT_DIRECTORY, EXTERNAL_PACKAGE_DIRECTORY, FilePath,
        FilePathConfiguration, OBJECT_DIRECTORY, TEST_DIRECTORY,
    },
};

const MAIN_ARCHIVE_BASENAME: &str = "main";
const TEST_ARCHIVE_SUFFIX: &str = "_test";

pub fn resolve_object_directory(output_directory: &FilePath) -> FilePath {
    output_directory.join(&FilePath::new([OBJECT_DIRECTORY]))
}

pub fn resolve_archive_directory(output_directory: &FilePath) -> FilePath {
    output_directory.join(&FilePath::new([ARCHIVE_DIRECTORY]))
}

pub fn resolve_build_script_directory(output_directory: &FilePath) -> FilePath {
    output_directory.join(&FilePath::new([BUILD_SCRIPT_DIRECTORY]))
}

pub fn resolve_test_directory(output_directory: &FilePath) -> FilePath {
    output_directory.join(&FilePath::new([TEST_DIRECTORY]))
}

pub fn resolve_source_file(
    package_directory: &FilePath,
    components: &[String],
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    package_directory
        .join(&FilePath::new(components))
        .with_extension(file_path_configuration.source_file_extension)
}

pub fn resolve_object_file(
    output_directory: &FilePath,
    source_file: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_target_file_basename(output_directory, source_file)
        .with_extension(file_path_configuration.object_file_extension)
}

pub fn resolve_interface_file(
    output_directory: &FilePath,
    source_file: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_target_file_basename(output_directory, source_file)
        .with_extension(file_path_configuration.interface_file_extension)
}

pub fn resolve_test_information_file(
    output_directory: &FilePath,
    source_file: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_test_directory(output_directory).join(
        &FilePath::new([&module_id_calculator::calculate(source_file)])
            .with_extension(file_path_configuration.test_information_file_extension),
    )
}

fn resolve_target_file_basename(output_directory: &FilePath, source_file: &FilePath) -> FilePath {
    resolve_object_directory(output_directory).join(&FilePath::new([
        &module_id_calculator::calculate(source_file),
    ]))
}

pub fn resolve_package_directory(output_directory: &FilePath, url: &url::Url) -> FilePath {
    output_directory.join(&FilePath::new(vec![
        EXTERNAL_PACKAGE_DIRECTORY.into(),
        package_id_calculator::calculate(url),
    ]))
}

pub fn resolve_application_file(
    package_directory: &FilePath,
    application_configuration: &ApplicationConfiguration,
) -> FilePath {
    package_directory.join(&FilePath::new([
        &application_configuration.application_filename
    ]))
}

pub fn resolve_main_package_archive_file(
    output_directory: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_package_archive_file(
        output_directory,
        MAIN_ARCHIVE_BASENAME,
        file_path_configuration,
    )
}

pub fn resolve_main_package_test_archive_file(
    output_directory: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_package_archive_file(
        output_directory,
        &(MAIN_ARCHIVE_BASENAME.to_owned() + TEST_ARCHIVE_SUFFIX),
        file_path_configuration,
    )
}

pub fn resolve_external_package_archive_file(
    output_directory: &FilePath,
    url: &url::Url,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_package_archive_file(
        output_directory,
        &package_id_calculator::calculate(url),
        file_path_configuration,
    )
}

fn resolve_package_archive_file(
    output_directory: &FilePath,
    basename: &str,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_archive_directory(output_directory)
        .join(&FilePath::new([format!("lib{basename}")]))
        .with_extension(file_path_configuration.archive_file_extension)
}

pub fn resolve_special_build_script_file(
    output_directory: &FilePath,
    name: &str,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_build_script_directory(output_directory).join(
        &FilePath::new([name]).with_extension(file_path_configuration.build_script_file_extension),
    )
}

pub fn resolve_external_package_build_script_file(
    output_directory: &FilePath,
    url: &url::Url,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_build_script_directory(output_directory).join(
        &FilePath::new([package_id_calculator::calculate(url)])
            .with_extension(file_path_configuration.build_script_file_extension),
    )
}

pub fn resolve_package_test_information_file(
    output_directory: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_test_directory(output_directory).join(
        &FilePath::new(["main"])
            .with_extension(file_path_configuration.test_information_file_extension),
    )
}

pub fn resolve_test_executable_file(output_directory: &FilePath) -> FilePath {
    resolve_test_directory(output_directory).join(&FilePath::new(["test"]))
}

pub fn resolve_module_path_components(
    package_directory: &FilePath,
    module_file_path: &FilePath,
) -> Vec<String> {
    module_file_path
        .relative_to(package_directory)
        .unwrap()
        .with_extension("")
        .components()
        .map(From::from)
        .collect()
}
