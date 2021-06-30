mod external_package_initializer;
mod package_initializer_infrastructure;

use crate::infra::{FilePath, PreludePackageConfiguration};
pub use package_initializer_infrastructure::*;
use std::error::Error;

pub fn initialize(
    infrastructure: &PackageInitializerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_configuration: &PreludePackageConfiguration,
) -> Result<(), Box<dyn Error>> {
    external_package_initializer::initialize(
        infrastructure,
        &prelude_package_configuration.url,
        output_directory,
        None,
    )?;

    external_package_initializer::initialize_recursively(
        infrastructure,
        package_directory,
        output_directory,
        Some(prelude_package_configuration),
    )?;

    Ok(())
}
