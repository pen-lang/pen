use super::package_id_calculator;
use crate::infra::{FilePath, FilePathConfiguration, EXTERNAL_PACKAGE_DIRECTORY};

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
        &output_directory.join(&FilePath::new(vec![
            EXTERNAL_PACKAGE_DIRECTORY,
            &package_id_calculator::calculate_package_id(url),
        ])),
        components,
        file_path_configuration,
    )
}
