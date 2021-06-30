mod external_package_initializer;
mod package_initializer_infrastructure;

use crate::infra::{FilePath, PreludeModuleConfiguration};
pub use package_initializer_infrastructure::*;
use std::error::Error;

pub fn initialize(
    infrastructure: &PackageInitializerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_module_configuration: &PreludeModuleConfiguration,
) -> Result<(), Box<dyn Error>> {
    external_package_initializer::initialize(
        infrastructure,
        &prelude_module_configuration.url,
        output_directory,
        None,
    )?;

    external_package_initializer::initialize_recursively(
        infrastructure,
        package_directory,
        output_directory,
        Some(prelude_module_configuration),
    )?;

    Ok(())
}
