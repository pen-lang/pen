mod module_finder;
mod module_target;
mod module_target_collector;
mod package_builder_infrastructure;

use crate::infra::FilePath;
pub use module_target::ModuleTarget;
pub use package_builder_infrastructure::PackageBuilderInfrastructure;
use std::error::Error;

pub fn build_main_package(
    infrastructure: &PackageBuilderInfrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.module_builder.build(
        &module_target_collector::collect_module_targets(
            infrastructure,
            main_package_directory,
            output_directory,
        )?,
        output_directory,
    )?;

    Ok(())
}
