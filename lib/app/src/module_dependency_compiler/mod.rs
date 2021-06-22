mod error;
mod module_dependency_compiler_infrastructure;

use crate::{
    common::{module_id_calculator, module_path_resolver},
    infra::{FilePath, OBJECT_DIRECTORY},
};
use error::ModuleDependencyCompilerError;
pub use module_dependency_compiler_infrastructure::*;
use std::error::Error;

pub fn compile_dependency(
    infrastructure: &ModuleDependencyCompilerInfrastructure,
    package_directory: &FilePath,
    source_file: &FilePath,
    object_file: &FilePath,
    output_directory: &FilePath,
    dependency_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let package_configuration = infrastructure
        .package_configuration_reader
        .read(package_directory)?;

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
                    let source_file = match import.module_path() {
                        lang::ast::ModulePath::Internal(path) => {
                            module_path_resolver::resolve_source_file(
                                package_directory,
                                path.components(),
                                &infrastructure.file_path_configuration,
                            )
                        }
                        lang::ast::ModulePath::External(path) => {
                            module_path_resolver::resolve_source_file_in_external_package(
                                output_directory,
                                package_configuration
                                    .dependencies
                                    .get(path.package())
                                    .ok_or_else(|| {
                                        ModuleDependencyCompilerError::PackageNotFound(
                                            path.package().into(),
                                        )
                                    })?,
                                path.components(),
                                &infrastructure.file_path_configuration,
                            )
                        }
                    };

                    Ok(output_directory.join(
                        &FilePath::new(vec![
                            OBJECT_DIRECTORY,
                            &module_id_calculator::calculate(&source_file),
                        ])
                        .with_extension(
                            infrastructure
                                .file_path_configuration
                                .interface_file_extension,
                        ),
                    ))
                })
                .collect::<Result<Vec<_>, Box<dyn Error>>>()?,
            )
            .as_bytes(),
    )?;

    Ok(())
}
