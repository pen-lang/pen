use crate::{
    common::file_path_resolver,
    infra::{FilePath, Infrastructure, TestModuleTarget},
    test_module_finder,
};
use std::error::Error;

pub fn collect(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<Vec<TestModuleTarget>, Box<dyn Error>> {
    Ok(test_module_finder::find(infrastructure, package_directory)?
        .iter()
        .map(|source_file| {
            TestModuleTarget::new(
                package_directory.clone(),
                source_file.clone(),
                file_path_resolver::resolve_object_file(
                    output_directory,
                    source_file,
                    &infrastructure.file_path_configuration,
                ),
                file_path_resolver::resolve_test_interface_file(
                    output_directory,
                    source_file,
                    &infrastructure.file_path_configuration,
                ),
            )
        })
        .collect::<Vec<_>>())
}
