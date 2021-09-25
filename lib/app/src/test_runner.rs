use crate::{
    common::{file_path_resolver, interface_serializer},
    infra::{FilePath, Infrastructure},
    module_finder,
};
use std::error::Error;

pub fn run(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let source_files = module_finder::find(infrastructure, main_package_directory)?
        .into_iter()
        .filter(|file| {
            file.has_extension(infrastructure.file_path_configuration.test_file_extension)
        })
        .collect::<Vec<_>>();

    let interface_files = source_files
        .iter()
        .map(|file| {
            file_path_resolver::resolve_interface_file(
                output_directory,
                file,
                &infrastructure.file_path_configuration,
            )
        })
        .collect::<Vec<_>>();

    let interfaces = interface_files
        .iter()
        .map(|file| -> Result<interface::Module, Box<dyn Error>> {
            Ok(interface_serializer::deserialize(
                &infrastructure.file_system.read_to_vec(&file)?,
            )?)
        })
        .collect::<Result<Vec<_>, _>>()?
        .iter()
        .flat_map(|interface| {
            interface.declarations().iter().filter(|declaration| {
                declaration
                    .name()
                    .starts_with(infrastructure.test_configuration.test_function_prefix)
            })
        });

    Ok(())
}
