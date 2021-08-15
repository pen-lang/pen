use super::package_id_calculator;
use crate::{
    common::module_id_calculator,
    infra::{
        FilePath, FilePathConfiguration, ARCHIVE_DIRECTORY, EXTERNAL_PACKAGE_DIRECTORY,
        OBJECT_DIRECTORY,
    },
};

const MAIN_ARCHIVE_BASENAME: &str = "main";

pub fn resolve_object_directory(output_directory: &FilePath) -> FilePath {
    output_directory.join(&FilePath::new([OBJECT_DIRECTORY]))
}

pub fn resolve_archive_directory(output_directory: &FilePath) -> FilePath {
    output_directory.join(&FilePath::new([ARCHIVE_DIRECTORY]))
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

pub fn resolve_target_files(
    output_directory: &FilePath,
    source_file: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> (FilePath, FilePath) {
    let target_file = resolve_object_directory(output_directory).join(&FilePath::new([
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

pub fn resolve_main_package_ffi_archive_file(
    output_directory: &FilePath,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_package_ffi_archive_file(
        output_directory,
        MAIN_ARCHIVE_BASENAME,
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

pub fn resolve_external_package_ffi_archive_file(
    output_directory: &FilePath,
    url: &url::Url,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_package_ffi_archive_file(
        output_directory,
        &package_id_calculator::calculate(url),
        file_path_configuration,
    )
}

fn resolve_package_ffi_archive_file(
    output_directory: &FilePath,
    basename: &str,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_package_archive_file(
        output_directory,
        &(basename.to_owned() + "-ffi"),
        file_path_configuration,
    )
}

fn resolve_package_archive_file(
    output_directory: &FilePath,
    basename: &str,
    file_path_configuration: &FilePathConfiguration,
) -> FilePath {
    resolve_archive_directory(output_directory)
        .join(&FilePath::new([basename]))
        .with_extension(file_path_configuration.archive_file_extension)
}
