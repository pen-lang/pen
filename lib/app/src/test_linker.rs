use crate::{
    common::{file_path_resolver, package_test_information_serializer},
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
        &package_test_information_serializer::deserialize(
            &infrastructure
                .file_system
                .read_to_vec(test_information_file)?,
        )?,
        archive_files,
        test_file,
        &file_path_resolver::resolve_test_directory(output_directory),
    )?;

    Ok(())
}
