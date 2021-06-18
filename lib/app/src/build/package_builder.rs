use super::build_infrastructure::BuildInfrastructure;
use super::module_finder;
use crate::infra::FilePath;
use std::error::Error;

pub fn build_package(
    infrastructure: &BuildInfrastructure,
    package_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    // TODO Build external packages.
    let source_file_paths = module_finder::find_modules(infrastructure, package_directory)?;

    infrastructure.builder.build(&source_file_paths)?;

    Ok(())
}
