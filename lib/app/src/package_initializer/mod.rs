mod external_package_initializer;
mod package_initializer_infrastructure;

use crate::infra::FilePath;
pub use package_initializer_infrastructure::*;
use std::error::Error;

pub fn initialize(
    infrastructure: &PackageInitializerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    external_package_initializer::initialize_recursively(
        infrastructure,
        package_directory,
        output_directory,
    )?;

    Ok(())
}
