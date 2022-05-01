use super::application_configuration::ApplicationConfiguration;
use crate::{
    common::file_path_resolver,
    error::ApplicationError,
    external_package_configuration_reader, external_package_topological_sorter,
    infra::{FilePath, Infrastructure},
    package_build_script_compiler, PackageType,
};
use std::error::Error;

pub fn build(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    target_triple: Option<&str>,
    prelude_package_url: &url::Url,
    ffi_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let child_build_script_files = [package_build_script_compiler::compile_modules(
        infrastructure,
        main_package_directory,
        output_directory,
        application_configuration,
    )?]
    .into_iter()
    .chain(
        external_package_topological_sorter::sort(
            &external_package_configuration_reader::read_all(
                infrastructure,
                main_package_directory,
                output_directory,
            )?,
        )?
        .iter()
        .chain([prelude_package_url, ffi_package_url])
        .map(|url| {
            file_path_resolver::resolve_external_package_build_script_file(
                output_directory,
                url,
                &infrastructure.file_path_configuration,
            )
        }),
    )
    .chain(
        if infrastructure
            .package_configuration_reader
            .read(main_package_directory)?
            .type_()
            == PackageType::Application
        {
            Some(package_build_script_compiler::compile_application(
                infrastructure,
                main_package_directory,
                output_directory,
                prelude_package_url,
                ffi_package_url,
                application_configuration,
            )?)
        } else {
            None
        },
    )
    .collect::<Vec<_>>();

    infrastructure
        .build_script_runner
        .run(&package_build_script_compiler::compile_main(
            infrastructure,
            prelude_package_url,
            output_directory,
            target_triple,
            &child_build_script_files,
        )?)
        .map_err(|_| ApplicationError::Build)?;

    Ok(())
}
