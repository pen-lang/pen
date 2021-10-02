use crate::{
    infra::{FilePath, Infrastructure},
    package_test_builder, ApplicationConfiguration, TestConfiguration,
};
use std::error::Error;

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

    // TODO Run a test executable.

    Ok(())
}
