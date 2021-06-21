mod module_finder;
mod module_target;
mod module_target_collector;
mod package_build_script_compiler_infrastructure;

use crate::infra::FilePath;
pub use module_target::ModuleTarget;
pub use package_build_script_compiler_infrastructure::PackageBuildScriptCompilerInfrastructure;
use std::error::Error;

pub fn compile(
    infrastructure: &PackageBuildScriptCompilerInfrastructure,
    package_directory: &FilePath,
    output_directory: &FilePath,
    child_build_script_files: &[FilePath],
    build_script_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        build_script_file,
        infrastructure
            .module_build_script_compiler
            .compile(
                &module_target_collector::collect_module_targets(
                    infrastructure,
                    package_directory,
                    output_directory,
                )?,
                child_build_script_files,
            )
            .as_bytes(),
    )?;

    Ok(())
}
