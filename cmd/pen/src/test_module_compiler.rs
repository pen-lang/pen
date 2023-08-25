use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use crate::{infrastructure, test_configuration::TEST_CONFIGURATION};
use std::rc::Rc;

pub fn compile(
    source_file: &str,
    dependency_file: &str,
    object_file: &str,
    test_information_file: &str,
    target_triple: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Rc::new(infra::FilePathConverter::new(
        main_package_directory_finder::find()?,
    ));

    app::module_compiler::compile_test(
        &infrastructure::create(file_path_converter.clone(), &main_package_directory)?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(dependency_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &file_path_converter.convert_to_file_path(test_information_file)?,
        target_triple,
        &COMPILE_CONFIGURATION,
        &TEST_CONFIGURATION.test_module_configuration,
    )?;

    Ok(())
}
