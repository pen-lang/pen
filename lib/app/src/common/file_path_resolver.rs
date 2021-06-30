use super::package_id_calculator;
use crate::{
    common::module_id_calculator,
    infra::{FilePath, FilePathConfiguration, EXTERNAL_PACKAGE_DIRECTORY, OBJECT_DIRECTORY},
};

pub fn resolve_source_file(
    package_directory: &FilePath,
    components: &[String],
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    package_directory
        .join(&FilePath::new(components))
        .with_extension(file_path_configuration.source_file_extension)
}

pub fn resolve_source_file_in_external_package(
    output_directory: &FilePath,
    url: &url::Url,
    components: &[String],
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_source_file(
        &resolve_package_directory(output_directory, url),
        components,
        file_path_configuration,
    )
}

pub fn resolve_target_files(
    output_directory: &FilePath,
    source_file: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> (FilePath, FilePath) {
    let target_file = output_directory.join(&FilePath::new(vec![
        OBJECT_DIRECTORY,
        &module_id_calculator::calculate(source_file),
    ]));

    (
        target_file.with_extension(file_path_configuration.object_file_extension),
        target_file.with_extension(file_path_configuration.interface_file_extension),
    )
}

pub fn resolve_package_directory(output_directory: &FilePath, url: &url::Url) -> FilePath {
    output_directory.join(&FilePath::new(vec![
        EXTERNAL_PACKAGE_DIRECTORY.into(),
        package_id_calculator::calculate(url),
    ]))
}
