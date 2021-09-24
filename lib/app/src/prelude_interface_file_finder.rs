use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
    module_finder,
};
use std::error::Error;

pub fn find(
    infrastructure: &Infrastructure,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
) -> Result<Vec<FilePath>, Box<dyn Error>> {
    Ok(module_finder::find(
        infrastructure,
        &file_path_resolver::resolve_package_directory(output_directory, prelude_package_url),
    )?
    .iter()
    .map(|source_file| {
        file_path_resolver::resolve_interface_file(
            output_directory,
            source_file,
            &infrastructure.file_path_configuration,
        )
    })
    .collect::<Vec<_>>())
}
