use super::{compile_configuration::COMPILE_CONFIGURATION, main_package_directory_finder};
use crate::{application_configuration::APPLICATION_CONFIGURATION, infrastructure};
use std::{collections::BTreeMap, error::Error, rc::Rc};

pub fn compile(
    source_file: &str,
    dependency_file: &str,
    object_file: &str,
    context_interface_files: &BTreeMap<&str, &str>,
    target_triple: Option<&str>,
) -> Result<(), Box<dyn std::error::Error>> {
    let main_package_directory = main_package_directory_finder::find()?;
    let file_path_converter = Rc::new(infra::FilePathConverter::new(&main_package_directory));

    app::module_compiler::compile_main(
        &infrastructure::create(file_path_converter.clone(), &main_package_directory)?,
        &file_path_converter.convert_to_file_path(source_file)?,
        &file_path_converter.convert_to_file_path(dependency_file)?,
        &file_path_converter.convert_to_file_path(object_file)?,
        &context_interface_files
            .iter()
            .map(|(&key, path)| Ok((key.into(), file_path_converter.convert_to_file_path(path)?)))
            .collect::<Result<BTreeMap<_, _>, Box<dyn Error>>>()?,
        target_triple,
        &COMPILE_CONFIGURATION,
        &APPLICATION_CONFIGURATION,
    )?;

    Ok(())
}
