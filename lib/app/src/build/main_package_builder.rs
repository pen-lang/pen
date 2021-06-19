use super::{build_infrastructure::BuildInfrastructure, module_target_collector};
use crate::infra::FilePath;
use std::error::Error;

pub fn build_main_package(
    infrastructure: &BuildInfrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.module_builder.build(
        &module_target_collector::collect_module_targets(
            infrastructure,
            "main",
            main_package_directory,
            output_directory,
        )?,
        output_directory,
    )?;

    Ok(())
}
