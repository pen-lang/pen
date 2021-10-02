use crate::{
    common::{file_path_resolver, test_interface_serializer},
    infra::{FilePath, Infrastructure},
    package_test_builder, test_module_finder, ApplicationConfiguration, TestConfiguration,
};
use std::{collections::BTreeMap, error::Error};

pub fn run(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
    test_configuration: &TestConfiguration,
) -> Result<(), Box<dyn Error>> {
    package_test_builder::build(
        infrastructure,
        main_package_directory,
        output_directory,
        prelude_package_url,
        application_configuration,
        test_configuration,
    )?;

    let interface_files = test_module_finder::find(infrastructure, main_package_directory)?
        .iter()
        .map(|file| {
            file_path_resolver::resolve_test_interface_file(
                output_directory,
                file,
                &infrastructure.file_path_configuration,
            )
        })
        .collect::<Vec<_>>();

    let interfaces = interface_files
        .iter()
        .map(|file| {
            test_interface_serializer::deserialize(&infrastructure.file_system.read_to_vec(file)?)
        })
        .collect::<Result<Vec<_>, _>>()?;

    let test_functions = interfaces.into_iter().flatten().collect::<BTreeMap<_, _>>();

    dbg!(test_functions);

    Ok(())
}
