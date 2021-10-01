use crate::{
    common::{file_path_resolver, interface_serializer},
    infra::{FilePath, Infrastructure},
    package_test_builder, test_module_finder, ApplicationConfiguration,
};
use std::error::Error;

pub fn run(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    package_test_builder::build(
        infrastructure,
        main_package_directory,
        output_directory,
        prelude_package_url,
        application_configuration,
    )?;

    let interface_files = test_module_finder::find(infrastructure, main_package_directory)?
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
            interface_serializer::deserialize(&infrastructure.file_system.read_to_vec(file)?)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let test_functions = interfaces
        .iter()
        .flat_map(|interface| {
            interface
                .declarations()
                .iter()
                .map(|declaration| declaration.name())
        })
        .collect::<Vec<_>>();

    dbg!(test_functions);

    Ok(())
}
