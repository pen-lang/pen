mod module_dependency_compiler_infrastructure;

use crate::infra::FilePath;
pub use module_dependency_compiler_infrastructure::*;
use std::error::Error;

pub fn compile_dependency(
    infrastructure: &ModuleDependencyCompilerInfrastructure,
    package_directory: &FilePath,
    source_file: &FilePath,
    object_file: &FilePath,
    dependency_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    infrastructure.file_system.write(
        dependency_file,
        infrastructure
            .dependency_compiler
            .compile(
                object_file,
                &lang::parse::parse(
                    &infrastructure.file_system.read_to_string(source_file)?,
                    &infrastructure.file_path_displayer.display(source_file),
                )?
                .imports()
                .iter()
                .map(|import| {
                    match import.module_path() {
                        lang::ast::ModulePath::Internal(path) => package_directory.join(
                            &FilePath::new(path.components().to_vec()).with_extension(
                                infrastructure.file_path_configuration.source_file_extension,
                            ),
                        ),
                        _ => todo!("external paths not supported yet"),
                    }
                    .with_extension(
                        infrastructure
                            .file_path_configuration
                            .interface_file_extension,
                    )
                })
                .collect::<Vec<_>>(),
            )
            .as_bytes(),
    )?;

    Ok(())
}
