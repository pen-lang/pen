use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure},
};
use std::error::Error;

pub fn link(
    infrastructure: &Infrastructure,
    archive_files: &[FilePath],
    test_information_file: &FilePath,
    test_file: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.test_linker.link(
        archive_files,
        test_information_file,
        test_file,
        &file_path_resolver::resolve_test_directory(output_directory),
    )?;

    Ok(())
}
