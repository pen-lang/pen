mod external_package_initializer;

use crate::infra::{FilePath, Infrastructure};
use std::error::Error;

pub fn initialize(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
) -> Result<(), Box<dyn Error>> {
    external_package_initializer::initialize_prelude(
        infrastructure,
        prelude_package_url,
        output_directory,
    )?;
    external_package_initializer::initialize(infrastructure, ffi_package_url, output_directory)?;

    external_package_initializer::initialize_dependencies(
        infrastructure,
        package_directory,
        output_directory,
    )?;

    Ok(())
}
