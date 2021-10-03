use crate::{infrastructure, main_package_directory_finder};
use std::sync::Arc;

pub fn compile(
    test_information_files: &[&str],
    package_test_information_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let file_path_converter = Arc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::package_test_information_compiler::compile(
        &infrastructure::create(file_path_converter.clone())?,
        &test_information_files
            .iter()
            .map(|file| file_path_converter.convert_to_file_path(file))
            .collect::<Result<Vec<_>, _>>()?,
        &file_path_converter.convert_to_file_path(package_test_information_file)?,
    )?;

    Ok(())
}
