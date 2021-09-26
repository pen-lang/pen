use crate::{
    common::{dependency_serializer, file_path_resolver, module_id_calculator},
    error::ApplicationError,
    infra::{FilePath, Infrastructure, OBJECT_DIRECTORY},
};
use std::{collections::HashMap, error::Error};

#[allow(clippy::too_many_arguments)]
pub fn resolve(
    infrastructure: &Infrastructure,
    package_directory: &FilePath,
    source_file: &FilePath,
    object_file: &FilePath,
    prelude_interface_files: &[FilePath],
    output_directory: &FilePath,
    dependency_file: &FilePath,
    build_script_dependency_file: &FilePath,
) -> Result<(), Box<dyn Error>> {
    let interface_files = parse::parse(
        &infrastructure.file_system.read_to_string(source_file)?,
        &infrastructure.file_path_displayer.display(source_file),
    )?
    .imports()
    .iter()
    .map(|import| {
        let source_file = match import.module_path() {
            ast::ModulePath::Internal(path) => file_path_resolver::resolve_source_file(
                package_directory,
                path.components(),
                &infrastructure.file_path_configuration,
            ),
            ast::ModulePath::External(path) => file_path_resolver::resolve_source_file(
                &file_path_resolver::resolve_package_directory(
                    output_directory,
                    infrastructure
                        .package_configuration_reader
                        .get_dependencies(package_directory)?
                        .get(path.package())
                        .ok_or_else(|| ApplicationError::PackageNotFound(path.package().into()))?,
                ),
                path.components(),
                &infrastructure.file_path_configuration,
            ),
        };

        if !infrastructure.file_system.exists(&source_file) {
            return Err(ApplicationError::ModuleNotFound(import.module_path().to_string()).into());
        }

        Ok((
            import.module_path().clone(),
            output_directory.join(
                &FilePath::new(vec![
                    OBJECT_DIRECTORY,
                    &module_id_calculator::calculate(&source_file),
                ])
                .with_extension(
                    infrastructure
                        .file_path_configuration
                        .interface_file_extension,
                ),
            ),
        ))
    })
    .collect::<Result<HashMap<_, _>, Box<dyn Error>>>()?;

    infrastructure.file_system.write(
        dependency_file,
        &dependency_serializer::serialize(&interface_files, prelude_interface_files)?,
    )?;

    infrastructure.file_system.write(
        build_script_dependency_file,
        infrastructure
            .build_script_dependency_compiler
            .compile(
                object_file,
                &interface_files
                    .values()
                    .chain(prelude_interface_files)
                    .cloned()
                    .collect::<Vec<_>>(),
            )
            .as_bytes(),
    )?;

    Ok(())
}
