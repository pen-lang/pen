use super::application_configuration::ApplicationConfiguration;
use crate::{
    common::file_path_resolver,
    external_package_topological_sorter,
    infra::{FilePath, Infrastructure},
    package_build_script_compiler,
};
use std::error::Error;

pub fn build(
    infrastructure: &Infrastructure,
    main_package_directory: &FilePath,
    output_directory: &FilePath,
    prelude_package_url: &url::Url,
    application_configuration: &ApplicationConfiguration,
) -> Result<(), Box<dyn Error>> {
    let child_build_script_files = vec![
        package_build_script_compiler::compile_modules(
            infrastructure,
            main_package_directory,
            output_directory,
            application_configuration,
        )?,
        package_build_script_compiler::compile_test_modules(
            infrastructure,
            main_package_directory,
            output_directory,
        )?,
    ]
    .into_iter()
    .chain(
        external_package_topological_sorter::sort(
            infrastructure,
            main_package_directory,
            output_directory,
        )?
        .iter()
        .chain([prelude_package_url])
        .map(|url| {
            file_path_resolver::resolve_external_package_build_script_file(
                output_directory,
                url,
                &infrastructure.file_path_configuration,
            )
        }),
    )
    .collect::<Vec<_>>();

    infrastructure
        .build_script_runner
        .run(&package_build_script_compiler::compile_main(
            infrastructure,
            prelude_package_url,
            output_directory,
            None,
            &child_build_script_files,
        )?)?;

    Ok(())
}
