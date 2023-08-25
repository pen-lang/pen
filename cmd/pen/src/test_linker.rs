use crate::{
    file_path_configuration::{DEFAULT_TARGET_DIRECTORY, OUTPUT_DIRECTORY},
    infrastructure, main_package_directory_finder,
};
use std::rc::Rc;

pub fn link(
    archive_files: &[&str],
    package_test_information_file: &str,
    test_file: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Rc::new(infra::FilePathConverter::new(&main_package_directory));

    app::test_linker::link(
        &infrastructure::create(file_path_converter.clone(), &main_package_directory)?,
        &archive_files
            .iter()
            .map(|file| file_path_converter.convert_to_file_path(file))
            .collect::<Result<Vec<_>, _>>()?,
        &file_path_converter.convert_to_file_path(package_test_information_file)?,
        &file_path_converter.convert_to_file_path(test_file)?,
        &file_path_converter
            .convert_to_file_path(main_package_directory)?
            .join(&app::infra::FilePath::new([
                OUTPUT_DIRECTORY,
                DEFAULT_TARGET_DIRECTORY,
            ])),
    )?;

    Ok(())
}
