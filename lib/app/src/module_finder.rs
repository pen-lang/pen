use crate::{
    file_finder,
    infra::{FilePath, Infrastructure},
};
use std::error::Error;

pub fn find(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
) -> Result<Vec<FilePath>, Box<dyn Error>> {
    file_finder::find(
        infrastructure,
        package_directory,
        infrastructure.file_path_configuration.source_file_extension,
    )
}
